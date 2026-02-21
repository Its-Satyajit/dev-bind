use super::*;
use devbind_core::config::DevBindConfig;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_handle_add_creates_new_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    fs::remove_file(&config_path).unwrap(); // Ensure it doesn't exist

    handle_add("newapp".to_string(), 3000, &config_path).unwrap();

    let config = DevBindConfig::load(&config_path).unwrap();
    assert_eq!(config.routes.len(), 1);
    assert_eq!(config.routes[0].domain, "newapp.test");
    assert_eq!(config.routes[0].port, 3000);
}

#[test]
fn test_handle_add_appends_test_suffix() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    fs::remove_file(&config_path).unwrap(); // Ensure it doesn't exist

    // The handler should append .test since it's missing
    handle_add("my-api".to_string(), 8080, &config_path).unwrap();

    let config = DevBindConfig::load(&config_path).unwrap();
    assert_eq!(config.routes[0].domain, "my-api.test");
    assert_eq!(config.routes[0].port, 8080);
}

#[test]
fn test_handle_add_does_not_double_suffix() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    fs::remove_file(&config_path).unwrap();

    // The handler should NOT append .test since it's already there
    handle_add("already-has.test".to_string(), 4000, &config_path).unwrap();

    let config = DevBindConfig::load(&config_path).unwrap();
    assert_eq!(config.routes[0].domain, "already-has.test");
    assert_eq!(config.routes[0].port, 4000);
}

#[test]
fn test_handle_add_updates_existing_route() {
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path().to_path_buf();
    fs::remove_file(&config_path).unwrap();

    // First addition
    handle_add("frontend".to_string(), 3000, &config_path).unwrap();
    // Second addition (update port)
    handle_add("frontend".to_string(), 5173, &config_path).unwrap();

    let config = DevBindConfig::load(&config_path).unwrap();
    assert_eq!(config.routes.len(), 1); // Should still be 1 route
    assert_eq!(config.routes[0].domain, "frontend.test");
    assert_eq!(config.routes[0].port, 5173); // Port should be updated
}
