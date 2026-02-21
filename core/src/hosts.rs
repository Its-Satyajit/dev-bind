use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const MARKER_START: &str = "# --- DevBind Start ---";
const MARKER_END: &str = "# --- DevBind End ---";

pub struct HostsManager<'a> {
    hosts_file: &'a Path,
}

impl<'a> HostsManager<'a> {
    pub fn new(hosts_file: &'a Path) -> Self {
        Self { hosts_file }
    }

    pub fn update_routes(&self, domains: &[String]) -> Result<()> {
        let content = if self.hosts_file.exists() {
            fs::read_to_string(self.hosts_file).context("Failed to read hosts file")?
        } else {
            String::new()
        };

        let mut out_lines = Vec::new();
        let mut in_devbind_block = false;

        // Parse existing content, stripping out our old block
        for line in content.lines() {
            if line.trim() == MARKER_START {
                in_devbind_block = true;
                continue;
            }
            if line.trim() == MARKER_END {
                in_devbind_block = false;
                continue;
            }
            if !in_devbind_block {
                out_lines.push(line.to_string());
            }
        }

        // Add back our new block if there are domains
        if !domains.is_empty() {
            out_lines.push(MARKER_START.to_string());

            // Deduplicate domains
            let mut unique_domains: Vec<String> = {
                let mut set = HashSet::new();
                domains
                    .iter()
                    .filter(|d| set.insert((*d).clone()))
                    .cloned()
                    .collect()
            };
            unique_domains.sort(); // Predictable ordering

            for domain in unique_domains {
                out_lines.push(format!("127.0.0.1 {}", domain));
            }
            out_lines.push(MARKER_END.to_string());
        }

        // Ensure newline at end
        let mut final_content = out_lines.join("\n");
        if !final_content.ends_with('\n') {
            final_content.push('\n');
        }

        if let Err(e) = fs::write(self.hosts_file, &final_content) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                // Try elevating privileges
                use std::io::Write;
                use std::process::Command;

                let mut tmp_file = tempfile::Builder::new()
                    .prefix("devbind-hosts")
                    .tempfile()
                    .context("Failed to create temporary file for elevated write")?;

                tmp_file
                    .write_all(final_content.as_bytes())
                    .context("Failed to write to temporary file")?;

                let tmp_path = tmp_file.into_temp_path();

                // Try pkexec first (better for GUI)
                let status = Command::new("pkexec")
                    .arg("cp")
                    .arg(&tmp_path)
                    .arg(self.hosts_file)
                    .status();

                match status {
                    Ok(s) if s.success() => {}
                    _ => {
                        // Fallback to sudo for CLI or headless
                        let status2 = Command::new("sudo")
                            .arg("cp")
                            .arg(&tmp_path)
                            .arg(self.hosts_file)
                            .status()
                            .context("Failed to execute elevated copy (Permission Denied). Try running with sudo.")?;

                        // Set proper permissions just in case
                        let _ = Command::new("sudo")
                            .arg("chmod")
                            .arg("644")
                            .arg(self.hosts_file)
                            .status();

                        if !status2.success() {
                            anyhow::bail!(
                                "Failed to write to hosts file even with elevated privileges"
                            );
                        }
                    }
                }
            } else {
                return Err(e).context("Failed to write to hosts file");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
