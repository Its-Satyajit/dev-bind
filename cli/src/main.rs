use anyhow::Result;
use clap::{Parser, Subcommand};
use devbind_core::config::{DevBindConfig, RouteConfig};
use devbind_core::hosts::HostsManager;
use devbind_core::proxy::ProxyServer;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "devbind")]
#[command(about = "Local Dev SSL Reverse Proxy CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new domain to port mapping
    Add {
        /// The domain name (e.g., app.dev.local)
        domain: String,
        /// The local port your service listens on (e.g., 3000)
        port: u16,
    },
    /// List all configured mappings
    List,
    /// Start the reverse proxy (stub for now)
    Start,
    /// Install DevBind Root CA into system and browser trust stores
    Trust,
    /// Uninstall DevBind Root CA from system and browser trust stores
    Untrust,
}

fn get_config_path() -> PathBuf {
    let mut path = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        PathBuf::from(format!("/home/{}/.config", sudo_user))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
    };
    path.push("devbind");
    path.push("config.toml");
    path
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config_path = get_config_path();

    match &cli.command {
        Commands::Add { domain, port } => {
            let mut domain = domain.clone();
            if !domain.ends_with(".local") {
                domain.push_str(".local");
                info!("Automatically appended .local to domain: {}", domain);
            }

            let mut config = DevBindConfig::load(&config_path)?;

            // Allow updating existing port if it already exists
            if let Some(route) = config.routes.iter_mut().find(|r| r.domain == domain) {
                route.port = *port;
                info!("Updated {} to port {}", domain, port);
            } else {
                config.routes.push(RouteConfig {
                    domain: domain.clone(),
                    port: *port,
                });
                info!("Added {} to port {}", domain, port);
            }

            // Sync with hosts file
            let hosts_path = PathBuf::from("/etc/hosts");
            let manager = HostsManager::new(&hosts_path);
            let domains: Vec<String> = config.routes.iter().map(|r| r.domain.clone()).collect();

            if let Err(e) = manager.update_routes(&domains) {
                warn!(
                    "Failed to update /etc/hosts (try running with sudo?): {}",
                    e
                );
            } else {
                info!("Successfully updated /etc/hosts");
            }

            config.save(&config_path)?;
        }
        Commands::List => {
            let config = DevBindConfig::load(&config_path)?;
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
                    println!("{:<25} | {:<8}", route.domain, route.port);
                }
            }
        }
        Commands::Start => {
            let config = DevBindConfig::load(&config_path)?;
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
        }
        Commands::Trust => {
            let mut config_dir = config_path.clone();
            config_dir.pop();

            info!("Initiating Root CA trust installation...");
            if let Err(e) = devbind_core::trust::install_root_ca(&config_dir) {
                error!("Failed to install trust: {}", e);
            }
        }
        Commands::Untrust => {
            info!("Initiating Root CA trust removal...");
            if let Err(e) = devbind_core::trust::uninstall_root_ca() {
                error!("Failed to uninstall trust: {}", e);
            }
        }
    }

    Ok(())
}
