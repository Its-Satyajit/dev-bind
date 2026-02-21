use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

pub fn handle_trust(mut config_dir: PathBuf) -> Result<()> {
    config_dir.pop();
    info!("Initiating Root CA trust installation...");
    if let Err(e) = devbind_core::trust::install_root_ca(&config_dir) {
        error!("Failed to install trust: {}", e);
    }
    Ok(())
}

pub fn handle_untrust() -> Result<()> {
    info!("Initiating Root CA trust removal...");
    if let Err(e) = devbind_core::trust::uninstall_root_ca() {
        error!("Failed to uninstall trust: {}", e);
    }
    Ok(())
}
