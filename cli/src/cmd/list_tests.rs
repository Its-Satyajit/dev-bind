use super::*;
use devbind_core::config::{DevBindConfig, RouteConfig};
use tempfile::NamedTempFile;

#[test]
fn test_handle_list_empty_config_does_not_error() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    std::fs::remove_file(&config_path).unwrap();

    // An empty config should just print "(no routes configured)"
    // and not err. We don't verify stdout in this simple test, just that it doesn't panic/err.
    let result = handle_list(&config_path);
    assert!(result.is_ok());
}

#[test]
fn test_handle_list_with_routes_does_not_error() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    let mut config = DevBindConfig::default();
    config.routes.push(RouteConfig {
        domain: "app.test".to_string(),
        port: 3000,
        ephemeral: false,
    });
    config.save(&config_path).unwrap();

    // Again, we just ensure it parses the config and prints without panicking/erroring out
    let result = handle_list(&config_path);
    assert!(result.is_ok());
}
