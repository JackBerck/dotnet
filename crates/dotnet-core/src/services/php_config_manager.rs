//! PHP Configuration Manager (php.ini, extension toggle, and preset profiles).

use crate::config::IniDocument;
use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhpExtensionItem {
    pub name: String,
    pub file_name: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpSettingItem {
    pub key: String,
    pub current_value: String,
    pub default_value: String,
    pub description: String,
    pub category: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpRuntimeInfo {
    pub php_dir: String,
    pub ini_path: String,
    pub is_installed: bool,
    pub detected_version: String,
    pub active_extensions_count: usize,
    pub total_extensions_count: usize,
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
        let (php_dir, ini_path) = Self::discover_php();
        Self { php_dir, ini_path }
    }

    /// Dynamically locate PHP directory and active php.ini on host system.
    pub fn discover_php() -> (PathBuf, PathBuf) {
        // 1. Check InstalledToolStore
        if let Ok(Some(tool)) =
            crate::installation::store::InstalledToolStore::new().get_active_tool("php")
        {
            let p = PathBuf::from(&tool.install_path);
            let dir = if p.is_file() {
                p.parent().unwrap_or(&p).to_path_buf()
            } else {
                p
            };
            if dir.is_dir() {
                let ini = dir.join("php.ini");
                return (dir, ini);
            }
        }

        // 2. Check well-known host installation directories
        let candidates = [
            r"C:\tools\php85",
            r"C:\tools\php",
            r"D:\Apps\php",
            r"C:\Apps\php",
            r"C:\php",
            r"C:\Program Files\PHP",
        ];

        for cand in candidates {
            let p = Path::new(cand);
            if p.join("php.exe").is_file() {
                let dir = p.to_path_buf();
                let ini = dir.join("php.ini");
                if !ini.is_file() {
                    if dir.join("php.ini-development").is_file() {
                        let _ = fs::copy(dir.join("php.ini-development"), &ini);
                    } else if dir.join("php.ini-production").is_file() {
                        let _ = fs::copy(dir.join("php.ini-production"), &ini);
                    }
                }
                return (dir, ini);
            }
        }

        // 3. Check system PATH
        if let Ok(exe_path) = which::which("php.exe") {
            if let Some(parent) = exe_path.parent() {
                let dir = parent.to_path_buf();
                let ini = dir.join("php.ini");
                return (dir, ini);
            }
        }

        // 4. Default managed path
        let dir = AppPaths::tools_dir().join("php").join("current");
        let ini = dir.join("php.ini");
        (dir, ini)
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

    /// Read raw text content of php.ini.
    pub fn get_raw_ini(&self) -> Result<String> {
        if !self.ini_path.is_file() {
            return Ok(String::new());
        }
        Ok(fs::read_to_string(&self.ini_path)?)
    }

    /// Save raw text content of php.ini with automatic backup.
    pub fn save_raw_ini(&self, content: &str) -> Result<()> {
        if self.ini_path.is_file() {
            let _ = self.backup_ini_file();
        } else if let Some(parent) = self.ini_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&self.ini_path, content)?;
        Ok(())
    }

    /// Get PHP runtime information.
    pub fn get_runtime_info(&self) -> PhpRuntimeInfo {
        let is_installed =
            self.php_dir.join("php.exe").is_file() || which::which("php.exe").is_ok();
        let exts = self.get_available_extensions().unwrap_or_default();
        let active = exts.iter().filter(|e| e.is_enabled).count();

        let mut detected_version = "Unknown".to_string();
        let php_exe = self.php_dir.join("php.exe");
        let target_exe = if php_exe.is_file() {
            Some(php_exe)
        } else {
            which::which("php.exe").ok()
        };

        if let Some(exe) = target_exe {
            let mut cmd = std::process::Command::new(exe);
            cmd.arg("-v");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }
            if let Ok(output) = cmd.output() {
                let out = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = out.lines().next() {
                    detected_version = first_line.to_string();
                }
            }
        }

        PhpRuntimeInfo {
            php_dir: self.php_dir.to_string_lossy().to_string(),
            ini_path: self.ini_path.to_string_lossy().to_string(),
            is_installed,
            detected_version,
            active_extensions_count: active,
            total_extensions_count: exts.len(),
        }
    }

    /// Get common editable PHP configuration directives.
    pub fn get_common_settings(&self) -> Vec<PhpSettingItem> {
        let specs = [
            (
                "memory_limit",
                "512M",
                "128M",
                "Maximum memory amount that a script may consume",
                "Resource Limits",
                vec!["128M", "256M", "512M", "1024M", "2048M", "-1"],
            ),
            (
                "upload_max_filesize",
                "64M",
                "2M",
                "Maximum size of an uploaded file",
                "File Uploads",
                vec!["16M", "32M", "64M", "128M", "256M", "512M", "1024M"],
            ),
            (
                "post_max_size",
                "64M",
                "8M",
                "Maximum size of POST data that PHP will accept",
                "File Uploads",
                vec!["16M", "32M", "64M", "128M", "256M", "512M", "1024M"],
            ),
            (
                "max_execution_time",
                "120",
                "30",
                "Maximum execution time of each script, in seconds",
                "Resource Limits",
                vec!["30", "60", "120", "300", "600", "0"],
            ),
            (
                "max_input_time",
                "60",
                "60",
                "Maximum time script may spend parsing input data",
                "Resource Limits",
                vec!["30", "60", "120", "300", "-1"],
            ),
            (
                "display_errors",
                "On",
                "Off",
                "Print errors out as part of the HTTP output",
                "Error Handling",
                vec!["On", "Off"],
            ),
            (
                "display_startup_errors",
                "On",
                "Off",
                "Display errors that occur during PHP startup sequence",
                "Error Handling",
                vec!["On", "Off"],
            ),
            (
                "error_reporting",
                "E_ALL",
                "E_ALL",
                "Error reporting level bitmask",
                "Error Handling",
                vec!["E_ALL", "E_ALL & ~E_NOTICE", "E_ALL & ~E_DEPRECATED", "0"],
            ),
            (
                "date.timezone",
                "UTC",
                "UTC",
                "Default timezone used by date/time functions",
                "Localization",
                vec![
                    "UTC",
                    "Asia/Jakarta",
                    "Asia/Singapore",
                    "Europe/London",
                    "America/New_York",
                ],
            ),
            (
                "default_socket_timeout",
                "60",
                "60",
                "Default timeout for socket-based streams",
                "Network",
                vec!["30", "60", "120"],
            ),
            (
                "opcache.enable",
                "1",
                "1",
                "Enables Zend OPcache bytecode caching engine",
                "Performance",
                vec!["1", "0"],
            ),
        ];

        let mut items = Vec::new();
        for (key, fallback, default_val, desc, cat, opts) in specs {
            let cur = self
                .get_setting(key)
                .ok()
                .flatten()
                .unwrap_or_else(|| fallback.to_string());
            items.push(PhpSettingItem {
                key: key.to_string(),
                current_value: cur,
                default_value: default_val.to_string(),
                description: desc.to_string(),
                category: cat.to_string(),
                options: opts.into_iter().map(String::from).collect(),
            });
        }
        items
    }

    /// Update multiple PHP settings in batch with automatic backup.
    pub fn update_settings(&self, settings: &HashMap<String, String>) -> Result<()> {
        if !self.ini_path.is_file() {
            if let Some(parent) = self.ini_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&self.ini_path, "; PHP configuration\r\n")?;
        }

        let _ = self.backup_ini_file();

        let content = fs::read_to_string(&self.ini_path)?;
        let mut doc = IniDocument::parse(&content);
        for (k, v) in settings {
            doc.set_value(k, v);
        }
        fs::write(&self.ini_path, doc.to_string())?;
        Ok(())
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
            if let Some(parent) = self.ini_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&self.ini_path, "; PHP configuration\r\n")?;
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

        // Fallback standard PHP extensions if no dlls discovered on disk
        if list.is_empty() {
            let standard_extensions = [
                "curl",
                "fileinfo",
                "gd",
                "intl",
                "mbstring",
                "mysqli",
                "openssl",
                "pdo_mysql",
                "pdo_pgsql",
                "pdo_sqlite",
                "soap",
                "sockets",
                "sodium",
                "sqlite3",
                "tidy",
                "xsl",
                "zip",
            ];

            for ext in standard_extensions {
                let is_enabled = active_doc
                    .as_ref()
                    .is_some_and(|doc| doc.is_extension_enabled(ext));

                list.push(PhpExtensionItem {
                    name: ext.to_string(),
                    file_name: format!("php_{ext}.dll"),
                    is_enabled,
                });
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

    #[test]
    fn test_common_settings_and_raw_ini() {
        let dir = tempdir().expect("tempdir");
        let php_dir = dir.path().join("php");
        let ini_path = php_dir.join("php.ini");
        fs::create_dir_all(&php_dir).expect("mkdir");
        fs::write(
            &ini_path,
            "upload_max_filesize = 16M\npost_max_size = 16M\n",
        )
        .expect("write ini");

        let mgr = PhpConfigManager::with_paths(php_dir, ini_path.clone());
        let settings = mgr.get_common_settings();
        assert!(settings
            .iter()
            .any(|s| s.key == "upload_max_filesize" && s.current_value == "16M"));

        let mut batch = HashMap::new();
        batch.insert("upload_max_filesize".to_string(), "128M".to_string());
        batch.insert("post_max_size".to_string(), "128M".to_string());
        mgr.update_settings(&batch).expect("batch update");

        assert_eq!(
            mgr.get_setting("upload_max_filesize").unwrap(),
            Some("128M".to_string())
        );
        assert_eq!(
            mgr.get_setting("post_max_size").unwrap(),
            Some("128M".to_string())
        );

        let raw = mgr.get_raw_ini().expect("get raw");
        assert!(raw.contains("upload_max_filesize = 128M"));

        mgr.save_raw_ini("memory_limit = 2048M\n")
            .expect("save raw");
        assert_eq!(
            mgr.get_setting("memory_limit").unwrap(),
            Some("2048M".to_string())
        );
    }
}
