use anyhow::Result;
use tracing::{error, info};

pub fn handle_install() -> Result<()> {
    info!("Installing DNS integration for .test domains...");
    match devbind_core::setup::install_dns(devbind_core::dns::DNS_LISTEN_ADDR) {
        Ok(()) => {
            println!("  ✅  DNS integration installed!");
            println!("      All *.test domains will resolve to 127.0.2.1");
            println!("      when DevBind is running (devbind start).");
            println!();
            println!("  Next steps:");
            println!("    1. devbind trust     # Install the SSL certificate");
            println!("    2. devbind start     # Start the proxy + DNS server");
            println!("    3. devbind add myapp 3000");
        }
        Err(e) => {
            error!("Failed to install DNS integration: {}", e);
        }
    }
    Ok(())
}

pub fn handle_uninstall() -> Result<()> {
    info!("Removing DNS integration...");
    match devbind_core::setup::uninstall_dns() {
        Ok(()) => {
            println!("  ✅  DNS integration removed.");
            println!("      .test domains will no longer auto-resolve.");
        }
        Err(e) => {
            error!("Failed to uninstall DNS integration: {}", e);
        }
    }
    Ok(())
}
