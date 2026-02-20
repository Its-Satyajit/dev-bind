use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use rustls::ServerConfig;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use crate::config::DevBindConfig;
use crate::cert::CertManager;
use std::path::PathBuf;

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
        // Try to load or generate the cert for the given SNI domain
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

        // Setup TLS config with SNI resolving using CertManager
        let cert_manager = CertManager::new(&config_dir);
        let resolver = Arc::new(SniResolver { cert_manager });

        let tls_cfg = ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);

        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

        println!("Listening on https://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let tls_acceptor = tls_acceptor.clone();

            tokio::spawn(async move {
                let tls_stream = match tls_acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("TLS handshake error: {}", e);
                        return;
                    }
                };

                let io = TokioIo::new(tls_stream);

                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service_fn(|req: Request<Incoming>| async move {
                        // Very simple echo proxy response for now mapping to stub backends
                        let host = req.headers().get("host")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("unknown");
                        Ok::<_, hyper::Error>(Response::new(format!("DevBind Proxy OK for Host: {}", host)))
                    }))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", e);
                }
            });
        }
    }
}
