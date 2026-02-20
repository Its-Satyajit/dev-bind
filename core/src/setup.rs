//! DNS integration setup for DevBind.
//!
//! Manages the systemd-resolved drop-in configuration that routes `.test`
//! domain queries to DevBind's embedded DNS server.

use anyhow::{Context, Result};
use std::path::Path;
use tracing::info;

const RESOLVED_DROP_IN_DIR: &str = "/etc/systemd/resolved.conf.d";
const DROP_IN_FILE: &str = "devbind.conf";

/// The content of the systemd-resolved drop-in config.
fn drop_in_content(dns_addr: &str) -> String {
    format!("[Resolve]\nDNS={}\nDomains=~test\n", dns_addr)
}

/// Full path to the drop-in config file.
fn drop_in_path() -> String {
    format!("{}/{}", RESOLVED_DROP_IN_DIR, DROP_IN_FILE)
}

/// Check whether the DNS drop-in is already installed.
pub fn is_dns_installed() -> bool {
    Path::new(&drop_in_path()).exists()
}

/// Check whether systemd-resolved is available on the system.
pub fn is_resolved_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "systemd-resolved"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Install the DNS drop-in for systemd-resolved.
///
/// This creates `/etc/systemd/resolved.conf.d/devbind.conf` and restarts
/// systemd-resolved so that `*.test` queries are forwarded to DevBind's DNS.
///
/// Requires elevated privileges (uses `pkexec` with `sudo` fallback).
pub fn install_dns(dns_listen_addr: &str) -> Result<()> {
    if !is_resolved_available() {
        anyhow::bail!(
            "systemd-resolved is not active. DevBind DNS integration requires systemd-resolved.\n\
             Alternatively, you can manually configure your DNS to forward .test queries to {}",
            dns_listen_addr
        );
    }

    let content = drop_in_content(dns_listen_addr);
    let target = drop_in_path();

    // Write to a temp file first, then copy with elevated privileges
    let tmp = tempfile::Builder::new()
        .prefix("devbind-dns-")
        .tempfile()
        .context("Failed to create temporary file")?;

    std::fs::write(tmp.path(), &content).context("Failed to write DNS config to temporary file")?;

    // Ensure the drop-in directory exists
    let mkdir_ok = try_elevated_command(&["mkdir", "-p", RESOLVED_DROP_IN_DIR]);
    if !mkdir_ok {
        anyhow::bail!("Failed to create directory {}", RESOLVED_DROP_IN_DIR);
    }

    // Copy the config file
    let tmp_path_str = tmp.path().to_string_lossy().to_string();
    let cp_ok = try_elevated_command(&["cp", &tmp_path_str, &target]);
    if !cp_ok {
        anyhow::bail!(
            "Failed to install DNS config to {}. Try running with sudo.",
            target
        );
    }

    // Restart systemd-resolved to pick up the new config
    let restart_ok = std::process::Command::new("sudo")
        .args(["systemctl", "restart", "systemd-resolved"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !restart_ok {
        // Non-fatal: config is installed, just needs manual restart
        info!("Note: Could not restart systemd-resolved automatically. Run: sudo systemctl restart systemd-resolved");
    }

    info!("DNS integration installed at {}", target);
    info!("All *.test domains will now resolve to 127.0.0.1 when DevBind is running");

    Ok(())
}

/// Uninstall the DNS drop-in and restart systemd-resolved.
pub fn uninstall_dns() -> Result<()> {
    let target = drop_in_path();

    if !Path::new(&target).exists() {
        info!("DNS integration is not installed (no drop-in found)");
        return Ok(());
    }

    let rm_ok = try_elevated_command(&["rm", "-f", &target]);
    if !rm_ok {
        anyhow::bail!(
            "Failed to remove DNS config at {}. Try running with sudo.",
            target
        );
    }

    // Restart systemd-resolved
    let _ = std::process::Command::new("sudo")
        .args(["systemctl", "restart", "systemd-resolved"])
        .status();

    info!("DNS integration removed. .test domains will no longer resolve.");

    Ok(())
}

/// Try to run a command with elevated privileges.
/// Tries `pkexec` first (for GUI), falls back to `sudo` (for CLI).
fn try_elevated_command(args: &[&str]) -> bool {
    // Try pkexec first
    if let Ok(status) = std::process::Command::new("pkexec").args(args).status() {
        if status.success() {
            return true;
        }
    }

    // Fallback to sudo
    std::process::Command::new("sudo")
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_in_content_format() {
        let content = drop_in_content("127.0.0.1:5453");
        assert!(content.contains("[Resolve]"));
        assert!(content.contains("DNS=127.0.0.1:5453"));
        assert!(content.contains("Domains=~test"));
    }

    #[test]
    fn test_drop_in_path() {
        assert_eq!(drop_in_path(), "/etc/systemd/resolved.conf.d/devbind.conf");
    }
}
