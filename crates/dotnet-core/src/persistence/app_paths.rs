use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub struct AppPaths;

impl AppPaths {
    /// Returns the base directory for Dotnet application data.
    /// Priority:
    /// 1. `DOTNET_DATA_DIR` environment variable
    /// 2. `%LOCALAPPDATA%\Dotnet` on Windows
    /// 3. Current directory fallback
    pub fn data_dir() -> &'static Path {
        DATA_DIR.get_or_init(|| {
            if let Ok(custom) = std::env::var("DOTNET_DATA_DIR") {
                if !custom.trim().is_empty() {
                    let path = PathBuf::from(custom);
                    let _ = std::fs::create_dir_all(&path);
                    return path;
                }
            }

            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data).join("Dotnet");
                let _ = std::fs::create_dir_all(&path);
                return path;
            }

            let fallback = PathBuf::from(".dotnet_data");
            let _ = std::fs::create_dir_all(&fallback);
            fallback
        })
    }

    /// Resolve a file path relative to the Dotnet data directory.
    pub fn get_path<P: AsRef<Path>>(subpath: P) -> PathBuf {
        Self::data_dir().join(subpath)
    }

    /// Tools directory: %LOCALAPPDATA%\Dotnet\tools
    pub fn tools_dir() -> PathBuf {
        Self::get_path("tools")
    }

    /// Nginx sites directory: %LOCALAPPDATA%\Dotnet\nginx\sites
    pub fn nginx_sites_dir() -> PathBuf {
        Self::get_path("nginx").join("sites")
    }

    /// SSL certificate directory: %LOCALAPPDATA%\Dotnet\ssl
    pub fn ssl_dir() -> PathBuf {
        Self::get_path("ssl")
    }

    /// Backups directory: %LOCALAPPDATA%\Dotnet\backups
    pub fn backups_dir() -> PathBuf {
        Self::get_path("backups")
    }
}
