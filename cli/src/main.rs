use anyhow::Result;
use clap::{Parser, Subcommand};
use devbind_core::config::{DevBindConfig, RouteConfig};
use devbind_core::proxy::ProxyServer;
use devbind_core::runner::{validate_command, EphemeralSession};
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
        /// The domain name (e.g., myapp.test)
        domain: String,
        /// The local port your service listens on (e.g., 3000)
        port: u16,
    },
    /// List all configured mappings
    List,
    /// Start the reverse proxy (with embedded DNS server)
    Start,
    /// Install DevBind Root CA into system and browser trust stores
    Trust,
    /// Uninstall DevBind Root CA from system and browser trust stores
    Untrust,
    /// Install DNS integration (systemd-resolved drop-in for .test domains)
    Install,
    /// Uninstall DNS integration (remove systemd-resolved drop-in)
    Uninstall,
    /// Run an app on an auto-assigned free port mapped to <name>.test
    ///
    /// Examples:
    ///   devbind run myapp next dev
    ///   devbind run api python manage.py runserver 0.0.0.0:$PORT
    ///   devbind run blog ruby bin/rails server -p $PORT
    Run {
        /// Short name mapped to <name>.test (e.g. "myapp" → myapp.test)
        name: String,
        /// Command and arguments to execute (receives PORT env var)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
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
            if !domain.ends_with(".test") {
                domain.push_str(".test");
                info!("Automatically appended .test to domain: {}", domain);
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

            config.save(&config_path)?;
            println!("  ✅  {} → localhost:{}", domain, port);
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
        Commands::Install => {
            info!("Installing DNS integration for .test domains...");
            match devbind_core::setup::install_dns(devbind_core::dns::DNS_LISTEN_ADDR) {
                Ok(()) => {
                    println!("  ✅  DNS integration installed!");
                    println!("      All *.test domains will resolve to 127.0.0.1");
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
        }
        Commands::Uninstall => {
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
        }
        Commands::Run { name, command } => {
            validate_command(command)?;

            // Load existing config
            let mut config = DevBindConfig::load(&config_path)?;

            // Allocate free port + normalize domain
            let session = EphemeralSession::new(name)?;

            // Register ephemeral route in config.toml so the proxy can route it
            config.add_route(session.domain.clone(), session.port);
            if let Err(e) = config.save(&config_path) {
                warn!("Could not save ephemeral route to config: {}", e);
            }

            println!(
                "\n  🔗  {} → http://127.0.0.1:{} (proxied at https://{})\n",
                session.domain, session.port, session.domain
            );
            println!("  ▶   Launching: {}\n", command.join(" "));

            // Build env vars: inherit everything, then override/add ours
            let env_vars = session.env_vars();

            // Substitute placeholders in arguments so things like `php -S 0.0.0.0:$PORT` work natively
            let port_str = session.port.to_string();
            let host_str = "0.0.0.0".to_string();
            let domain_str = format!("https://{}", session.domain);

            let final_args: Vec<String> = command[1..]
                .iter()
                .map(|arg| {
                    arg.replace("$PORT", &port_str)
                        .replace("${PORT}", &port_str)
                        .replace("$HOST", &host_str)
                        .replace("${HOST}", &host_str)
                        .replace("$DEVBIND_DOMAIN", &domain_str)
                        .replace("${DEVBIND_DOMAIN}", &domain_str)
                })
                .collect();

            let mut cmd = tokio::process::Command::new(&command[0]);
            cmd.args(&final_args).envs(env_vars);

            // Drop privileges if running under sudo
            if let (Ok(uid_str), Ok(gid_str)) =
                (std::env::var("SUDO_UID"), std::env::var("SUDO_GID"))
            {
                if let (Ok(uid), Ok(gid)) = (uid_str.parse::<u32>(), gid_str.parse::<u32>()) {
                    use std::os::unix::process::CommandExt;
                    // Provide a pre_exec closure to change UID and GID in the child process just before exec
                    unsafe {
                        cmd.pre_exec(move || {
                            // Drop groups first
                            libc::setgroups(0, std::ptr::null());
                            libc::setgid(gid);
                            libc::setuid(uid);
                            Ok(())
                        });
                    }
                }
            }

            // Spawn child process
            let mut child = cmd.spawn().map_err(|e| {
                anyhow::anyhow!(
                    "Failed to launch '{}': {}. Is the command installed?",
                    command[0],
                    e
                )
            })?;

            // Wait for child exit or Ctrl-C
            let exit_status = tokio::select! {
                status = child.wait() => {
                    match status {
                        Ok(s) => Some(s),
                        Err(e) => {
                            error!("Error waiting for child process: {}", e);
                            None
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\n  ⏹  Stopping {} (Ctrl-C received)...", command[0]);
                    let _ = child.kill().await;
                    None
                }
            };

            // Always clean up, regardless of how the process ended
            println!("\n  🧹  Cleaning up {}...", session.domain);

            // Remove ephemeral route from config
            let mut config = DevBindConfig::load(&config_path).unwrap_or_default();
            config.remove_route(&session.domain);
            if let Err(e) = config.save(&config_path) {
                warn!("Failed to remove ephemeral route from config: {}", e);
            }

            println!("  ✅  {} unregistered.", session.domain);

            if let Some(status) = exit_status {
                if let Some(code) = status.code() {
                    std::process::exit(code);
                }
            }
        }
    }

    Ok(())
}
