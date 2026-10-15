//! Project Manager - Manages local dev projects, framework detection, and developer tool launchers.

use crate::error::{CoreError, Result};
use crate::models::ProjectInfo;
use crate::persistence::{AppPaths, JsonStore};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ProjectManager {
    state_file: PathBuf,
    legacy_file: PathBuf,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectManager {
    #[must_use]
    pub fn new() -> Self {
        let state_file = AppPaths::get_path("state").join("projects.json");
        let legacy_file = AppPaths::get_path("projects.json");
        Self {
            state_file,
            legacy_file,
        }
    }

    #[must_use]
    pub fn with_paths(state_file: PathBuf, legacy_file: PathBuf) -> Self {
        Self {
            state_file,
            legacy_file,
        }
    }

    /// Load all projects from disk, migrating legacy projects.json if present.
    pub fn get_projects(&self) -> Result<Vec<ProjectInfo>> {
        // Migration check
        if !self.state_file.exists() && self.legacy_file.exists() {
            if let Ok(Some(legacy_projects)) =
                JsonStore::load::<Vec<ProjectInfo>, _>(&self.legacy_file)
            {
                tracing::info!(
                    "ProjectManager: Migrating {} projects from legacy path {:?}",
                    legacy_projects.len(),
                    self.legacy_file
                );
                let _ = JsonStore::save_atomic(&self.state_file, &legacy_projects);
                return Ok(legacy_projects);
            }
        }

        let projects =
            JsonStore::load::<Vec<ProjectInfo>, _>(&self.state_file)?.unwrap_or_default();
        Ok(projects)
    }

    /// Persist the full list of projects atomically.
    pub fn save_projects(&self, projects: &[ProjectInfo]) -> Result<()> {
        JsonStore::save_atomic(&self.state_file, projects)
    }

    /// Add a new project or update an existing one matched by ID or path.
    pub fn add_or_update_project(&self, project: ProjectInfo) -> Result<()> {
        let mut projects = self.get_projects()?;
        if let Some(pos) = projects.iter().position(|p| {
            p.id == project.id
                || (!project.path.is_empty() && p.path.eq_ignore_ascii_case(&project.path))
        }) {
            projects[pos] = project;
        } else {
            projects.push(project);
        }
        self.save_projects(&projects)
    }

    /// Find a project by exact ID.
    pub fn get_project_by_id(&self, id: &str) -> Result<Option<ProjectInfo>> {
        let projects = self.get_projects()?;
        Ok(projects.into_iter().find(|p| p.id.eq_ignore_ascii_case(id)))
    }

    /// Find a project by ID, Name, or Domain.
    pub fn get_project_by_identifier(&self, ident: &str) -> Result<Option<ProjectInfo>> {
        let projects = self.get_projects()?;
        Ok(projects.into_iter().find(|p| {
            p.id.eq_ignore_ascii_case(ident)
                || p.name.eq_ignore_ascii_case(ident)
                || p.nginx_host
                    .as_deref()
                    .is_some_and(|h| h.eq_ignore_ascii_case(ident))
        }))
    }

    /// Delete a project by ID or Name. Returns true if removed.
    pub fn delete_project(&self, ident: &str) -> Result<bool> {
        let mut projects = self.get_projects()?;
        let orig_len = projects.len();
        projects.retain(|p| {
            !p.id.eq_ignore_ascii_case(ident)
                && !p.name.eq_ignore_ascii_case(ident)
                && !p
                    .nginx_host
                    .as_deref()
                    .is_some_and(|h| h.eq_ignore_ascii_case(ident))
        });

        if projects.len() < orig_len {
            self.save_projects(&projects)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Launch configured dev command in an external interactive Windows Command Prompt.
    pub fn launch_dev_command(project: &ProjectInfo) -> Result<()> {
        let proj_path = Path::new(&project.path);
        if !proj_path.is_dir() {
            return Err(CoreError::Config(format!(
                "Project directory does not exist: {}",
                project.path
            )));
        }

        let dev_cmd = match &project.dev_command {
            Some(cmd) if !cmd.trim().is_empty() => cmd.trim(),
            _ => {
                return Err(CoreError::Config(format!(
                    "No dev command configured for project '{}'",
                    project.name
                )))
            }
        };

        let cmd_arg = format!("cd /d \"{}\" && {}", project.path, dev_cmd);
        Command::new("cmd.exe")
            .args(["/k", &cmd_arg])
            .current_dir(proj_path)
            .spawn()?;

        tracing::info!(
            "ProjectManager: Launched dev command '{}' for project '{}'",
            dev_cmd,
            project.name
        );
        Ok(())
    }

    /// Open directory in Windows File Explorer.
    pub fn open_in_explorer(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(CoreError::Config(format!(
                "Path is not a valid directory: {}",
                path.display()
            )));
        }

        Command::new("explorer.exe").arg(path.as_os_str()).spawn()?;
        Ok(())
    }

    /// Open directory in Windows Terminal (wt.exe) or fallback to PowerShell.
    pub fn open_terminal(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(CoreError::Config(format!(
                "Path is not a valid directory: {}",
                path.display()
            )));
        }

        // Try Windows Terminal first
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let wt_path = PathBuf::from(local_app_data)
                .join("Microsoft")
                .join("WindowsApps")
                .join("wt.exe");

            if wt_path.is_file() {
                if let Ok(_child) = Command::new(&wt_path)
                    .args(["-d", &path.to_string_lossy()])
                    .spawn()
                {
                    return Ok(());
                }
            }
        }

        // Fallback to PowerShell
        let ps_cmd = format!("Set-Location '{}'", path.to_string_lossy());
        Command::new("powershell.exe")
            .args(["-NoExit", "-Command", &ps_cmd])
            .current_dir(path)
            .spawn()?;

        Ok(())
    }

    /// Open directory in Visual Studio Code.
    pub fn open_in_vscode(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Err(CoreError::Config(format!(
                "Path is not a valid directory: {}",
                path.display()
            )));
        }

        // Try code.cmd or code.exe via PATH
        let code_bin = which::which("code.cmd")
            .or_else(|_| which::which("code.exe"))
            .or_else(|_| which::which("code"))
            .unwrap_or_else(|_| PathBuf::from("code.cmd"));

        Command::new(code_bin).arg(".").current_dir(path).spawn()?;

        Ok(())
    }

    /// Open project .env file in Notepad.
    pub fn open_env_file(path: &Path) -> Result<()> {
        let env_file = path.join(".env");
        if !env_file.is_file() {
            return Err(CoreError::Config(format!(
                ".env file does not exist at {}",
                env_file.display()
            )));
        }

        Command::new("notepad.exe")
            .arg(env_file.as_os_str())
            .spawn()?;
        Ok(())
    }

    /// Open URL or host in default web browser.
    pub fn open_browser(host_or_url: &str) -> Result<()> {
        let trimmed = host_or_url.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Config("URL is empty".to_string()));
        }

        let full_url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else {
            format!("http://{trimmed}")
        };

        Command::new("cmd.exe")
            .args(["/c", "start", "", &full_url])
            .spawn()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_project_manager_crud() {
        let dir = tempdir().expect("tempdir");
        let state_file = dir.path().join("state").join("projects.json");
        let legacy_file = dir.path().join("projects.json");

        let manager = ProjectManager::with_paths(state_file.clone(), legacy_file);

        // Initially empty
        let list = manager.get_projects().expect("get");
        assert!(list.is_empty());

        // Add project
        let mut proj1 = ProjectInfo::new("my-app", "D:/code/my-app");
        proj1.id = "proj-1".to_string();
        proj1.nginx_host = Some("my-app.test".to_string());
        manager.add_or_update_project(proj1.clone()).expect("add");

        let list = manager.get_projects().expect("get");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "my-app");

        // Lookup by identifier
        let found = manager
            .get_project_by_identifier("my-app.test")
            .expect("find")
            .expect("must exist");
        assert_eq!(found.id, "proj-1");

        // Update project
        proj1.dev_command = Some("npm run dev".to_string());
        manager.add_or_update_project(proj1).expect("update");

        let updated = manager
            .get_project_by_id("proj-1")
            .expect("find")
            .expect("must exist");
        assert_eq!(updated.dev_command, Some("npm run dev".to_string()));

        // Delete project
        assert!(manager.delete_project("my-app").expect("delete"));
        assert_eq!(manager.get_projects().expect("get").len(), 0);
    }

    #[test]
    fn test_project_manager_legacy_migration() {
        let dir = tempdir().expect("tempdir");
        let state_file = dir.path().join("state").join("projects.json");
        let legacy_file = dir.path().join("projects.json");

        // Prepopulate legacy file
        let legacy_data = vec![ProjectInfo::new("old-app", "C:/old-app")];
        JsonStore::save_atomic(&legacy_file, &legacy_data).expect("save legacy");

        let manager = ProjectManager::with_paths(state_file.clone(), legacy_file);
        let list = manager.get_projects().expect("get");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "old-app");

        // Confirm migrated to state_file
        assert!(state_file.exists());
    }
}
