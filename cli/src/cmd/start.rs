use anyhow::Result;
use devbind_core::config::DevBindConfig;
use devbind_core::proxy::ProxyServer;
use std::path::PathBuf;
use tracing::{error, info};

pub async fn handle_start(config_path: &PathBuf) -> Result<()> {
    let config = DevBindConfig::load(config_path)?;
    info!(
        "Starting DevBind proxy on port {}...",
        config.proxy.port_https
    );

    let proxy = ProxyServer::new(config);
    let mut config_dir = config_path.clone();
    config_dir.pop(); // Remove config.toml to get the dir

    if let Err(e) = proxy.start(config_dir).await {
        error!("Proxy server terminated with error: {:?}", e);
    }
    Ok(())
}
