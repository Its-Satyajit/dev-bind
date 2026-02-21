use super::*;
    use tempfile::NamedTempFile;

    // ── Basic insertion and update ─────────────────────────────────────────────

    #[test]
    fn test_update_routes_basic_insert_and_removal() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // Initial insert
        manager
            .update_routes(&["example.test".to_string()])
            .unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains(MARKER_START));
        assert!(content.contains("127.0.0.1 example.test"));

        // Update insert
        manager
            .update_routes(&["example.test".to_string(), "foo.test".to_string()])
            .unwrap();
        let content2 = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content2.contains("127.0.0.1 example.test"));
        assert!(content2.contains("127.0.0.1 foo.test"));
        assert_eq!(content2.matches(MARKER_START).count(), 1);

        // Remove all
        manager.update_routes(&[]).unwrap();
        let content3 = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(!content3.contains(MARKER_START));
        assert!(!content3.contains("127.0.0.1 example.test"));
    }

    #[test]
    fn test_update_routes_single_domain() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        manager.update_routes(&["myapp.test".to_string()]).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("127.0.0.1 myapp.test"));
        assert!(content.contains(MARKER_START));
        assert!(content.contains(MARKER_END));
    }

    // ── Empty domain list ────────────────────────────────────────────────────

    #[test]
    fn test_update_routes_empty_list_produces_no_markers() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // Write some routes first, then clear
        manager.update_routes(&["alpha.test".to_string()]).unwrap();
        manager.update_routes(&[]).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(
            !content.contains(MARKER_START),
            "no block when domain list is empty"
        );
        assert!(!content.contains(MARKER_END));
    }

    #[test]
    fn test_update_routes_empty_list_on_empty_file_is_noop() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // File starts empty
        manager.update_routes(&[]).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        // Only a trailing newline should be present — no markers
        assert!(!content.contains(MARKER_START));
    }

    // ── Pre-existing content preservation ───────────────────────────────────

    #[test]
    fn test_pre_existing_non_devbind_content_is_preserved() {
        let tmp = NamedTempFile::new().unwrap();
        // Seed hosts file with typical system content
        let seed = "127.0.0.1 localhost\n127.0.1.1 mymachine\n";
        std::fs::write(tmp.path(), seed).unwrap();

        let manager = HostsManager::new(tmp.path());
        manager.update_routes(&["devapp.test".to_string()]).unwrap();

        let result = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(
            result.contains("127.0.0.1 localhost"),
            "existing localhost entry must be retained"
        );
        assert!(
            result.contains("127.0.1.1 mymachine"),
            "existing mymachine entry must be retained"
        );
        assert!(result.contains("127.0.0.1 devapp.test"));
    }

    #[test]
    fn test_previous_devbind_block_is_replaced_not_duplicated() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // First write
        manager.update_routes(&["first.test".to_string()]).unwrap();
        // Second write should NOT accumulate a second block
        manager.update_routes(&["second.test".to_string()]).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert_eq!(
            content.matches(MARKER_START).count(),
            1,
            "only one DevBind block must exist after two consecutive writes"
        );
        // Old domain must be gone
        assert!(
            !content.contains("127.0.0.1 first.test"),
            "old domain must be removed after update"
        );
        // New domain must be present
        assert!(content.contains("127.0.0.1 second.test"));
    }

    // ── Deduplication ────────────────────────────────────────────────────────

    #[test]
    fn test_duplicate_domains_in_input_are_deduplicated() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // Pass the same domain twice
        let domains = vec![
            "dup.test".to_string(),
            "dup.test".to_string(),
            "other.test".to_string(),
        ];
        manager.update_routes(&domains).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        // Must appear exactly once
        assert_eq!(
            content.matches("127.0.0.1 dup.test").count(),
            1,
            "duplicate domains must produce only one hosts entry"
        );
    }

    // ── Output format ────────────────────────────────────────────────────────

    #[test]
    fn test_output_ends_with_newline() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        manager
            .update_routes(&["newline.test".to_string()])
            .unwrap();
        let bytes = std::fs::read(tmp.path()).unwrap();
        assert_eq!(
            bytes.last(),
            Some(&b'\n'),
            "hosts file must always end with a newline"
        );
    }

    #[test]
    fn test_nonexistent_hosts_file_is_created() {
        // Use a path that does not exist yet inside a temp dir
        let dir = tempfile::tempdir().unwrap();
        let new_path = dir.path().join("new_hosts");
        let manager = HostsManager::new(&new_path);

        manager.update_routes(&["fresh.test".to_string()]).unwrap();
        assert!(
            new_path.exists(),
            "hosts file must be created when it does not exist"
        );
        let content = std::fs::read_to_string(&new_path).unwrap();
        assert!(content.contains("127.0.0.1 fresh.test"));
    }
