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
mod tests {
    use super::*;
    use std::net::TcpListener as StdListener;

    #[test]
    fn test_find_free_port_returns_valid_port() {
        let port = find_free_port().expect("should find a free port");
        assert!(port > 0, "port should be non-zero");
        // Verify we can actually bind to it (it was freed)
        let listener = StdListener::bind(format!("127.0.0.1:{}", port));
        assert!(
            listener.is_ok(),
            "returned port {} should be bindable",
            port
        );
    }

    #[test]
    fn test_find_free_port_two_calls_may_differ() {
        // Simply assert both succeed; they might or might not be the same port.
        let p1 = find_free_port().unwrap();
        let p2 = find_free_port().unwrap();
        assert!(p1 > 0);
        assert!(p2 > 0);
    }

    #[test]
    fn test_domain_normalisation() {
        // EphemeralSession normalises "myapp" → "myapp.test"
        let session = EphemeralSession::new("myapp").expect("should create session");
        assert_eq!(session.domain, "myapp.test");
    }

    #[test]
    fn test_domain_already_has_test_suffix() {
        let session = EphemeralSession::new("myapp.test").expect("should create session");
        assert_eq!(session.domain, "myapp.test");
    }

    #[test]
    fn test_env_vars_contains_required_keys() {
        let session = EphemeralSession::new("envtest").expect("should create session");
        let vars: std::collections::HashMap<_, _> = session.env_vars().into_iter().collect();
        assert!(vars.contains_key("PORT"), "PORT must be set");
        assert!(vars.contains_key("HOST"), "HOST must be set");
        assert!(
            vars.contains_key("DEVBIND_DOMAIN"),
            "DEVBIND_DOMAIN must be set"
        );
        assert_eq!(vars["HOST"], "0.0.0.0");
        assert!(vars["DEVBIND_DOMAIN"].starts_with("https://"));
        let port: u16 = vars["PORT"].parse().expect("PORT should be numeric");
        assert!(port > 0);
    }
}
