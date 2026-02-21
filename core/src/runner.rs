use anyhow::{bail, Context, Result};
use std::net::TcpListener;

/// Bind to `127.0.0.1:0` and let the OS allocate a free ephemeral port.
/// The listener is immediately dropped so the port can be reused by the child.
pub fn find_free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .context("Failed to bind to a random port while searching for a free one")?;
    let port = listener
        .local_addr()
        .context("Failed to read local address of temporary listener")?
        .port();
    // listener drops here, freeing the port
    Ok(port)
}

/// An ephemeral DevBind session created by `devbind run`.
///
/// Normalises the domain name and allocates a free port.
/// DNS resolution is handled by the embedded DNS server — no /etc/hosts needed.
pub struct EphemeralSession {
    pub domain: String,
    pub port: u16,
}

impl EphemeralSession {
    /// Create a new session: normalise the domain name and allocate a free port.
    pub fn new(name: &str) -> Result<Self> {
        let domain = if name.ends_with(".test") {
            name.to_string()
        } else {
            format!("{}.test", name)
        };

        let port = find_free_port()?;

        Ok(Self { domain, port })
    }

    /// Environment variables to inject into the child process.
    pub fn env_vars(&self) -> Vec<(String, String)> {
        vec![
            ("PORT".to_string(), self.port.to_string()),
            ("HOST".to_string(), "0.0.0.0".to_string()),
            (
                "DEVBIND_DOMAIN".to_string(),
                format!("https://{}", self.domain),
            ),
        ]
    }
}

/// Validate that the first element of `command` is present.
pub fn validate_command(command: &[String]) -> Result<()> {
    if command.is_empty() {
        bail!("No command specified. Usage: devbind run <name> <command...>");
    }
    Ok(())
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
