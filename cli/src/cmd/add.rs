use anyhow::Result;
use devbind_core::config::{DevBindConfig, RouteConfig};
use std::path::PathBuf;
use tracing::info;

pub fn handle_add(mut domain: String, port: u16, config_path: &PathBuf) -> Result<()> {
    if !domain.ends_with(".test") {
        domain.push_str(".test");
        info!("Automatically appended .test to domain: {}", domain);
    }

    let mut config = DevBindConfig::load(config_path)?;

    if let Some(route) = config.routes.iter_mut().find(|r| r.domain == domain) {
        route.port = port;
        info!("Updated {} to port {}", domain, port);
    } else {
        config.routes.push(RouteConfig {
            domain: domain.clone(),
            port,
        });
        info!("Added {} to port {}", domain, port);
    }

    config.save(config_path)?;
    println!("  ✅  {} → localhost:{}", domain, port);
    Ok(())
}
