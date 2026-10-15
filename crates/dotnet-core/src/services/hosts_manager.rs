//! Windows Hosts File Manager with Managed Blocks and atomic safety.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

pub const BLOCK_START: &str = "# >>> Dotnet (managed) >>>";
pub const BLOCK_END: &str = "# <<< Dotnet (managed) <<<";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostEntry {
    pub ip: String,
    pub hostname: String,
    pub is_enabled: bool,
    pub is_managed: bool,
}

pub struct HostsFileManager;

impl HostsFileManager {
    /// Return default Windows hosts file path: C:\Windows\System32\drivers\etc\hosts
    pub fn default_hosts_path() -> PathBuf {
        if let Ok(windir) = std::env::var("WINDIR") {
            PathBuf::from(windir).join(r"System32\drivers\etc\hosts")
        } else {
            PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
        }
    }

    /// RFC 1123 Hostname validation.
    pub fn is_valid_hostname(hostname: &str) -> bool {
        let name = hostname.trim();
        if name.is_empty() || name.len() > 255 {
            return false;
        }

        for label in name.split('.') {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
        }
        true
    }

    /// Parse hosts file content into a list of HostEntry items.
    pub fn parse_content(content: &str) -> Vec<HostEntry> {
        let mut entries = Vec::new();
        let mut in_managed = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case(BLOCK_START) {
                in_managed = true;
                continue;
            }
            if trimmed.eq_ignore_ascii_case(BLOCK_END) {
                in_managed = false;
                continue;
            }

            if trimmed.is_empty() {
                continue;
            }

            let is_commented = trimmed.starts_with('#');
            let clean = if is_commented {
                trimmed.trim_start_matches('#').trim()
            } else {
                trimmed
            };

            let parts: Vec<&str> = clean.split_whitespace().collect();
            if parts.len() >= 2 && parts[0].parse::<IpAddr>().is_ok() {
                let ip = parts[0].to_string();
                for candidate in &parts[1..] {
                    if candidate.starts_with('#') {
                        break;
                    }
                    if Self::is_valid_hostname(candidate) {
                        entries.push(HostEntry {
                            ip: ip.clone(),
                            hostname: candidate.to_string(),
                            is_enabled: !is_commented,
                            is_managed: in_managed,
                        });
                    }
                }
            }
        }

        entries
    }

    /// Update managed block inside hosts file content, preserving external lines.
    pub fn render_with_managed_entries(
        original_content: &str,
        managed_entries: &[HostEntry],
    ) -> String {
        let mut before_block = Vec::new();
        let mut after_block = Vec::new();
        let mut in_managed = false;
        let mut passed_managed = false;

        for line in original_content.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case(BLOCK_START) {
                in_managed = true;
                continue;
            }
            if trimmed.eq_ignore_ascii_case(BLOCK_END) {
                in_managed = false;
                passed_managed = true;
                continue;
            }

            if in_managed {
                continue;
            }

            if !passed_managed {
                before_block.push(line);
            } else {
                after_block.push(line);
            }
        }

        // Clean up trailing empty lines in before_block
        while let Some(last) = before_block.last() {
            if last.trim().is_empty() {
                before_block.pop();
            } else {
                break;
            }
        }

        // Clean up leading empty lines in after_block
        while let Some(first) = after_block.first() {
            if first.trim().is_empty() {
                after_block.remove(0);
            } else {
                break;
            }
        }

        let mut output = String::with_capacity(original_content.len() + 512);

        for line in before_block {
            output.push_str(line);
            output.push_str("\r\n");
        }

        if !managed_entries.is_empty() {
            if !output.is_empty() {
                output.push_str("\r\n");
            }
            output.push_str(BLOCK_START);
            output.push_str("\r\n");

            for entry in managed_entries {
                let prefix = if entry.is_enabled { "" } else { "# " };
                output.push_str(&format!("{}{}  {}\r\n", prefix, entry.ip, entry.hostname));
            }

            output.push_str(BLOCK_END);
            output.push_str("\r\n");
        }

        if !after_block.is_empty() {
            output.push_str("\r\n");
            for line in after_block {
                output.push_str(line);
                output.push_str("\r\n");
            }
        }

        output
    }

    /// Create a timestamped backup of the hosts file in AppPaths::backups_dir().
    pub fn backup_hosts_file<P: AsRef<Path>>(hosts_path: P) -> Result<PathBuf> {
        let hosts_path = hosts_path.as_ref();
        if !hosts_path.exists() {
            return Err(CoreError::Config(format!(
                "Hosts file does not exist at {}",
                hosts_path.display()
            )));
        }

        let backups_dir = AppPaths::backups_dir();
        fs::create_dir_all(&backups_dir)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "backup".to_string());

        let backup_path = backups_dir.join(format!("hosts_{}.bak", timestamp));
        fs::copy(hosts_path, &backup_path)?;

        tracing::info!("Hosts file backed up to: {}", backup_path.display());
        Ok(backup_path)
    }

    /// Atomically write new content to hosts file via .tmp file.
    pub fn write_atomic<P: AsRef<Path>>(hosts_path: P, content: &str) -> Result<()> {
        let hosts_path = hosts_path.as_ref();
        let tmp_path = hosts_path.with_extension("tmp");

        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, hosts_path)?;

        Ok(())
    }

    /// Add or update a managed entry in the hosts file.
    pub fn add_or_update_managed_entry<P: AsRef<Path>>(
        hosts_path: P,
        ip: &str,
        hostname: &str,
    ) -> Result<()> {
        let hosts_path = hosts_path.as_ref();
        if !hosts_path.exists() {
            return Err(CoreError::Config(format!(
                "Hosts file not found at {}",
                hosts_path.display()
            )));
        }
        let content = fs::read_to_string(hosts_path)?;
        let entries = Self::parse_content(&content);
        let mut managed: Vec<HostEntry> = entries.into_iter().filter(|e| e.is_managed).collect();
        if let Some(existing) = managed
            .iter_mut()
            .find(|e| e.hostname.eq_ignore_ascii_case(hostname))
        {
            existing.ip = ip.to_string();
            existing.is_enabled = true;
        } else {
            managed.push(HostEntry {
                ip: ip.to_string(),
                hostname: hostname.to_string(),
                is_enabled: true,
                is_managed: true,
            });
        }
        let new_content = Self::render_with_managed_entries(&content, &managed);
        Self::write_atomic(hosts_path, &new_content)?;
        Ok(())
    }

    /// Remove a managed entry from the hosts file by hostname.
    pub fn remove_managed_entry<P: AsRef<Path>>(hosts_path: P, hostname: &str) -> Result<()> {
        let hosts_path = hosts_path.as_ref();
        if !hosts_path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(hosts_path)?;
        let entries = Self::parse_content(&content);
        let managed: Vec<HostEntry> = entries
            .into_iter()
            .filter(|e| e.is_managed && !e.hostname.eq_ignore_ascii_case(hostname))
            .collect();
        let new_content = Self::render_with_managed_entries(&content, &managed);
        Self::write_atomic(hosts_path, &new_content)?;
        Ok(())
    }

    /// Read all entries from default Windows hosts file.
    pub fn read_entries_from_default() -> Result<Vec<HostEntry>> {
        let path = Self::default_hosts_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)?;
        Ok(Self::parse_content(&content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_hostname() {
        assert!(HostsFileManager::is_valid_hostname("laravel.test"));
        assert!(HostsFileManager::is_valid_hostname("api.dev-site.local"));
        assert!(HostsFileManager::is_valid_hostname("localhost"));
        assert!(!HostsFileManager::is_valid_hostname(""));
        assert!(!HostsFileManager::is_valid_hostname("-bad.domain"));
        assert!(!HostsFileManager::is_valid_hostname("bad..domain"));
    }

    #[test]
    fn test_parse_and_render_managed_block() {
        let original = "# Windows hosts file\r\n127.0.0.1 localhost\r\n::1 localhost\r\n";
        let parsed = HostsFileManager::parse_content(original);
        assert_eq!(parsed.len(), 2);
        assert!(!parsed[0].is_managed);

        let managed_entries = vec![
            HostEntry {
                ip: "127.0.0.1".into(),
                hostname: "myproject.test".into(),
                is_enabled: true,
                is_managed: true,
            },
            HostEntry {
                ip: "127.0.0.1".into(),
                hostname: "admin.test".into(),
                is_enabled: false,
                is_managed: true,
            },
        ];

        let updated = HostsFileManager::render_with_managed_entries(original, &managed_entries);
        assert!(updated.contains(BLOCK_START));
        assert!(updated.contains("127.0.0.1  myproject.test"));
        assert!(updated.contains("# 127.0.0.1  admin.test"));
        assert!(updated.contains(BLOCK_END));
        assert!(updated.contains("127.0.0.1 localhost"));

        let reparsed = HostsFileManager::parse_content(&updated);
        let managed: Vec<_> = reparsed.into_iter().filter(|e| e.is_managed).collect();
        assert_eq!(managed.len(), 2);
        assert!(managed[0].is_enabled);
        assert!(!managed[1].is_enabled);
    }

    #[test]
    fn test_add_and_remove_managed_entry() {
        let temp = tempfile::tempdir().unwrap();
        let hosts_file = temp.path().join("hosts");
        fs::write(&hosts_file, "127.0.0.1 localhost\r\n").unwrap();

        HostsFileManager::add_or_update_managed_entry(&hosts_file, "127.0.0.1", "site.test")
            .unwrap();
        let content = fs::read_to_string(&hosts_file).unwrap();
        assert!(content.contains("127.0.0.1  site.test"));

        // Remove entry
        HostsFileManager::remove_managed_entry(&hosts_file, "site.test").unwrap();
        let content2 = fs::read_to_string(&hosts_file).unwrap();
        assert!(!content2.contains("site.test"));
    }
}
