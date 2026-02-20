use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

/// Shell script for installing the DevBind Root CA into system and browser trust stores.
///
/// The certificate path is passed via the `DEVBIND_CERT_PATH` environment variable — it is
/// NOT interpolated into the script string directly.  This prevents shell injection if a
/// future code-path allows the config directory to be user-controlled.
const INSTALL_SCRIPT: &str = r#"#!/bin/bash
set -e

if [ -z "$DEVBIND_CERT_PATH" ]; then
    echo "Error: DEVBIND_CERT_PATH is not set." >&2
    exit 1
fi

# Validate the cert path is an absolute path to an existing file
if [ ! -f "$DEVBIND_CERT_PATH" ]; then
    echo "Error: Certificate not found at '$DEVBIND_CERT_PATH'" >&2
    exit 1
fi

echo "Installing DevBind CA to system certificates..."
if command -v update-ca-trust &> /dev/null; then
    # Fedora/Arch/RHEL
    mkdir -p /etc/ca-certificates/trust-source/anchors/
    cp -- "$DEVBIND_CERT_PATH" /etc/ca-certificates/trust-source/anchors/devbind.crt
    update-ca-trust
elif command -v update-ca-certificates &> /dev/null; then
    # Debian/Ubuntu
    mkdir -p /usr/local/share/ca-certificates/
    cp -- "$DEVBIND_CERT_PATH" /usr/local/share/ca-certificates/devbind.crt
    update-ca-certificates
else
    echo "Warning: Could not find system CA update tool. Browsers may still work if certutil succeeds."
fi

echo "Installing DevBind CA into loaded NSS databases (Chrome/Firefox/Brave/Zen)..."
if command -v certutil &> /dev/null; then
    find /home/*/.mozilla /home/*/.pki/nssdb /home/*/.zen /home/*/.waterfox \
         /home/*/.librewolf /home/*/.var/app /home/*/snap \
         -maxdepth 6 -type f \( -name "cert9.db" -o -name "cert8.db" \) 2>/dev/null \
    | while IFS= read -r certDB; do
        certdir=$(dirname -- "$certDB")
        echo "Injecting into ${certdir}..."
        certutil -A -n "DevBind Root CA" -t "TCu,Cu,Tu" -i "$DEVBIND_CERT_PATH" -d sql:"${certdir}" || true
    done
else
    echo "Warning: 'certutil' is not installed. Browser DBs skipped. Install 'libnss3-tools' for automatic browser trusting."
fi
echo "Trust installation complete!"
"#;

/// Shell script for removing the DevBind Root CA from system and browser trust stores.
const UNINSTALL_SCRIPT: &str = r#"#!/bin/bash
set -e
echo "Removing DevBind CA from system certificates..."
if command -v update-ca-trust &> /dev/null; then
    # Fedora/Arch/RHEL
    rm -f /etc/ca-certificates/trust-source/anchors/devbind.crt
    update-ca-trust
elif command -v update-ca-certificates &> /dev/null; then
    # Debian/Ubuntu
    rm -f /usr/local/share/ca-certificates/devbind.crt
    update-ca-certificates
else
    echo "Warning: Could not find system CA update tool."
fi

echo "Removing DevBind CA from NSS databases (Chrome/Firefox/Brave/Zen)..."
if command -v certutil &> /dev/null; then
    find /home/*/.mozilla /home/*/.pki/nssdb /home/*/.zen /home/*/.waterfox \
         /home/*/.librewolf /home/*/.var/app /home/*/snap \
         -maxdepth 6 -type f \( -name "cert9.db" -o -name "cert8.db" \) 2>/dev/null \
    | while IFS= read -r certDB; do
        certdir=$(dirname -- "$certDB")
        echo "Removing from ${certdir}..."
        certutil -D -n "DevBind Root CA" -d sql:"${certdir}" || true
    done
else
    echo "Warning: 'certutil' is not installed. Browser DBs skipped."
fi
echo "DevBind CA removal complete!"
"#;

pub fn install_root_ca(config_dir: &Path) -> Result<()> {
    let cert_path = config_dir.join("certs").join("devbind-rootCA.crt");

    if !cert_path.exists() {
        return Err(anyhow::anyhow!(
            "Root CA certificate not found at {:?}. Please start the proxy first to generate it.",
            cert_path
        ));
    }

    run_elevated_script(INSTALL_SCRIPT, Some(&cert_path), "install")
}

pub fn uninstall_root_ca() -> Result<()> {
    run_elevated_script(UNINSTALL_SCRIPT, None, "uninstall")
}

/// Writes `script` to a temporary file and runs it with elevated privileges.
///
/// If `cert_path` is provided it is passed via the `DEVBIND_CERT_PATH` environment variable,
/// not interpolated into the script text, preventing shell injection attacks.
fn run_elevated_script(script: &str, cert_path: Option<&Path>, action: &str) -> Result<()> {
    let mut temp_script = tempfile::NamedTempFile::new()
        .with_context(|| format!("Failed to create temp file for {} script", action))?;
    temp_script
        .write_all(script.as_bytes())
        .with_context(|| format!("Failed to write {} script", action))?;

    let temp_path = temp_script.into_temp_path();
    let temp_path_str = temp_path
        .to_str()
        .context("Temp file path is not valid UTF-8")?;

    // Make the script executable
    Command::new("chmod")
        .arg("+x")
        .arg(temp_path_str)
        .status()
        .context("Failed to chmod temp script")?;

    // Detect whether to use pkexec (GUI) or sudo (CLI/headless).
    let use_pkexec = std::env::var("DISPLAY").is_ok()
        && Command::new("which")
            .arg("pkexec")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    let mut cmd = if use_pkexec {
        info!(
            "Requesting elevated privileges via pkexec for {}...",
            action
        );
        Command::new("pkexec")
    } else {
        info!("Requesting elevated privileges via sudo for {}...", action);
        Command::new("sudo")
    };

    cmd.arg(temp_path_str);

    // Pass the cert path via environment variable — NOT inline in the script.
    if let Some(path) = cert_path {
        cmd.env(
            "DEVBIND_CERT_PATH",
            path.to_str().context("Cert path is not valid UTF-8")?,
        );
    }

    let status = cmd
        .status()
        .with_context(|| format!("Failed to run elevated {} script", action))?;

    if status.success() {
        info!("Root CA {} succeeded.", action);
        Ok(())
    } else {
        error!("Root CA {} failed.", action);
        Err(anyhow::anyhow!(
            "Privilege escalation failed or script error during {}. Status: {}",
            action,
            status
        ))
    }
}
