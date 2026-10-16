//! Auto-scanner for existing local developer tools (MySQL, PostgreSQL, Nginx, PHP, Node, Bun, Git, Composer, Go).

use crate::error::Result;
use crate::installation::store::{InstalledToolRecord, InstalledToolStore};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub tool_id: String,
    pub display_name: String,
    pub detected_version: String,
    pub directory_path: String,
    pub exe_path: String,
}

pub struct AdoptExistingScanner;

impl AdoptExistingScanner {
    /// Scan entire system for installed local developer tools.
    pub fn scan_all() -> Vec<DiscoveredTool> {
        let mut list = Vec::new();

        Self::scan_tools_dir(&mut list);
        Self::scan_mysql(&mut list);
        Self::scan_postgres(&mut list);
        Self::scan_nginx(&mut list);
        Self::scan_php(&mut list);
        Self::scan_node_npm(&mut list);
        Self::scan_bun(&mut list);
        Self::scan_git(&mut list);
        Self::scan_composer(&mut list);
        Self::scan_go(&mut list);
        Self::scan_mailpit(&mut list);

        list
    }

    /// Auto-adopt discovered tools into the persistent InstalledToolStore if not already registered.
    pub fn adopt_all_discovered(store: &InstalledToolStore) -> Result<usize> {
        let existing = store.load_all().unwrap_or_default();
        let discovered = Self::scan_all();
        let mut adopted_count = 0;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for d in discovered {
            let already_registered = existing
                .iter()
                .any(|e| e.tool_id.eq_ignore_ascii_case(&d.tool_id) && e.is_active);

            if !already_registered {
                let bin_parent = Path::new(&d.exe_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| d.directory_path.clone());

                let record = InstalledToolRecord {
                    tool_id: d.tool_id.clone(),
                    display_name: d.display_name.clone(),
                    version: d.detected_version.clone(),
                    install_path: d.directory_path.clone(),
                    is_adopted: true,
                    is_active: true,
                    added_path_entries: vec![bin_parent],
                    installed_at_secs: now,
                };

                let _ = store.register_tool(record);
                adopted_count += 1;
                tracing::info!("Adopted local tool: {} v{}", d.display_name, d.detected_version);
            }
        }

        Ok(adopted_count)
    }

    fn scan_tools_dir(list: &mut Vec<DiscoveredTool>) {
        let root = Path::new(r"C:\tools");
        if !root.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_lowercase();

                // C:\tools\php*
                if name.starts_with("php") {
                    let php_exe = path.join("php.exe");
                    if php_exe.is_file() {
                        let ver = detect_version(&php_exe, "-v")
                            .unwrap_or_else(|| name.replace("php", ""));
                        list.push(DiscoveredTool {
                            tool_id: "php".into(),
                            display_name: format!("PHP ({name})"),
                            detected_version: ver,
                            directory_path: path.to_string_lossy().to_string(),
                            exe_path: php_exe.to_string_lossy().to_string(),
                        });
                    }
                }

                // C:\tools\nginx*
                if name.contains("nginx") {
                    let nginx_exe = path.join("nginx.exe");
                    if nginx_exe.is_file() {
                        let ver = detect_version(&nginx_exe, "-v")
                            .unwrap_or_else(|| "1.x".to_string());
                        list.push(DiscoveredTool {
                            tool_id: "nginx".into(),
                            display_name: "Nginx Web Server".into(),
                            detected_version: ver,
                            directory_path: path.to_string_lossy().to_string(),
                            exe_path: nginx_exe.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn scan_mysql(list: &mut Vec<DiscoveredTool>) {
        if list.iter().any(|t| t.tool_id == "mysql") {
            return;
        }

        let candidates = [
            PathBuf::from(r"C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe"),
            PathBuf::from(r"C:\Program Files\MySQL\MySQL Server 8.0\bin\mysqld.exe"),
            PathBuf::from(r"C:\tools\mysql\bin\mysqld.exe"),
        ];

        for c in &candidates {
            if c.is_file() {
                let ver = detect_version(c, "--version").unwrap_or_else(|| "8.4".into());
                let dir = c.parent().and_then(|p| p.parent()).unwrap_or(c.parent().unwrap());
                list.push(DiscoveredTool {
                    tool_id: "mysql".into(),
                    display_name: "MySQL Database Server".into(),
                    detected_version: ver,
                    directory_path: dir.to_string_lossy().to_string(),
                    exe_path: c.to_string_lossy().to_string(),
                });
                return;
            }
        }

        if let Ok(path) = which::which("mysqld.exe") {
            let ver = detect_version(&path, "--version").unwrap_or_else(|| "8.x".into());
            let dir = path.parent().and_then(|p| p.parent()).unwrap_or(path.parent().unwrap());
            list.push(DiscoveredTool {
                tool_id: "mysql".into(),
                display_name: "MySQL Database Server".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_postgres(list: &mut Vec<DiscoveredTool>) {
        if list.iter().any(|t| t.tool_id == "postgres") {
            return;
        }

        let base_dir = Path::new(r"C:\Program Files\PostgreSQL");
        if base_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    let pg_exe = entry.path().join("bin").join("postgres.exe");
                    if pg_exe.is_file() {
                        let ver = detect_version(&pg_exe, "--version")
                            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
                        list.push(DiscoveredTool {
                            tool_id: "postgres".into(),
                            display_name: "PostgreSQL Database".into(),
                            detected_version: ver,
                            directory_path: entry.path().to_string_lossy().to_string(),
                            exe_path: pg_exe.to_string_lossy().to_string(),
                        });
                        return;
                    }
                }
            }
        }

        if let Ok(path) = which::which("postgres.exe") {
            let ver = detect_version(&path, "--version").unwrap_or_else(|| "17.x".into());
            let dir = path.parent().and_then(|p| p.parent()).unwrap_or(path.parent().unwrap());
            list.push(DiscoveredTool {
                tool_id: "postgres".into(),
                display_name: "PostgreSQL Database".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_nginx(list: &mut Vec<DiscoveredTool>) {
        if list.iter().any(|t| t.tool_id == "nginx") {
            return;
        }

        let default_path = PathBuf::from(r"C:\tools\nginx\nginx.exe");
        if default_path.is_file() {
            let ver = detect_version(&default_path, "-v").unwrap_or_else(|| "1.30".into());
            let dir = default_path.parent().unwrap_or(Path::new(r"C:\tools\nginx"));
            list.push(DiscoveredTool {
                tool_id: "nginx".into(),
                display_name: "Nginx Web Server".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: default_path.to_string_lossy().to_string(),
            });
            return;
        }

        if let Ok(path) = which::which("nginx.exe") {
            let ver = detect_version(&path, "-v").unwrap_or_else(|| "1.x".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "nginx".into(),
                display_name: "Nginx Web Server".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_php(list: &mut Vec<DiscoveredTool>) {
        if list.iter().any(|t| t.tool_id == "php") {
            return;
        }

        if let Ok(path) = which::which("php.exe") {
            let ver = detect_version(&path, "-v").unwrap_or_else(|| "8.x".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "php".into(),
                display_name: "PHP Hypertext Preprocessor".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_node_npm(list: &mut Vec<DiscoveredTool>) {
        let nvm_path = PathBuf::from(r"C:\nvm4w\nodejs\node.exe");
        let node_exe = if nvm_path.is_file() {
            Some(nvm_path)
        } else {
            which::which("node.exe").ok()
        };

        if let Some(path) = node_exe {
            let ver = detect_version(&path, "-v").unwrap_or_else(|| "Node.js".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "node".into(),
                display_name: "Node.js JavaScript Runtime".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_bun(list: &mut Vec<DiscoveredTool>) {
        let mut candidates = Vec::new();
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            candidates.push(PathBuf::from(profile).join(".bun").join("bin").join("bun.exe"));
        }
        if let Ok(w) = which::which("bun.exe") {
            candidates.push(w);
        }

        for c in candidates {
            if c.is_file() {
                let ver = detect_version(&c, "-v").unwrap_or_else(|| "1.x".into());
                let dir = c.parent().unwrap_or(Path::new("."));
                list.push(DiscoveredTool {
                    tool_id: "bun".into(),
                    display_name: "Bun Runtime & Package Manager".into(),
                    detected_version: ver,
                    directory_path: dir.to_string_lossy().to_string(),
                    exe_path: c.to_string_lossy().to_string(),
                });
                return;
            }
        }
    }

    fn scan_git(list: &mut Vec<DiscoveredTool>) {
        let candidates = [
            PathBuf::from(r"C:\Program Files\Git\cmd\git.exe"),
            PathBuf::from(r"C:\Program Files\Git\bin\git.exe"),
        ];

        for c in &candidates {
            if c.is_file() {
                let ver = detect_version(c, "--version").unwrap_or_else(|| "Git".into());
                let dir = c.parent().unwrap_or(Path::new("."));
                list.push(DiscoveredTool {
                    tool_id: "git".into(),
                    display_name: "Git Version Control".into(),
                    detected_version: ver,
                    directory_path: dir.to_string_lossy().to_string(),
                    exe_path: c.to_string_lossy().to_string(),
                });
                return;
            }
        }

        if let Ok(path) = which::which("git.exe") {
            let ver = detect_version(&path, "--version").unwrap_or_else(|| "Git".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "git".into(),
                display_name: "Git Version Control".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_composer(list: &mut Vec<DiscoveredTool>) {
        let candidates = [
            PathBuf::from(r"C:\ProgramData\ComposerSetup\bin\composer.bat"),
            PathBuf::from(r"C:\ProgramData\ComposerSetup\bin\composer"),
        ];

        for c in &candidates {
            if c.is_file() {
                let ver = detect_version(c, "--version").unwrap_or_else(|| "2.x".into());
                let dir = c.parent().unwrap_or(Path::new("."));
                list.push(DiscoveredTool {
                    tool_id: "composer".into(),
                    display_name: "Composer Dependency Manager".into(),
                    detected_version: ver,
                    directory_path: dir.to_string_lossy().to_string(),
                    exe_path: c.to_string_lossy().to_string(),
                });
                return;
            }
        }

        if let Ok(path) = which::which("composer.bat").or_else(|_| which::which("composer")) {
            let ver = detect_version(&path, "--version").unwrap_or_else(|| "2.x".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "composer".into(),
                display_name: "Composer Dependency Manager".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_go(list: &mut Vec<DiscoveredTool>) {
        let default_path = PathBuf::from(r"C:\Program Files\Go\bin\go.exe");
        if default_path.is_file() {
            let ver = detect_version(&default_path, "version").unwrap_or_else(|| "Go".into());
            let dir = default_path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "go".into(),
                display_name: "Go Programming Language".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: default_path.to_string_lossy().to_string(),
            });
            return;
        }

        if let Ok(path) = which::which("go.exe") {
            let ver = detect_version(&path, "version").unwrap_or_else(|| "Go".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "go".into(),
                display_name: "Go Programming Language".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }

    fn scan_mailpit(list: &mut Vec<DiscoveredTool>) {
        let candidates = [
            r"D:\Apps\Mailpit\mailpit.exe",
            r"C:\Apps\Mailpit\mailpit.exe",
            r"C:\tools\mailpit\mailpit.exe",
            r"C:\Program Files\Mailpit\mailpit.exe",
        ];

        for c_str in candidates {
            let p = PathBuf::from(c_str);
            if p.is_file() {
                let ver = detect_version(&p, "version").unwrap_or_else(|| "1.27".into());
                let dir = p.parent().unwrap_or(Path::new("."));
                list.push(DiscoveredTool {
                    tool_id: "mailpit".into(),
                    display_name: "Mailpit SMTP & Web Inbox".into(),
                    detected_version: ver,
                    directory_path: dir.to_string_lossy().to_string(),
                    exe_path: p.to_string_lossy().to_string(),
                });
                return;
            }
        }

        if let Ok(path) = which::which("mailpit.exe").or_else(|_| which::which("mailpit")) {
            let ver = detect_version(&path, "version").unwrap_or_else(|| "1.x".into());
            let dir = path.parent().unwrap_or(Path::new("."));
            list.push(DiscoveredTool {
                tool_id: "mailpit".into(),
                display_name: "Mailpit SMTP & Web Inbox".into(),
                detected_version: ver,
                directory_path: dir.to_string_lossy().to_string(),
                exe_path: path.to_string_lossy().to_string(),
            });
        }
    }
}

fn detect_version(exe: &Path, arg: &str) -> Option<String> {
    let mut cmd = Command::new(exe);
    cmd.arg(arg);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    // Extract first version-like token (e.g. 8.4.11, v24.13.1, 1.30.4)
    let re = regex::Regex::new(r"(\d+\.\d+(\.\d+)?)").ok()?;
    let m = re.find(&text)?;
    Some(m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_all_does_not_panic() {
        let tools = AdoptExistingScanner::scan_all();
        // Should find installed system tools without panics
        assert!(tools.iter().all(|t| !t.tool_id.is_empty()));
    }
}
