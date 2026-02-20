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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use tracing::{error, info, warn};

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

        // Save config_dir for hot reloading routes
        let config_path = config_dir.join("config.toml");

        loop {
            let (stream, _) = listener.accept().await?;
            let tls_acceptor = tls_acceptor.clone();
            let client = client.clone();

            // Hot reload routes for each connection
            let current_config = DevBindConfig::load(&config_path).unwrap_or_default();
            let routes = Arc::new(current_config.routes);

            tokio::spawn(async move {
                let tls_stream = match tls_acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("TLS handshake error: {}", e);
                        return;
                    }
                };

                let io = TokioIo::new(tls_stream);

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

                        // Find the corresponding local port for this domain
                        let target_port = routes
                            .iter()
                            .find(|r| r.domain.to_lowercase() == host)
                            .map(|r| r.port);

                        if let Some(port) = target_port {
                            info!("Proxying {} to 127.0.0.1:{}", host, port);

                            // Rewrite the URI to point to local backend service
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
                                    // Convert Incoming body to our Full body type for simplicity right now
                                    let (parts, body) = res.into_parts();
                                    let collected_body = body
                                        .collect()
                                        .await
                                        .map(|b| b.to_bytes())
                                        .unwrap_or_default();
                                    Ok::<_, hyper::Error>(Response::from_parts(
                                        parts,
                                        Full::new(collected_body),
                                    ))
                                }
                                Err(e) => {
                                    error!(
                                        "Backend connection failed for {}:{}: {}",
                                        host, port, e
                                    );
                                    Ok(Response::builder()
                                        .status(502)
                                        .body(Full::new(Bytes::from(
                                            "Bad Gateway: Backend unreachable",
                                        )))
                                        .unwrap())
                                }
                            }
                        } else {
                            warn!(
                                "Unknown host requested: '{}'. Registered: {:?}",
                                host,
                                routes.iter().map(|r| &r.domain).collect::<Vec<_>>()
                            );
                            Ok(Response::builder()
                                .status(404)
                                .body(Full::new(Bytes::from(
                                    "Not Found: Domain not registered in DevBind",
                                )))
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
