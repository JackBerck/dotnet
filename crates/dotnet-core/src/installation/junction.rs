//! Windows NTFS directory junction manager.

use crate::error::{CoreError, Result};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct JunctionManager;

impl JunctionManager {
    /// Create or update an NTFS directory junction (`current` -> `version`).
    pub fn create_or_update_junction<J: AsRef<Path>, T: AsRef<Path>>(
        junction_path: J,
        target_dir: T,
    ) -> Result<()> {
        let junction = junction_path.as_ref();
        let target = target_dir.as_ref();

        if !target.exists() {
            return Err(CoreError::Validation(format!(
                "Junction target does not exist: {}",
                target.display()
            )));
        }

        // Remove existing junction if already present
        if junction.exists() {
            Self::delete_junction(junction)?;
        }

        if let Some(parent) = junction.parent() {
            fs::create_dir_all(parent)?;
        }

        // Run mklink /J (non-elevated junction creation)
        let output = Command::new("cmd.exe")
            .args([
                "/c",
                "mklink",
                "/J",
                &junction.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() && junction.exists() {
            tracing::info!(
                "Junction created: {} -> {}",
                junction.display(),
                target.display()
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(CoreError::Validation(format!(
                "Failed to create junction: {} {}",
                stderr, stdout
            )))
        }
    }

    /// Delete a directory junction without removing target directory contents.
    pub fn delete_junction<P: AsRef<Path>>(junction_path: P) -> Result<()> {
        let path = junction_path.as_ref();
        if !path.exists() {
            return Ok(());
        }

        // Directory::remove_dir removes the link on Windows without deleting contents
        if fs::remove_dir(path).is_ok() {
            return Ok(());
        }

        // Fallback: rmdir command
        let output = Command::new("cmd.exe")
            .args(["/c", "rmdir", &path.to_string_lossy()])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() || !path.exists() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CoreError::Validation(format!(
                "Failed to delete junction: {stderr}"
            )))
        }
    }
}
