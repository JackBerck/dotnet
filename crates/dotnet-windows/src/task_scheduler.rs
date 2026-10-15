//! Windows Task Scheduler integration for elevated autostart (UAC bypass on logon).

use crate::error::{Result, WindowsError};
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const DEFAULT_TASK_NAME: &str = "DotnetElevatedStartup";

pub struct TaskScheduler;

impl TaskScheduler {
    /// Check whether a scheduled task exists.
    pub fn is_task_scheduled(task_name: &str) -> Result<bool> {
        let output = Command::new("schtasks.exe")
            .args(["/Query", "/TN", task_name])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        Ok(output.status.success())
    }

    /// Create an ONLOGON task with HIGHEST privileges.
    pub fn create_logon_task(task_name: &str, exe_path: &str, arguments: &str) -> Result<()> {
        let task_run = format!("\"{}\" {}", exe_path, arguments).trim().to_string();

        let output = Command::new("schtasks.exe")
            .args([
                "/Create", "/TN", task_name, "/TR", &task_run, "/SC", "ONLOGON", "/RL", "HIGHEST",
                "/F",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(WindowsError::ProcessError(format!(
                "Failed to create scheduled task '{}': {} {}",
                task_name, stderr, stdout
            )))
        }
    }

    /// Delete a scheduled task.
    pub fn delete_logon_task(task_name: &str) -> Result<()> {
        let output = Command::new("schtasks.exe")
            .args(["/Delete", "/TN", task_name, "/F"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(WindowsError::ProcessError(format!(
                "Failed to delete scheduled task '{}': {}",
                task_name, stderr
            )))
        }
    }
}
