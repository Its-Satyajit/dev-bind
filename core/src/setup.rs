use anyhow::{bail, Result};
use tracing::info;

/// Check whether NetworkManager is available and active on the system.
pub fn is_networkmanager_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "NetworkManager"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check whether systemd-resolved is available and active.
pub fn is_resolved_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "systemd-resolved"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check whether the specific devbind0 NetworkManager connection exists.
pub fn is_dns_installed() -> bool {
    std::process::Command::new("nmcli")
        .args(["connection", "show", "devbind0"])
        .output()
        .map(|opt| opt.status.success())
        .unwrap_or(false)
}

/// Install the DNS integration by creating a dummy network interface
/// in NetworkManager. This binds `*.test` queries exclusively to `127.0.2.1:53`
/// without polluting global DNS resolvers (fixes AdGuard compatibility).
pub fn install_dns(dns_listen_addr: &str) -> Result<()> {
    if !is_networkmanager_available() || !is_resolved_available() {
        bail!(
            "DevBind auto-DNS integration requires both NetworkManager and systemd-resolved.\n\
             Please configure your DNS manually to forward .test queries to {}",
            dns_listen_addr
        );
    }

    // Only install if not already installed
    if is_dns_installed() {
        info!("DNS integration is already installed.");
        return Ok(());
    }

    // Create the dummy interface via nmcli
    let args = [
        "connection",
        "add",
        "type",
        "dummy",
        "ifname",
        "devbind0",
        "con-name",
        "devbind0",
        "ipv4.method",
        "manual",
        "ipv4.addresses",
        "10.254.254.254/32",
        "ipv4.ignore-auto-dns",
        "yes",
        "ipv4.dns",
        "127.0.2.1",
        "ipv4.dns-search",
        "~test",
    ];

    if !try_elevated_command("nmcli", &args) {
        bail!("Failed to create devbind0 network interface. Try running with sudo.");
    }

    info!("DNS integration installed successfully!");
    info!("NetworkManager 'devbind0' dummy interface will route *.test to 127.0.2.1");

    Ok(())
}

/// Remove the DNS integration by deleting the dummy network interface.
pub fn uninstall_dns() -> Result<()> {
    if !is_dns_installed() {
        info!("DNS integration is not installed.");
        return Ok(());
    }

    if !try_elevated_command("nmcli", &["connection", "delete", "devbind0"]) {
        bail!("Failed to remove devbind0 network interface. Try running with sudo.");
    }

    info!("DNS integration removed successfully.");
    Ok(())
}

/// Run a command with privilege elevation, trying pkexec then sudo.
fn try_elevated_command(bin: &str, args: &[&str]) -> bool {
    // Determine if we're in an interactive GUI session (pkexec is preferred)
    // For CLI, sudo is often better on headless setups, but pkexec works if polkit is present.
    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(status) = std::process::Command::new("pkexec")
            .arg(bin)
            .args(args)
            .status()
        {
            if status.success() {
                return true;
            }
        }
    }

    // Fallback to sudo
    std::process::Command::new("sudo")
        .arg(bin)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
