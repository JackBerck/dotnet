//! PATH Shadow Analyzer: detects system-level binary collisions shadowing user-space tools.

use crate::environment::PathEditor;
use crate::error::Result;
use dotnet_windows::{get_system_path, get_user_path};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowAnalysisResult {
    pub binary_name: String,
    pub user_path_location: Option<String>,
    pub system_path_location: Option<String>,
    pub is_shadowed: bool,
    pub recommendation: String,
}

pub struct PathShadowAnalyzer;

impl PathShadowAnalyzer {
    pub const DEFAULT_BINARIES: &'static [&'static str] = &[
        "php.exe",
        "node.exe",
        "composer.bat",
        "git.exe",
        "nginx.exe",
        "bun.exe",
        "go.exe",
        "mysqld.exe",
        "postgres.exe",
        "redis-server.exe",
    ];

    /// Analyze whether User PATH binaries are shadowed by earlier System PATH entries.
    pub fn analyze(
        custom_binaries: Option<&[&str]>,
        custom_user_dirs: Option<&[PathBuf]>,
    ) -> Result<Vec<ShadowAnalysisResult>> {
        let user_path_str = get_user_path().unwrap_or_default();
        let system_path_str = get_system_path().unwrap_or_default();

        let mut user_dirs = PathEditor::split_path(&user_path_str);
        if let Some(extra) = custom_user_dirs {
            for dir in extra {
                let dir_str = dir.to_string_lossy().to_string();
                if !user_dirs.contains(&dir_str) {
                    user_dirs.push(dir_str);
                }
            }
        }

        let system_dirs = PathEditor::split_path(&system_path_str);
        let binaries = custom_binaries.unwrap_or(Self::DEFAULT_BINARIES);
        let mut results = Vec::new();

        for &binary in binaries {
            let found_in_user = find_binary_in_dirs(&user_dirs, binary);
            if found_in_user.is_none() {
                continue; // Not managed or installed in user space
            }

            let found_in_system = find_binary_in_dirs(&system_dirs, binary);
            let is_shadowed = found_in_system.is_some();

            let recommendation = if is_shadowed {
                format!(
                    "System PATH takes precedence in Windows! Remove '{}' from System PATH to use the managed version.",
                    found_in_system.as_deref().unwrap_or("")
                )
            } else {
                "OK - Managed version has precedence.".to_string()
            };

            results.push(ShadowAnalysisResult {
                binary_name: binary.to_string(),
                user_path_location: found_in_user,
                system_path_location: found_in_system,
                is_shadowed,
                recommendation,
            });
        }

        Ok(results)
    }
}

fn find_binary_in_dirs(dirs: &[String], binary_name: &str) -> Option<String> {
    for dir in dirs {
        let candidate = Path::new(dir).join(binary_name);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_detection_logic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let user_bin_dir = temp_dir.path().join("user_bin");
        let sys_bin_dir = temp_dir.path().join("sys_bin");

        std::fs::create_dir_all(&user_bin_dir).unwrap();
        std::fs::create_dir_all(&sys_bin_dir).unwrap();

        let user_node = user_bin_dir.join("node.exe");
        let sys_node = sys_bin_dir.join("node.exe");

        std::fs::write(&user_node, "user node").unwrap();
        std::fs::write(&sys_node, "sys node").unwrap();

        let user_dirs = vec![user_bin_dir.to_string_lossy().to_string()];
        let sys_dirs = vec![sys_bin_dir.to_string_lossy().to_string()];

        let found_user = find_binary_in_dirs(&user_dirs, "node.exe");
        let found_sys = find_binary_in_dirs(&sys_dirs, "node.exe");

        assert!(found_user.is_some());
        assert!(found_sys.is_some());
    }
}
