use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use std::collections::HashSet;

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
                domains.iter()
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

        fs::write(self.hosts_file, final_content).context("Failed to write to hosts file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_update_routes() {
        let tmp = NamedTempFile::new().unwrap();
        let manager = HostsManager::new(tmp.path());

        // Initial insert
        manager.update_routes(&["test.local".to_string()]).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains(MARKER_START));
        assert!(content.contains("127.0.0.1 test.local"));

        // Update insert
        manager.update_routes(&["test.local".to_string(), "foo.local".to_string()]).unwrap();
        let content2 = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content2.contains("127.0.0.1 test.local"));
        assert!(content2.contains("127.0.0.1 foo.local"));
        assert_eq!(content2.matches(MARKER_START).count(), 1);

        // Remove all
        manager.update_routes(&[]).unwrap();
        let content3 = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(!content3.contains(MARKER_START));
        assert!(!content3.contains("127.0.0.1 test.local"));
    }
}
