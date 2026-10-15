//! PHP Configuration Manager (php.ini, extension toggle, and preset profiles).

use crate::config::IniDocument;
use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhpExtensionItem {
    pub name: String,
    pub file_name: String,
    pub is_enabled: bool,
}

pub struct PhpConfigManager {
    php_dir: PathBuf,
    ini_path: PathBuf,
}

impl Default for PhpConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpConfigManager {
    #[must_use]
    pub fn new() -> Self {
        let php_dir = AppPaths::tools_dir().join("php").join("current");
        let ini_path = php_dir.join("php.ini");
        Self { php_dir, ini_path }
    }

    #[must_use]
    pub fn with_paths(php_dir: PathBuf, ini_path: PathBuf) -> Self {
        Self { php_dir, ini_path }
    }

    #[must_use]
    pub fn php_dir(&self) -> &Path {
        &self.php_dir
    }

    #[must_use]
    pub fn ini_path(&self) -> &Path {
        &self.ini_path
    }

    /// Create a timestamped backup of php.ini in %LOCALAPPDATA%\Dotnet\backups\.
    pub fn backup_ini_file(&self) -> Result<PathBuf> {
        if !self.ini_path.is_file() {
            return Err(CoreError::Config(format!(
                "php.ini does not exist at {}",
                self.ini_path.display()
            )));
        }

        let backups_dir = AppPaths::backups_dir();
        fs::create_dir_all(&backups_dir)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "backup".to_string());

        let backup_path = backups_dir.join(format!("php_ini_{timestamp}.bak"));
        fs::copy(&self.ini_path, &backup_path)?;
        tracing::info!(
            "PhpConfigManager: Backed up php.ini to {}",
            backup_path.display()
        );
        Ok(backup_path)
    }

    /// Read setting value from php.ini.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        if !self.ini_path.is_file() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.ini_path)?;
        let doc = IniDocument::parse(&content);
        Ok(doc.get_value(key))
    }

    /// Update or uncomment setting in php.ini with automatic backup.
    pub fn update_setting(&self, key: &str, value: &str) -> Result<()> {
        if !self.ini_path.is_file() {
            return Err(CoreError::Config(format!(
                "php.ini file not found at {}",
                self.ini_path.display()
            )));
        }

        let _ = self.backup_ini_file();

        let content = fs::read_to_string(&self.ini_path)?;
        let mut doc = IniDocument::parse(&content);
        doc.set_value(key, value);
        fs::write(&self.ini_path, doc.to_string())?;

        tracing::info!(
            "PhpConfigManager: Updated php.ini setting '{}' = '{}'",
            key,
            value
        );
        Ok(())
    }

    /// Enumerate all available extensions in <php_dir>/ext/ and check enabled status in php.ini.
    pub fn get_available_extensions(&self) -> Result<Vec<PhpExtensionItem>> {
        let mut list = Vec::new();
        let ext_dir = self.php_dir.join("ext");

        let active_doc = if self.ini_path.is_file() {
            let content = fs::read_to_string(&self.ini_path)?;
            Some(IniDocument::parse(&content))
        } else {
            None
        };

        if ext_dir.is_dir() {
            for entry in fs::read_dir(ext_dir)? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                if file_type.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.to_lowercase().ends_with(".dll") {
                        let norm = IniDocument::normalize_extension_name(&file_name);
                        let is_enabled = active_doc
                            .as_ref()
                            .is_some_and(|doc| doc.is_extension_enabled(&norm));

                        list.push(PhpExtensionItem {
                            name: norm,
                            file_name,
                            is_enabled,
                        });
                    }
                }
            }
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    /// Enable or disable a PHP extension in php.ini with automatic backup.
    pub fn toggle_extension(&self, ext_name: &str, enable: bool) -> Result<()> {
        if !self.ini_path.is_file() {
            return Err(CoreError::Config(format!(
                "php.ini file not found at {}",
                self.ini_path.display()
            )));
        }

        let _ = self.backup_ini_file();

        let content = fs::read_to_string(&self.ini_path)?;
        let mut doc = IniDocument::parse(&content);

        let norm = IniDocument::normalize_extension_name(ext_name);
        if enable {
            doc.enable_extension(&norm);
        } else {
            doc.disable_extension(&norm);
        }

        fs::write(&self.ini_path, doc.to_string())?;
        tracing::info!(
            "PhpConfigManager: Toggled extension '{}' -> {}",
            norm,
            if enable { "Enabled" } else { "Disabled" }
        );
        Ok(())
    }

    /// Apply an official PHP preset (development vs production).
    pub fn apply_preset(&self, preset_name: &str) -> Result<String> {
        let preset_file_name = if preset_name.eq_ignore_ascii_case("production") {
            "php.ini-production"
        } else {
            "php.ini-development"
        };

        let preset_path = self.php_dir.join(preset_file_name);
        if !preset_path.is_file() {
            return Err(CoreError::Config(format!(
                "Preset file not found at {}",
                preset_path.display()
            )));
        }

        if self.ini_path.is_file() {
            let _ = self.backup_ini_file();
        }

        fs::copy(&preset_path, &self.ini_path)?;
        tracing::info!(
            "PhpConfigManager: Applied preset '{}' from {}",
            preset_name,
            preset_path.display()
        );
        Ok(format!("Successfully applied PHP preset '{preset_name}'."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_php_config_settings_and_backup() {
        let dir = tempdir().expect("tempdir");
        let php_dir = dir.path().join("php");
        let ini_path = php_dir.join("php.ini");
        fs::create_dir_all(&php_dir).expect("mkdir");
        fs::write(&ini_path, "memory_limit = 128M\n;extension=curl\n").expect("write ini");

        let mgr = PhpConfigManager::with_paths(php_dir, ini_path.clone());

        assert_eq!(
            mgr.get_setting("memory_limit").expect("get"),
            Some("128M".to_string())
        );

        mgr.update_setting("memory_limit", "512M").expect("update");
        assert_eq!(
            mgr.get_setting("memory_limit").expect("get"),
            Some("512M".to_string())
        );

        // Toggle extension on
        mgr.toggle_extension("curl", true).expect("enable curl");
        let updated_content = fs::read_to_string(&ini_path).expect("read");
        assert!(updated_content.contains("extension = curl"));

        // Toggle extension off
        mgr.toggle_extension("curl", false).expect("disable curl");
        let disabled_content = fs::read_to_string(&ini_path).expect("read");
        assert!(disabled_content.contains(";extension = curl"));
    }

    #[test]
    fn test_apply_preset() {
        let dir = tempdir().expect("tempdir");
        let php_dir = dir.path().join("php");
        let ini_path = php_dir.join("php.ini");
        fs::create_dir_all(&php_dir).expect("mkdir");

        fs::write(php_dir.join("php.ini-development"), "display_errors = On\n").expect("write dev");
        fs::write(&ini_path, "display_errors = Off\n").expect("write initial");

        let mgr = PhpConfigManager::with_paths(php_dir, ini_path.clone());
        mgr.apply_preset("development").expect("apply dev");

        let content = fs::read_to_string(&ini_path).expect("read");
        assert_eq!(content, "display_errors = On\n");
    }
}
