/// Check whether the systemd user service is active.
pub fn is_service_active() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "devbind"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check whether the systemd user service is installed (unit file exists).
pub fn is_service_installed() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".config/systemd/user/devbind.service").exists())
        .unwrap_or(false)
}

/// Write the systemd user service file and enable + start it.
pub fn install_service(devbind_bin: &str) -> Result<(), String> {
    let service_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".config/systemd/user");
    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let service_content = format!(
        "[Unit]\nDescription=DevBind Local Dev SSL Reverse Proxy\nAfter=network.target\n\n\
         [Service]\nExecStart={bin} start\nRestart=on-failure\nRestartSec=5\n\n\
         [Install]\nWantedBy=default.target\n",
        bin = devbind_bin
    );
    std::fs::write(service_dir.join("devbind.service"), service_content)
        .map_err(|e| e.to_string())?;

    for args in &[
        vec!["--user", "daemon-reload"],
        vec!["--user", "enable", "devbind"],
        vec!["--user", "start", "devbind"],
    ] {
        let status = std::process::Command::new("systemctl")
            .args(args)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("systemctl {} failed", args.join(" ")));
        }
    }
    Ok(())
}

/// Stop, disable and remove the systemd user service.
pub fn uninstall_service() -> Result<(), String> {
    for args in &[
        vec!["--user", "stop", "devbind"],
        vec!["--user", "disable", "devbind"],
    ] {
        let _ = std::process::Command::new("systemctl").args(args).status();
    }
    if let Some(path) = dirs::home_dir()
        .map(|h| h.join(".config/systemd/user/devbind.service"))
        .filter(|p| p.exists())
    {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    Ok(())
}
