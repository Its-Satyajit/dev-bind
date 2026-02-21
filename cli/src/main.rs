use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cmd;

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
    /// When no command is given, DevBind inspects the current directory and
    /// automatically detects the correct dev-server command.
    ///
    /// Examples:
    ///   devbind run myapp                              # auto-detect
    ///   devbind run myapp next dev                     # override
    ///   devbind run api python manage.py runserver 0.0.0.0:$PORT
    ///   devbind run blog rails server -p $PORT -b 0.0.0.0
    Run {
        /// Short name mapped to <name>.test (e.g. "myapp" → myapp.test)
        name: String,
        /// Command and arguments to execute. Omit to auto-detect from the
        /// current directory (package.json, pyproject.toml, config files…).
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
            cmd::add::handle_add(domain.clone(), *port, &config_path)?
        }
        Commands::List => cmd::list::handle_list(&config_path)?,
        Commands::Start => cmd::start::handle_start(&config_path).await?,
        Commands::Trust => cmd::trust::handle_trust(config_path)?,
        Commands::Untrust => cmd::trust::handle_untrust()?,
        Commands::Install => cmd::install::handle_install()?,
        Commands::Uninstall => cmd::install::handle_uninstall()?,
        Commands::Run { name, command } => {
            cmd::run::handle_run(name, command, &config_path).await?
        }
    }

    Ok(())
}
