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

    // ── find_free_port ───────────────────────────────────────────────────────

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
    fn test_find_free_port_two_calls_succeed() {
        // Both allocations must succeed and return non-zero ports.
        let p1 = find_free_port().unwrap();
        let p2 = find_free_port().unwrap();
        assert!(p1 > 0);
        assert!(p2 > 0);
    }

    // ── validate_command ─────────────────────────────────────────────────────

    #[test]
    fn test_validate_command_ok_with_non_empty_slice() {
        let cmd = vec!["node".to_string(), "server.js".to_string()];
        assert!(
            validate_command(&cmd).is_ok(),
            "non-empty command must be valid"
        );
    }

    #[test]
    fn test_validate_command_errors_on_empty_slice() {
        let empty: Vec<String> = vec![];
        let result = validate_command(&empty);
        assert!(result.is_err(), "empty command list must return an error");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("No command specified"),
            "error message must mention missing command, got: {}",
            msg
        );
    }

    #[test]
    fn test_validate_command_ok_with_single_element() {
        // Edge-case: just the binary name, no arguments
        let cmd = vec!["npm".to_string()];
        assert!(validate_command(&cmd).is_ok());
    }

    // ── EphemeralSession domain normalisation ────────────────────────────────

    #[test]
    fn test_domain_normalisation_appends_test_suffix() {
        let session = EphemeralSession::new("myapp").expect("should create session");
        assert_eq!(session.domain, "myapp.test");
    }

    #[test]
    fn test_domain_already_has_test_suffix_not_doubled() {
        let session = EphemeralSession::new("myapp.test").expect("should create session");
        assert_eq!(
            session.domain, "myapp.test",
            ".test suffix must not be appended twice"
        );
    }

    #[test]
    fn test_domain_with_hyphen_normalises_correctly() {
        let session = EphemeralSession::new("my-cool-app").unwrap();
        assert_eq!(session.domain, "my-cool-app.test");
    }

    #[test]
    fn test_domain_with_numbers_normalises_correctly() {
        let session = EphemeralSession::new("app123").unwrap();
        assert_eq!(session.domain, "app123.test");
    }

    // ── EphemeralSession port ────────────────────────────────────────────────

    #[test]
    fn test_ephemeral_session_allocates_nonzero_port() {
        let session = EphemeralSession::new("porttest").unwrap();
        assert!(session.port > 0, "allocated port must be > 0");
    }

    // ── EphemeralSession env_vars ────────────────────────────────────────────

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

    #[test]
    fn test_env_vars_port_matches_session_port() {
        let session = EphemeralSession::new("portmatch").unwrap();
        let expected_port = session.port;
        let vars: std::collections::HashMap<_, _> = session.env_vars().into_iter().collect();
        let env_port: u16 = vars["PORT"].parse().expect("PORT must be numeric");
        assert_eq!(
            env_port, expected_port,
            "PORT env var must equal the session's allocated port"
        );
    }

    #[test]
    fn test_env_vars_devbind_domain_contains_domain() {
        let session = EphemeralSession::new("domaincheck").unwrap();
        let domain = session.domain.clone();
        let vars: std::collections::HashMap<_, _> = session.env_vars().into_iter().collect();
        assert!(
            vars["DEVBIND_DOMAIN"].contains(&domain),
            "DEVBIND_DOMAIN '{}' must contain the session domain '{}'",
            vars["DEVBIND_DOMAIN"],
            domain
        );
        assert_eq!(vars["DEVBIND_DOMAIN"], format!("https://{}", domain));
    }

    #[test]
    fn test_env_vars_has_exactly_three_entries() {
        let session = EphemeralSession::new("counttest").unwrap();
        let vars = session.env_vars();
        assert_eq!(
            vars.len(),
            3,
            "env_vars must return exactly PORT, HOST, DEVBIND_DOMAIN"
        );
    }
}
