//! Unified Windows Autostart Manager
//! Handles Task Scheduler (UAC-free elevated background boot), Registry Run key, and Startup folder shortcut.

use crate::error::Result;
use crate::registry;
use crate::shortcut::ShortcutManager;
use crate::task_scheduler::TaskScheduler;
use std::path::Path;

pub const AUTOSTART_TASK_NAME: &str = "DotnetAutostart";
pub const AUTOSTART_REG_KEY: &str = "Dotnet";
pub const AUTOSTART_SHORTCUT_NAME: &str = "Dotnet";

pub struct AutostartManager;

impl AutostartManager {
    /// Check whether any autostart mechanism is currently enabled.
    #[must_use]
    pub fn is_autostart_enabled() -> bool {
        TaskScheduler::is_task_scheduled(AUTOSTART_TASK_NAME).unwrap_or(false)
            || registry::get_run_key(AUTOSTART_REG_KEY)
                .map(|cmd| !cmd.is_empty())
                .unwrap_or(false)
            || ShortcutManager::is_startup_shortcut_installed(AUTOSTART_SHORTCUT_NAME)
    }

    /// Enable autostart with `--minimized` so Dotnet starts cleanly in background on boot.
    pub fn enable_autostart(target_exe: &Path, icon_path: Option<&Path>) -> Result<()> {
        let exe_str = target_exe.to_string_lossy();
        let run_cmd = format!("\"{}\" --minimized", exe_str);

        // 1. If elevated, register Task Scheduler logon task (highest privileges, no UAC on boot)
        if crate::process::is_elevated() {
            let _ = TaskScheduler::create_logon_task(AUTOSTART_TASK_NAME, &exe_str, "--minimized");
        }

        // 2. Set Registry Run key so Windows Settings -> Apps -> Startup recognizes Dotnet
        let _ = registry::set_run_key(AUTOSTART_REG_KEY, &run_cmd);

        // 3. Create shortcut in user's Startup folder
        let _ = ShortcutManager::create_or_update_startup_shortcut(
            AUTOSTART_SHORTCUT_NAME,
            target_exe,
            "--minimized",
            icon_path,
            "Dotnet",
        );

        Ok(())
    }

    /// Disable autostart across all mechanisms.
    pub fn disable_autostart() -> Result<()> {
        let _ = TaskScheduler::delete_logon_task(AUTOSTART_TASK_NAME);
        let _ = registry::delete_run_key(AUTOSTART_REG_KEY);
        let _ = ShortcutManager::remove_startup_shortcut(AUTOSTART_SHORTCUT_NAME);
        Ok(())
    }
}
