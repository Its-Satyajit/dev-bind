use anyhow::Result;
use devbind_core::config::DevBindConfig;
use std::path::PathBuf;

pub fn handle_list(config_path: &PathBuf) -> Result<()> {
    let config = DevBindConfig::load(config_path)?;
    println!(
        "DevBind Configuration (Proxy Port: {}):",
        config.proxy.port_https
    );
    println!("{:-<40}", "");
    println!("{:<25} | {:<8}", "Domain", "Port");
    println!("{:-<40}", "");
    if config.routes.is_empty() {
        println!("  (no routes configured)");
    } else {
        for route in &config.routes {
            let tag = if route.ephemeral { " [EPHEMERAL]" } else { "" };
            println!("{:<25} | {:<8}{}", route.domain, route.port, tag);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "list_tests.rs"]
mod tests;
