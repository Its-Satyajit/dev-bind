use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn tmp_config_path(dir: &TempDir) -> PathBuf {
        dir.path().join("config.toml")
    }

    // ── Load / Save ─────────────────────────────────────────────────────────

    #[test]
    fn test_load_returns_default_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let path = tmp_config_path(&dir);
        // File does not exist — should silently return defaults
        let cfg = DevBindConfig::load(&path).expect("load should succeed even if file missing");
        assert_eq!(cfg, DevBindConfig::default());
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = tmp_config_path(&dir);

        let mut original = DevBindConfig::default();
        original.routes.push(RouteConfig {
            domain: "myapp.test".to_string(),
            port: 3000,
        });
        original.routes.push(RouteConfig {
            domain: "api.test".to_string(),
            port: 8080,
        });

        original.save(&path).expect("save should succeed");
        let loaded = DevBindConfig::load(&path).expect("load should succeed");

        assert_eq!(original, loaded, "round-tripped config must be identical");
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        // Nested path that does not yet exist
        let path = dir.path().join("a").join("b").join("config.toml");

        let cfg = DevBindConfig::default();
        cfg.save(&path)
            .expect("save should create parent dirs automatically");
        assert!(path.exists(), "config file should exist after save");
    }

    #[test]
    fn test_load_errors_on_malformed_toml() {
        let dir = TempDir::new().unwrap();
        let path = tmp_config_path(&dir);
        std::fs::write(&path, b"this is NOT valid TOML !!!").unwrap();

        let result = DevBindConfig::load(&path);
        assert!(result.is_err(), "malformed TOML must produce an error");
    }

    // ── Default values ───────────────────────────────────────────────────────

    #[test]
    fn test_default_proxy_ports() {
        let cfg = DevBindConfig::default();
        assert_eq!(cfg.proxy.port_http, 80);
        assert_eq!(cfg.proxy.port_https, 443);
    }

    #[test]
    fn test_default_routes_empty() {
        let cfg = DevBindConfig::default();
        assert!(cfg.routes.is_empty(), "default config has no routes");
    }

    // ── add_route ────────────────────────────────────────────────────────────

    #[test]
    fn test_add_route_appends_new_domain() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("myapp.test".to_string(), 3000);
        assert_eq!(cfg.routes.len(), 1);
        assert_eq!(cfg.routes[0].domain, "myapp.test");
        assert_eq!(cfg.routes[0].port, 3000);
    }

    #[test]
    fn test_add_route_updates_port_for_existing_domain() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("myapp.test".to_string(), 3000);
        // Adding same domain again with different port should update in-place
        cfg.add_route("myapp.test".to_string(), 4000);

        assert_eq!(
            cfg.routes.len(),
            1,
            "duplicate domain must not create a second entry"
        );
        assert_eq!(
            cfg.routes[0].port, 4000,
            "port must be updated to new value"
        );
    }

    #[test]
    fn test_add_route_multiple_distinct_domains() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("alpha.test".to_string(), 3000);
        cfg.add_route("beta.test".to_string(), 4000);
        cfg.add_route("gamma.test".to_string(), 5000);
        assert_eq!(cfg.routes.len(), 3);
    }

    // ── remove_route ─────────────────────────────────────────────────────────

    #[test]
    fn test_remove_route_deletes_existing_domain() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("myapp.test".to_string(), 3000);
        cfg.add_route("api.test".to_string(), 8080);

        cfg.remove_route("myapp.test");

        assert_eq!(cfg.routes.len(), 1);
        assert_eq!(cfg.routes[0].domain, "api.test");
    }

    #[test]
    fn test_remove_route_is_noop_for_missing_domain() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("myapp.test".to_string(), 3000);

        // Removing a domain that was never added should be a no-op
        cfg.remove_route("nonexistent.test");

        assert_eq!(cfg.routes.len(), 1, "unrelated routes must be unaffected");
    }

    #[test]
    fn test_remove_route_on_empty_config_is_safe() {
        let mut cfg = DevBindConfig::default();
        // Must not panic
        cfg.remove_route("something.test");
        assert!(cfg.routes.is_empty());
    }

    #[test]
    fn test_remove_route_clears_last_entry() {
        let mut cfg = DevBindConfig::default();
        cfg.add_route("solo.test".to_string(), 9000);
        cfg.remove_route("solo.test");
        assert!(cfg.routes.is_empty());
    }

    // ── Persistence of edits ─────────────────────────────────────────────────

    #[test]
    fn test_add_then_remove_then_save_and_reload() {
        let dir = TempDir::new().unwrap();
        let path = tmp_config_path(&dir);

        let mut cfg = DevBindConfig::default();
        cfg.add_route("ephemeral.test".to_string(), 12345);
        cfg.save(&path).unwrap();

        // Simulate cleanup: remove the route and save again
        cfg.remove_route("ephemeral.test");
        cfg.save(&path).unwrap();

        let reloaded = DevBindConfig::load(&path).unwrap();
        assert!(
            reloaded.routes.is_empty(),
            "removed route must not appear after reload"
        );
    }
