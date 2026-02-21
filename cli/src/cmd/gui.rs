use anyhow::Result;
use std::process::Command;
use tracing::error;

pub fn handle_gui() -> Result<()> {
    match Command::new("devbind-gui").spawn() {
        Ok(_) => {
            // Spawned successfully, CLI can exit.
        }
        Err(e) => {
            error!("Failed to launch DevBind GUI: {}", e);
            println!("  [ERROR] Could not start devbind-gui. Is it installed?");
        }
    }
    Ok(())
}
