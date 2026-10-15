//! Tool installer orchestrating download, safe extraction, junction creation, and PATH management.

use crate::environment::PathEditor;
use crate::error::{CoreError, Result};
use crate::installation::downloader::ToolDownloader;
use crate::installation::extractor::SafeExtractor;
use crate::installation::junction::JunctionManager;
use crate::installation::post_install::PostInstallRunner;
use crate::installation::store::{InstalledToolRecord, InstalledToolStore};
use crate::models::{ToolManifest, ToolVersion};
use crate::persistence::AppPaths;
use dotnet_windows::{get_user_path, set_user_path};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ToolInstaller;

impl ToolInstaller {
    /// Return the root tools directory (%LOCALAPPDATA%\Dotnet\tools).
    pub fn tools_root() -> PathBuf {
        AppPaths::tools_dir()
    }

    /// Install a tool version from manifest.
    pub async fn install<F>(
        manifest: &ToolManifest,
        version: &ToolVersion,
        store: &InstalledToolStore,
        mut progress_cb: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(f64),
    {
        let root = Self::tools_root();
        let target_dir = root.join(&manifest.id).join(&version.version);
        let current_junction = root.join(&manifest.id).join("current");
        let staging_id = format!("staging_{}_{}", manifest.id, std::process::id());
        let staging_dir = root.join(".staging").join(staging_id);

        fs::create_dir_all(&staging_dir)?;

        // 1. Download
        let archive_path = match version.source.source_type.as_str() {
            "zip" | "archive" => {
                ToolDownloader::download(
                    &version.source.url,
                    &version.source.sha256,
                    &manifest.allowed_hosts,
                    |p| progress_cb(p * 0.7), // 0% - 70%
                )
                .await?
            }
            "adopt" => {
                return Err(CoreError::Validation(
                    "Adopt source type must be handled via adoption scanner".into(),
                ));
            }
            other => {
                return Err(CoreError::Validation(format!(
                    "Unsupported source type: {other}"
                )));
            }
        };

        // 2. Extract to staging
        SafeExtractor::extract_zip(
            &archive_path,
            &staging_dir,
            version.source.strip_root,
            crate::installation::extractor::DEFAULT_MAX_UNCOMPRESSED_BYTES,
            |p| progress_cb(70.0 + p * 0.2), // 70% - 90%
        )?;

        // 3. Post-install hooks
        PostInstallRunner::run(manifest, &staging_dir)?;

        // 4. Move staging to target directory
        if target_dir.exists() {
            let _ = fs::remove_dir_all(&target_dir);
        }
        if let Some(parent) = target_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&staging_dir, &target_dir)?;

        // 5. Update 'current' junction
        let _ = JunctionManager::create_or_update_junction(&current_junction, &target_dir);

        // 6. Update Windows User PATH
        let mut added_paths = Vec::new();
        if let Ok(current_user_path) = get_user_path() {
            let mut updated_path = current_user_path;

            for bin_rel in &manifest.layout.bin_dirs {
                let bin_full = if current_junction.exists() {
                    current_junction.join(bin_rel)
                } else {
                    target_dir.join(bin_rel)
                };

                let bin_str = bin_full.to_string_lossy().to_string();
                updated_path = PathEditor::add_path(&updated_path, &bin_str, true);
                added_paths.push(bin_str);
            }

            let _ = set_user_path(&updated_path);
        }

        // 7. Register in tool store
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = InstalledToolRecord {
            tool_id: manifest.id.clone(),
            display_name: manifest.display_name.clone(),
            version: version.version.clone(),
            install_path: target_dir.to_string_lossy().to_string(),
            is_adopted: false,
            is_active: true,
            added_path_entries: added_paths,
            installed_at_secs: now,
        };

        store.register_tool(record)?;

        progress_cb(100.0);
        tracing::info!(
            "Successfully installed {} v{}",
            manifest.display_name,
            version.version
        );

        Ok(target_dir)
    }

    /// Uninstall an installed tool version.
    pub fn uninstall(record: &InstalledToolRecord, store: &InstalledToolStore) -> Result<()> {
        // 1. Remove added PATH entries
        if let Ok(current_path) = get_user_path() {
            let mut updated = current_path;
            for path in &record.added_path_entries {
                updated = PathEditor::remove_path(&updated, path);
            }
            let _ = set_user_path(&updated);
        }

        // 2. Delete junction if active
        if !record.is_adopted {
            let junction = Self::tools_root().join(&record.tool_id).join("current");
            let _ = JunctionManager::delete_junction(junction);

            // 3. Delete directory
            let install_path = Path::new(&record.install_path);
            if install_path.exists() {
                let _ = fs::remove_dir_all(install_path);
            }
        }

        // 4. Unregister from store
        store.unregister_tool(&record.tool_id, &record.version)?;
        tracing::info!("Uninstalled {} v{}", record.display_name, record.version);

        Ok(())
    }
}
