use crate::cert::CertManager;
use crate::config::DevBindConfig;
use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming, Request, Response};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::ServerConfig;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;

use tracing::{error, info, warn};

// Use a type alias for the boxed body to keep signatures clean.
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn boxed_full(data: impl Into<Bytes>) -> BoxBody {
    Full::new(data.into())
        .map_err(|e: Infallible| match e {})
        .boxed()
}

pub struct ProxyServer {
    config: DevBindConfig,
}

struct SniResolver {
    cert_manager: CertManager,
}

impl std::fmt::Debug for SniResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SniResolver").finish()
    }
}

impl ResolvesServerCert for SniResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let sni = client_hello.server_name()?;
        self.cert_manager.get_or_generate_cert(sni).ok()
    }
}

/// Cached configuration with a freshness timestamp.
struct CachedConfig {
    routes: Arc<HashMap<String, u16>>,
    loaded_at: Instant,
}

impl ProxyServer {
    pub fn new(config: DevBindConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, config_dir: PathBuf) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.config.proxy.port_https);
        let listener = TcpListener::bind(&addr).await?;

        let cert_manager = CertManager::new(&config_dir);
        let resolver = Arc::new(SniResolver { cert_manager });

        let tls_cfg = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

        let client: Client<HttpConnector, Incoming> =
            Client::builder(TokioExecutor::new()).build(HttpConnector::new());

        info!("DevBind proxy listening on https://{}", addr);

        // Spawn a simple HTTP -> HTTPS redirector on port 80
        tokio::spawn(async move {
            if let Ok(http_listener) = TcpListener::bind("127.0.0.1:80").await {
                info!("HTTP to HTTPS redirector listening on 127.0.0.1:80");
                loop {
                    if let Ok((stream, _)) = http_listener.accept().await {
                        let io = TokioIo::new(stream);
                        tokio::spawn(async move {
                            let _ = http1::Builder::new()
                                .serve_connection(
                                    io,
                                    service_fn(|req: Request<Incoming>| async move {
                                        let host = req
                                            .headers()
                                            .get("host")
                                            .and_then(|h| h.to_str().ok())
                                            .unwrap_or("");
                                        let url = format!(
                                            "https://{}{}",
                                            host,
                                            req.uri()
                                                .path_and_query()
                                                .map(|pq| pq.as_str())
                                                .unwrap_or("/")
                                        );
                                        Ok::<_, hyper::Error>(
                                            Response::builder()
                                                .status(301)
                                                .header("Location", url)
                                                .body(Full::new(hyper::body::Bytes::default()))
                                                .unwrap(),
                                        )
                                    }),
                                )
                                .await;
                        });
                    }
                }
            } else {
                warn!("Could not bind to port 80 for HTTP redirects. Are you running an existing web server?");
            }
        });

        // Build initial route map. Reload from disk at most once every CACHE_TTL.
        let config_path = config_dir.join("config.toml");
        let initial_routes =
            build_route_map(&DevBindConfig::load(&config_path).unwrap_or_default());
        let cached_config: Arc<RwLock<CachedConfig>> = Arc::new(RwLock::new(CachedConfig {
            routes: Arc::new(initial_routes),
            loaded_at: Instant::now(),
        }));

        const CACHE_TTL: Duration = Duration::from_secs(5);

        loop {
            let (stream, _) = listener.accept().await?;
            let tls_acceptor = tls_acceptor.clone();
            let client = client.clone();
            let cached_config = cached_config.clone();
            let config_path = config_path.clone();

            tokio::spawn(async move {
                // Refresh config only when TTL has expired — avoids per-connection disk I/O.
                let routes = {
                    let needs_refresh = {
                        let guard = cached_config.read().await;
                        guard.loaded_at.elapsed() > CACHE_TTL
                    };

                    if needs_refresh {
                        // Acquire write lock and double-check to prevent a thundering herd.
                        let mut guard = cached_config.write().await;
                        if guard.loaded_at.elapsed() > CACHE_TTL {
                            if let Ok(new_cfg) = DevBindConfig::load(&config_path) {
                                guard.routes = Arc::new(build_route_map(&new_cfg));
                            }
                            guard.loaded_at = Instant::now();
                        }
                        guard.routes.clone()
                    } else {
                        cached_config.read().await.routes.clone()
                    }
                };

                let tls_stream = match tls_acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("TLS handshake error: {}", e);
                        return;
                    }
                };

                let io = TokioIo::new(tls_stream);

                // Service returns BoxBody so we can either stream or send a static error body.
                let service = service_fn(move |mut req: Request<Incoming>| {
                    let client = client.clone();
                    let routes = routes.clone();

                    async move {
                        let host = req
                            .headers()
                            .get("host")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("")
                            .split(':')
                            .next()
                            .unwrap_or("")
                            .trim_end_matches('.')
                            .to_lowercase();

                        // O(1) lookup via HashMap — no linear scan.
                        if let Some(&port) = routes.get(&host) {
                            info!("Proxying {} to 127.0.0.1:{}", host, port);

                            let uri_string = format!(
                                "http://127.0.0.1:{}{}",
                                port,
                                req.uri()
                                    .path_and_query()
                                    .map(|pq| pq.as_str())
                                    .unwrap_or("/")
                            );
                            *req.uri_mut() = uri_string.parse().unwrap();

                            req.headers_mut()
                                .insert("X-Forwarded-Proto", "https".parse().unwrap());

                            match client.request(req).await {
                                Ok(res) => {
                                    // Stream body directly from backend — no full buffering.
                                    let (parts, body) = res.into_parts();
                                    let streamed = body.map_err(|e| {
                                        error!("Upstream body error: {}", e);
                                        // Propagate as a hyper error to signal broken connection.
                                        e.into()
                                    });
                                    Ok::<Response<BoxBody>, hyper::Error>(Response::from_parts(
                                        parts,
                                        streamed.boxed(),
                                    ))
                                }
                                Err(e) => {
                                    error!(
                                        "Backend connection failed for {}:{}: {}",
                                        host, port, e
                                    );
                                    Ok(Response::builder()
                                        .status(502)
                                        .body(boxed_full("Bad Gateway: Backend unreachable"))
                                        .unwrap())
                                }
                            }
                        } else {
                            warn!(
                                "Unknown host requested: '{}'. Registered: {:?}",
                                host,
                                routes.keys().collect::<Vec<_>>()
                            );
                            Ok(Response::builder()
                                .status(404)
                                .body(boxed_full("Not Found: Domain not registered in DevBind"))
                                .unwrap())
                        }
                    }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection: {:?}", e);
                }
            });
        }
    }
}

/// Builds an O(1) domain→port lookup map from a config.
fn build_route_map(config: &DevBindConfig) -> HashMap<String, u16> {
    config
        .routes
        .iter()
        .map(|r| (r.domain.to_lowercase(), r.port))
        .collect()
}
