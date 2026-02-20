use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::{BodyExt, Full};
use rustls::ServerConfig;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use crate::config::DevBindConfig;
use crate::cert::CertManager;
use std::path::PathBuf;
use hyper::body::Bytes;

use tracing::{info, warn, error};

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
        let addr = format!("127.0.0.1:{}", self.config.proxy.listen_port);
        let listener = TcpListener::bind(&addr).await?;

        let cert_manager = CertManager::new(&config_dir);
        let resolver = Arc::new(SniResolver { cert_manager });

        let tls_cfg = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

        let client: Client<HttpConnector, Incoming> = Client::builder(TokioExecutor::new())
            .build(HttpConnector::new());

        info!("DevBind proxy listening on https://{}", addr);

        // Share the routes mapping
        let routes = Arc::new(self.config.routes.clone());

        loop {
            let (stream, _) = listener.accept().await?;
            let tls_acceptor = tls_acceptor.clone();
            let client = client.clone();
            let routes = routes.clone();

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
                        let host = req.headers().get("host")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("")
                            .split(':')
                            .next()
                            .unwrap_or("")
                            .to_string(); // Owned string to avoid borrowing req

                        // Find the corresponding local port for this domain
                        let target_port = routes.iter()
                            .find(|r| r.domain == host)
                            .map(|r| r.port);

                        if let Some(port) = target_port {
                            info!("Proxying {} to 127.0.0.1:{}", host, port);

                            // Rewrite the URI to point to local backend service
                            let uri_string = format!("http://127.0.0.1:{}{}", port, req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/"));
                            *req.uri_mut() = uri_string.parse().unwrap();

                            req.headers_mut().insert("X-Forwarded-Proto", "https".parse().unwrap());

                            match client.request(req).await {
                                Ok(res) => {
                                    // Convert Incoming body to our Full body type for simplicity right now
                                    let (parts, body) = res.into_parts();
                                    let collected_body = body.collect().await.map(|b| b.to_bytes()).unwrap_or_default();
                                    Ok::<_, hyper::Error>(Response::from_parts(parts, Full::new(collected_body)))
                                },
                                Err(e) => {
                                    error!("Backend connection failed for {}:{}: {}", host, port, e);
                                    Ok(Response::builder()
                                        .status(502)
                                        .body(Full::new(Bytes::from("Bad Gateway: Backend unreachable")))
                                        .unwrap())
                                }
                            }
                        } else {
                            warn!("Unknown host requested: {}", host);
                            Ok(Response::builder()
                                .status(404)
                                .body(Full::new(Bytes::from("Not Found: Domain not registered in DevBind")))
                                .unwrap())
                        }
                    }
                });

                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    error!("Error serving connection: {:?}", e);
                }
            });
        }
    }
}
