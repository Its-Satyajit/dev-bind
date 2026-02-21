use anyhow::Result;
use devbind_core::config::DevBindConfig;
use devbind_core::runner::{validate_command, EphemeralSession};
use std::path::PathBuf;
use tracing::{error, warn};

pub async fn handle_run(name: &str, command: &[String], config_path: &PathBuf) -> Result<()> {
    // If the user didn't supply a command, try to auto-detect it.
    let command: Vec<String> = if command.is_empty() {
        let cwd = std::env::current_dir()?;
        println!(
            "  [?]  No command given — detecting framework in {}…",
            cwd.display()
        );
        match devbind_core::detect::detect_command(&cwd) {
            Some(cmd) => {
                println!("  [OK]  Detected: {}\n", cmd.join(" "));
                cmd
            }
            None => {
                anyhow::bail!(
                    "Could not auto-detect a dev command for this project.\n\
                     Please specify one explicitly:\n\
                     \n\
                       devbind run {} <command...>\n\
                     \n\
                     Tip: run 'devbind run --help' for examples.",
                    name
                );
            }
        }
    } else {
        command.to_vec()
    };

    validate_command(&command)?;

    // Load existing config
    let mut config = DevBindConfig::load(config_path)?;

    // Allocate free port + normalize domain
    let session = EphemeralSession::new(name)?;

    // Register ephemeral route in config.toml so the proxy can route it
    config.add_route(session.domain.clone(), session.port);
    if let Err(e) = config.save(config_path) {
        warn!("Could not save ephemeral route to config: {}", e);
    }

    println!(
        "\n  [LINK]  {} → http://127.0.0.1:{} (proxied at https://{})\n",
        session.domain, session.port, session.domain
    );
    println!("  [EXEC]   Launching: {}\n", command.join(" "));

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

    let mut cmd_builder = tokio::process::Command::new(&command[0]);
    cmd_builder.args(&final_args).envs(env_vars);

    // Drop privileges if running under sudo
    if let (Ok(uid_str), Ok(gid_str)) = (std::env::var("SUDO_UID"), std::env::var("SUDO_GID")) {
        if let (Ok(uid), Ok(gid)) = (uid_str.parse::<u32>(), gid_str.parse::<u32>()) {
            #[allow(unused_imports)]
            use std::os::unix::process::CommandExt;
            // Provide a pre_exec closure to change UID and GID in the child process just before exec
            unsafe {
                cmd_builder.pre_exec(move || {
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
    let mut child = cmd_builder.spawn().map_err(|e| {
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
    println!("\n  [CLEAN]  Cleaning up {}...", session.domain);

    // Remove ephemeral route from config
    let mut config = DevBindConfig::load(config_path).unwrap_or_default();
    config.remove_route(&session.domain);
    if let Err(e) = config.save(config_path) {
        warn!("Failed to remove ephemeral route from config: {}", e);
    }

    println!("  [OK]  {} unregistered.", session.domain);

    if let Some(status) = exit_status {
        if let Some(code) = status.code() {
            std::process::exit(code);
        }
    }

    Ok(())
}
