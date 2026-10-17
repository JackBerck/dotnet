//! Windows Start Menu shortcut manager for seamless search & OS integration.

use crate::error::{Result, WindowsError};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct ShortcutManager;

impl ShortcutManager {
    /// Return the path to the user's Start Menu Programs folder.
    #[must_use]
    pub fn get_start_menu_programs_dir() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
        })
    }

    /// Check if the shortcut is already installed in the Start Menu Programs directory.
    #[must_use]
    pub fn is_start_menu_shortcut_installed(shortcut_name: &str) -> bool {
        Self::get_start_menu_programs_dir()
            .map(|dir| dir.join(format!("{shortcut_name}.lnk")).exists())
            .unwrap_or(false)
    }

    /// Install or update the ".lnk" shortcut in Start Menu.
    pub fn create_or_update_start_menu_shortcut(
        shortcut_name: &str,
        target_exe: &Path,
        icon_path: Option<&Path>,
        description: &str,
    ) -> Result<PathBuf> {
        let programs_dir = Self::get_start_menu_programs_dir().ok_or_else(|| {
            WindowsError::SystemError("Could not resolve APPDATA directory".to_string())
        })?;

        if !programs_dir.exists() {
            let _ = std::fs::create_dir_all(&programs_dir);
        }

        let link_path = programs_dir.join(format!("{shortcut_name}.lnk"));
        let working_dir = target_exe.parent().unwrap_or(target_exe);

        let icon_arg = if let Some(ico) = icon_path {
            format!("$s.IconLocation = '{}'; ", ico.display())
        } else {
            format!("$s.IconLocation = '{},0'; ", target_exe.display())
        };

        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; \
             $s = $ws.CreateShortcut('{}'); \
             $s.TargetPath = '{}'; \
             $s.WorkingDirectory = '{}'; \
             $s.Description = '{}'; \
             {} \
             $s.Save()",
            link_path.display(),
            target_exe.display(),
            working_dir.display(),
            description,
            icon_arg
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() {
            Ok(link_path)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(WindowsError::SystemError(format!(
                "Failed to create shortcut: {err}"
            )))
        }
    }

    /// Remove the shortcut from Start Menu.
    pub fn remove_start_menu_shortcut(shortcut_name: &str) -> Result<()> {
        if let Some(programs_dir) = Self::get_start_menu_programs_dir() {
            let link_path = programs_dir.join(format!("{shortcut_name}.lnk"));
            if link_path.exists() {
                std::fs::remove_file(link_path)?;
            }
        }
        Ok(())
    }

    /// Return the path to the user's Startup folder.
    #[must_use]
    pub fn get_startup_dir() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup")
        })
    }

    /// Check if the shortcut is installed in Startup folder.
    #[must_use]
    pub fn is_startup_shortcut_installed(shortcut_name: &str) -> bool {
        Self::get_startup_dir()
            .map(|dir| dir.join(format!("{shortcut_name}.lnk")).exists())
            .unwrap_or(false)
    }

    /// Install or update shortcut in Startup folder with optional arguments.
    pub fn create_or_update_startup_shortcut(
        shortcut_name: &str,
        target_exe: &Path,
        arguments: &str,
        icon_path: Option<&Path>,
        description: &str,
    ) -> Result<PathBuf> {
        let startup_dir = Self::get_startup_dir().ok_or_else(|| {
            WindowsError::SystemError("Could not resolve APPDATA directory".to_string())
        })?;

        if !startup_dir.exists() {
            let _ = std::fs::create_dir_all(&startup_dir);
        }

        let link_path = startup_dir.join(format!("{shortcut_name}.lnk"));
        let working_dir = target_exe.parent().unwrap_or(target_exe);

        let icon_arg = if let Some(ico) = icon_path {
            format!("$s.IconLocation = '{}'; ", ico.display())
        } else {
            format!("$s.IconLocation = '{},0'; ", target_exe.display())
        };

        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; \
             $s = $ws.CreateShortcut('{}'); \
             $s.TargetPath = '{}'; \
             $s.Arguments = '{}'; \
             $s.WorkingDirectory = '{}'; \
             $s.Description = '{}'; \
             {} \
             $s.Save()",
            link_path.display(),
            target_exe.display(),
            arguments,
            working_dir.display(),
            description,
            icon_arg
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        if output.status.success() {
            Ok(link_path)
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(WindowsError::SystemError(format!(
                "Failed to create startup shortcut: {err}"
            )))
        }
    }

    /// Remove shortcut from Startup folder.
    pub fn remove_startup_shortcut(shortcut_name: &str) -> Result<()> {
        if let Some(startup_dir) = Self::get_startup_dir() {
            let link_path = startup_dir.join(format!("{shortcut_name}.lnk"));
            if link_path.exists() {
                std::fs::remove_file(link_path)?;
            }
        }
        Ok(())
    }
}
