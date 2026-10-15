//! Safe ZIP extractor with Zip-Slip traversal defense and Zip-Bomb size guards.

use crate::error::{CoreError, Result};
use std::fs::{self, File};
use std::io::{copy, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub const DEFAULT_MAX_UNCOMPRESSED_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GB

pub struct SafeExtractor;

impl SafeExtractor {
    /// Extract a ZIP archive to target directory with security safeguards.
    pub fn extract_zip<P: AsRef<Path>, D: AsRef<Path>, F>(
        zip_file_path: P,
        destination_dir: D,
        strip_root: bool,
        max_bytes: u64,
        mut progress_cb: F,
    ) -> Result<()>
    where
        F: FnMut(f64),
    {
        let zip_path = zip_file_path.as_ref();
        let dest_dir = destination_dir.as_ref();

        if !zip_path.exists() {
            return Err(CoreError::Validation(format!(
                "ZIP archive not found: {}",
                zip_path.display()
            )));
        }

        fs::create_dir_all(dest_dir)?;
        let canonical_dest = dest_dir.canonicalize()?;

        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| CoreError::Validation(format!("Invalid ZIP archive: {e}")))?;

        // Determine common root prefix if strip_root is requested
        let root_prefix = if strip_root {
            Self::determine_root_prefix(&mut archive)
        } else {
            None
        };

        let total_entries = archive.len();
        let mut total_uncompressed: u64 = 0;

        for i in 0..total_entries {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| CoreError::Validation(format!("Failed reading ZIP entry {i}: {e}")))?;

            total_uncompressed += entry.size();
            if total_uncompressed > max_bytes {
                return Err(CoreError::Validation(format!(
                    "Zip-Bomb protection triggered: uncompressed size exceeded {} MB limit",
                    max_bytes / (1024 * 1024)
                )));
            }

            let entry_name = entry.name().replace('\\', "/");
            let mut rel_path = entry_name.as_str();

            if let Some(prefix) = &root_prefix {
                if rel_path.starts_with(prefix) {
                    rel_path = &rel_path[prefix.len()..];
                }
            }

            let rel_path = rel_path.trim_start_matches('/');
            if rel_path.is_empty() {
                continue;
            }

            // Zip-Slip security check: verify target path is inside canonical_dest
            let target_path = dest_dir.join(rel_path);

            // Canonicalize parent directory to prevent traversal tricks
            let parent_dir = target_path
                .parent()
                .ok_or_else(|| CoreError::Validation("Target path has no parent".into()))?;

            fs::create_dir_all(parent_dir)?;
            let canonical_parent = parent_dir.canonicalize()?;

            if !canonical_parent.starts_with(&canonical_dest) {
                return Err(CoreError::Validation(format!(
                    "Zip-Slip vulnerability detected! Malicious entry '{}' resolves outside target directory.",
                    entry.name()
                )));
            }

            if entry.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                let mut out_file = File::create(&target_path)?;
                copy(&mut entry, &mut out_file)?;
            }

            let pct = ((i + 1) as f64 / total_entries as f64) * 100.0;
            progress_cb(pct);
        }

        tracing::info!("Extracted archive safely to: {}", dest_dir.display());
        Ok(())
    }

    /// Detect if all entries share a single root directory prefix.
    fn determine_root_prefix<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
    ) -> Option<String> {
        let mut candidate_prefix: Option<String> = None;

        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().replace('\\', "/");
                let slash_idx = name.find('/')?;
                let prefix = name[..=slash_idx].to_string();

                match &candidate_prefix {
                    None => candidate_prefix = Some(prefix),
                    Some(candidate) => {
                        if !candidate.eq_ignore_ascii_case(&prefix) {
                            return None; // Multiple root folders exist
                        }
                    }
                }
            }
        }

        candidate_prefix
    }

    /// Install a single executable file.
    pub fn install_single_file<S: AsRef<Path>, D: AsRef<Path>>(
        source: S,
        dest_dir: D,
        dest_file_name: &str,
    ) -> Result<PathBuf> {
        let dest_dir = dest_dir.as_ref();
        fs::create_dir_all(dest_dir)?;

        let target = dest_dir.join(dest_file_name);
        fs::copy(source.as_ref(), &target)?;
        tracing::info!("Installed single file to: {}", target.display());
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_zip_slip_prevention() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("malicious.zip");
        let extract_dir = temp_dir.path().join("extracted");

        // Create a ZIP with a malicious entry pointing outside destination
        {
            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default();

            zip.start_file("../outside.txt", options).unwrap();
            zip.write_all(b"malicious content").unwrap();
            zip.finish().unwrap();
        }

        // Must reject Zip-Slip entry
        let result =
            SafeExtractor::extract_zip(&zip_path, &extract_dir, false, 1024 * 1024, |_| {});
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Zip-Slip"));
    }

    #[test]
    fn test_safe_zip_extraction() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("valid.zip");
        let extract_dir = temp_dir.path().join("extracted");

        {
            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default();

            zip.start_file("bin/app.exe", options).unwrap();
            zip.write_all(b"binary content").unwrap();
            zip.finish().unwrap();
        }

        let result =
            SafeExtractor::extract_zip(&zip_path, &extract_dir, false, 1024 * 1024, |_| {});
        assert!(result.is_ok());
        assert!(extract_dir.join("bin/app.exe").exists());
    }
}
