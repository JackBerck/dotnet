//! Managed child process lifecycle supervisor with safe termination and non-blocking logging.

use crate::error::{CoreError, Result};
use crate::models::{DevServiceDescriptor, ServiceStatus};
use crate::persistence::AppPaths;
use dotnet_windows::{get_owner_pid_for_port, is_pid_alive, terminate_pid};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

pub struct TrackedProcess {
    pub service_id: String,
    pub pid: u32,
    pub executable_path: PathBuf,
}

#[derive(Clone)]
pub struct ProcessSupervisor {
    tracked: Arc<RwLock<HashMap<String, TrackedProcess>>>,
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self {
            tracked: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Query service status: Running, Stopped, PortConflict, NotInstalled.
    pub fn get_service_status(&self, service: &DevServiceDescriptor) -> ServiceStatus {
        let lock = self.tracked.read();

        // 1. Check if tracked by Dotnet
        if let Some(tracked) = lock.get(&service.id) {
            if is_pid_alive(tracked.pid) {
                return ServiceStatus::Running;
            }
        }
        drop(lock);

        // 2. Check executable presence
        if let Some(bin) = &service.binary_name {
            let candidate = AppPaths::tools_dir()
                .join(&service.id)
                .join("current")
                .join(bin);
            if !candidate.exists() && which::which(bin).is_err() {
                return ServiceStatus::NotInstalled;
            }
        }

        // 3. Check port usage
        if service.port > 0 {
            if let Ok(Some(pid)) = get_owner_pid_for_port(service.port) {
                let lock = self.tracked.read();
                if let Some(tracked) = lock.get(&service.id) {
                    if tracked.pid == pid {
                        return ServiceStatus::Running;
                    }
                }
                return ServiceStatus::PortConflict;
            }
        }

        ServiceStatus::Stopped
    }

    /// Start a service process with non-blocking stdout/stderr log redirection.
    pub async fn start_service(
        &self,
        service: &DevServiceDescriptor,
        exe_path: PathBuf,
        work_dir: Option<PathBuf>,
    ) -> Result<u32> {
        if !exe_path.exists() {
            return Err(CoreError::Service(format!(
                "Executable not found at {}",
                exe_path.display()
            )));
        }

        // Check if port is already taken
        if service.port > 0 {
            if let Ok(Some(pid)) = get_owner_pid_for_port(service.port) {
                return Err(CoreError::Service(format!(
                    "Cannot start {}: port {} is already in use by PID {}",
                    service.name, service.port, pid
                )));
            }
        }

        let logs_dir = AppPaths::get_path("logs");
        fs::create_dir_all(&logs_dir)?;
        let log_file_path = logs_dir.join(format!("{}.log", service.id));

        let mut child = Command::new(&exe_path)
            .args(&service.default_args)
            .current_dir(work_dir.unwrap_or_else(|| {
                exe_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .to_path_buf()
            }))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()?;

        let pid = child.id().ok_or_else(|| {
            CoreError::Service("Failed to retrieve child process PID".to_string())
        })?;

        // Spawn async log streamers for stdout and stderr to prevent pipe buffer hangs
        spawn_log_streamer(child.stdout.take(), log_file_path.clone());
        spawn_log_streamer(child.stderr.take(), log_file_path);

        // Track process
        {
            let mut lock = self.tracked.write();
            lock.insert(
                service.id.clone(),
                TrackedProcess {
                    service_id: service.id.clone(),
                    pid,
                    executable_path: exe_path,
                },
            );
        }

        tracing::info!("Started service '{}' with PID {}", service.name, pid);
        Ok(pid)
    }

    /// Stop a managed service safely (never kill arbitrary foreign processes).
    pub async fn stop_service(&self, service_id: &str) -> Result<()> {
        let tracked = {
            let mut lock = self.tracked.write();
            lock.remove(service_id)
        };

        if let Some(t) = tracked {
            if is_pid_alive(t.pid) {
                terminate_pid(t.pid)?;
                tracing::info!("Stopped service '{}' (PID {})", service_id, t.pid);
            }
        }

        Ok(())
    }
}

fn spawn_log_streamer<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader_opt: Option<R>,
    log_file_path: PathBuf,
) {
    if let Some(reader) = reader_opt {
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file_path)
                {
                    let _ = writeln!(file, "{}", line);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_supervisor_initial_state() {
        let supervisor = ProcessSupervisor::new();
        let svc = DevServiceDescriptor {
            id: "dummy".into(),
            name: "Dummy Service".into(),
            description: "Unit test dummy".into(),
            port: 65530,
            service_type: crate::models::ServiceType::ManagedProcess,
            windows_service_name: None,
            binary_name: Some("nonexistent_binary_xyz.exe".into()),
            default_args: vec![],
            auto_start: false,
        };

        let status = supervisor.get_service_status(&svc);
        assert_eq!(status, ServiceStatus::NotInstalled);
    }
}
