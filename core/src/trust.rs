use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

pub fn install_root_ca(config_dir: &Path) -> Result<()> {
    let cert_path = config_dir.join("certs").join("devbind-rootCA.crt");

    if !cert_path.exists() {
        return Err(anyhow::anyhow!(
            "Root CA certificate not found at {:?}. Please start the proxy first to generate it.",
            cert_path
        ));
    }

    let script_content = format!(
        r#"#!/bin/bash
set -e
echo "Installing DevBind CA to system certificates..."
if command -v update-ca-trust &> /dev/null; then
    # Fedora/Arch/RHEL
    mkdir -p /etc/ca-certificates/trust-source/anchors/
    cp "{cert_path}" /etc/ca-certificates/trust-source/anchors/devbind.crt
    update-ca-trust
elif command -v update-ca-certificates &> /dev/null; then
    # Debian/Ubuntu
    mkdir -p /usr/local/share/ca-certificates/
    cp "{cert_path}" /usr/local/share/ca-certificates/devbind.crt
    update-ca-certificates
else
    echo "Warning: Could not find system CA update tool. Browsers may still work if certutil succeeds."
fi

echo "Installing DevBind CA into loaded NSS databases (Chrome/Firefox/Brave/Zen)..."
if command -v certutil &> /dev/null; then
    # Safely find cert9.db and cert8.db across all known browser profiles (Chrome, Brave, Firefox, Librewolf, etc.)
    # Chromium-based browsers use ~/.pki/nssdb. Flatpaks use ~/.var/app. Snaps use ~/snap.
    find /home/*/.mozilla /home/*/.pki/nssdb /home/*/.zen /home/*/.waterfox /home/*/.librewolf /home/*/.var/app /home/*/snap -maxdepth 6 -type f \( -name "cert9.db" -o -name "cert8.db" \) 2>/dev/null | while read certDB; do
        certdir=$(dirname "${{certDB}}")
        echo "Injecting into ${{certdir}}..."
        certutil -A -n "DevBind Root CA" -t "TCu,Cu,Tu" -i "{cert_path}" -d sql:"${{certdir}}" || true
    done
else
    echo "Warning: 'certutil' is not installed. Browser specific DBs skipped. Please install 'libnss3-tools' for automatic browser trusting."
fi
echo "Trust installation complete!"
"#,
        cert_path = cert_path.display()
    );

    let mut temp_script = tempfile::NamedTempFile::new()?;
    temp_script.write_all(script_content.as_bytes())?;

    let temp_path = temp_script.into_temp_path();
    let temp_path_str = temp_path.to_str().unwrap();

    // Make the script executable
    Command::new("chmod")
        .arg("+x")
        .arg(temp_path_str)
        .status()?;

    // Choose pkexec if in a GUI session and available, otherwise fallback to sudo
    let use_pkexec = std::env::var("DISPLAY").is_ok()
        && Command::new("which")
            .arg("pkexec")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    let status = if use_pkexec {
        info!("Requesting elevated privileges via pkexec...");
        Command::new("pkexec")
            .arg(temp_path_str)
            .status()
            .context("Failed to run pkexec")?
    } else {
        info!("Requesting elevated privileges via sudo...");
        Command::new("sudo")
            .arg(temp_path_str)
            .status()
            .context("Failed to run sudo")?
    };

    if status.success() {
        info!("Root CA successfully installed to system trust store.");
        Ok(())
    } else {
        error!("Failed to install Root CA into system trust store.");
        Err(anyhow::anyhow!(
            "Privilege escalation failed or script error. Status: {}",
            status
        ))
    }
}

pub fn uninstall_root_ca() -> Result<()> {
    let script_content = r#"#!/bin/bash
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
    find /home/*/.mozilla /home/*/.pki/nssdb /home/*/.zen /home/*/.waterfox /home/*/.librewolf /home/*/.var/app /home/*/snap -maxdepth 6 -type f \( -name "cert9.db" -o -name "cert8.db" \) 2>/dev/null | while read certDB; do
        certdir=$(dirname "${certDB}")
        echo "Removing from ${certdir}..."
        certutil -D -n "DevBind Root CA" -d sql:"${certdir}" || true
    done
else
    echo "Warning: 'certutil' is not installed. Browser specific DBs skipped."
fi
echo "DevBind CA removal complete!"
"#;

    let mut temp_script = tempfile::NamedTempFile::new()?;
    temp_script.write_all(script_content.as_bytes())?;

    let temp_path = temp_script.into_temp_path();
    let temp_path_str = temp_path.to_str().unwrap();

    // Make the script executable
    Command::new("chmod")
        .arg("+x")
        .arg(temp_path_str)
        .status()?;

    // Choose pkexec if in a GUI session and available, otherwise fallback to sudo
    let use_pkexec = std::env::var("DISPLAY").is_ok()
        && Command::new("which")
            .arg("pkexec")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    let status = if use_pkexec {
        info!("Requesting elevated privileges via pkexec for removal...");
        Command::new("pkexec")
            .arg(temp_path_str)
            .status()
            .context("Failed to run pkexec")?
    } else {
        info!("Requesting elevated privileges via sudo for removal...");
        Command::new("sudo")
            .arg(temp_path_str)
            .status()
            .context("Failed to run sudo")?
    };

    if status.success() {
        info!("Root CA successfully removed from system trust store.");
        Ok(())
    } else {
        error!("Failed to remove Root CA from system trust store.");
        Err(anyhow::anyhow!(
            "Privilege escalation failed or script error. Status: {}",
            status
        ))
    }
}
