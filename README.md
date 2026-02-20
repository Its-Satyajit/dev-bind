# DevBind

DevBind is a high-performance, secure local development reverse proxy implemented in Rust utilizing the Dioxus framework. The application facilitates the mapping of custom `.local` domains to local development ports, providing automated SSL/TLS termination through an integrated Root Certificate Authority (CA) management system.

## Features

- **Automated Domain Mapping**: Facilitates the mapping of arbitrary `.local` domains to specified local ports.
- **Integrated SSL/TLS Termination**: Provides automated certificate generation and signing via a machine-local Root CA.
- **Centralized CA Management**: Includes mechanisms for the secure installation and trust management of the Root CA across various system certificate stores and browser environments (including Chromium and Firefox derivatives).
- **Dual Interface System**: Offers both a graphical user interface (GUI) developed with Dioxus and a comprehensive command-line interface (CLI) for system administration.
- **Cross-Distribution Compatibility**: Engineered for compatibility with major Linux distributions, including Arch, Fedora, Debian, and Ubuntu, with explicit support for containerized browser environments such as Flatpak and Snap.
- **State-Agnostic Privilege Escalation**: Utilizes `pkexec` and `sudo` for administrative operations (such as hosts file modification and certificate installation) only when required by the operating system security model.

## Installation and Deployment

### System Requirements

The application requires `libnss3-tools` for browser trust management and a functional `polkit` implementation for secure privilege escalation.

**Arch Linux Configuration:**
```bash
sudo pacman -S nss
```

**Debian/Ubuntu Configuration:**
```bash
sudo apt install libnss3-tools
```

### Deployment Instructions

1. Clone the repository and execute the installation script:
```bash
./install.sh
```
2. Initialize the DevBind background proxy service:
```bash
devbind start
```
3. Execute the graphical user interface:
```bash
devbind-gui
```

## Operational Usage

### Command-Line Interface (CLI)
```bash
# Register a domain mapping (appends .local suffix automatically)
devbind add myapp 8080

# Install Root CA into system trust stores
devbind trust

# List current active mappings
devbind list

# Remove Root CA from system trust stores
devbind untrust
```

### Graphical User Interface (GUI)
The `devbind-gui` executable provides a centralized dashboard for the management of domain mappings and the administrative status of the Root CA.

## Troubleshooting

### Port Binding Failures
Technical execution of proxy services on ports `80` and `443` requires elevated administrative privileges.
- **Symptom**: `Error: Permission Denied (os error 13)`
- **Solution**: Ensure a functional `polkit` agent is running. DevBind will utilize `pkexec` to bind these ports securely. Alternatively, verify that no other service (such as Apache or Nginx) is currently occupying these ports.

### SSL Certificate Trust Issues
If your browser continues to report an insecure connection for `.local` domains:
- **Symptom**: `Your connection is not private` or `SEC_ERROR_UNKNOWN_ISSUER`.
- **Solution**:
    1. Navigate to the **SSL TRUST** tab in the GUI and execute **INSTALL TRUST**.
    2. Ensure `libnss3-tools` (or `nss` on Arch) is installed on your system.
    3. Restart your browser applications to force a reload of the NSS certificate database.
    4. For **Flatpak/Snap** browsers, manually verify if the browser is utilizing the system certificate store.

### Domain Resolution (hosts)
- **Symptom**: `DNS_PROBE_FINISHED_NXDOMAIN`
- **Solution**: Verify that your entries exist in `/etc/hosts`. DevBind automates this process, but certain VPN clients or network managers may overwrite these settings.

## TLD Enforcement
DevBind enforces the usage of the `.local` top-level domain (TLD) to ensure consistency across development environments and to mitigate potential naming conflicts with public internet domains.

## Licensing
This project is licensed under the MIT License. Refer to the [LICENSE](LICENSE) file for the full text of the license agreement.
