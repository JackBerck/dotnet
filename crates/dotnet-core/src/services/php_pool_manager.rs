//! PHP FastCGI Multi-Worker Pool Manager.
//! Manages sequential FastCGI worker processes and Nginx upstream configuration.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use crate::services::process_tracker::ProcessTracker;
use dotnet_windows::{get_active_tcp_listeners, is_pid_alive, terminate_pid};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhpWorkerInfo {
    pub port: u16,
    pub pid: Option<u32>,
    pub is_running: bool,
}

pub struct PhpPoolManager {
    upstream_conf_path: PathBuf,
}

impl Default for PhpPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpPoolManager {
    #[must_use]
    pub fn new() -> Self {
        let upstream_conf_path = AppPaths::get_path("nginx").join("upstream_php.conf");
        Self { upstream_conf_path }
    }

    #[must_use]
    pub fn with_path(upstream_conf_path: PathBuf) -> Self {
        Self { upstream_conf_path }
    }

    #[must_use]
    pub fn upstream_conf_path(&self) -> &Path {
        &self.upstream_conf_path
    }

    /// Generate Nginx upstream block string for multi-worker PHP pool.
    #[must_use]
    pub fn generate_upstream_block(pool_size: usize, base_port: u16) -> String {
        let clamped_size = pool_size.clamp(1, 8);
        let mut out = String::with_capacity(256);
        out.push_str("# >>> Dotnet PHP FastCGI Pool >>>\n");
        out.push_str("upstream php_pool {\n");
        for i in 0..clamped_size {
            let port = base_port + i as u16;
            out.push_str(&format!("    server 127.0.0.1:{port};\n"));
        }
        out.push_str("}\n");
        out.push_str("# <<< Dotnet PHP FastCGI Pool <<<\n");
        out
    }

    /// Save the upstream block configuration file to disk.
    pub fn save_upstream_config(&self, pool_size: usize, base_port: u16) -> Result<()> {
        if let Some(parent) = self.upstream_conf_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = Self::generate_upstream_block(pool_size, base_port);
        fs::write(&self.upstream_conf_path, content)?;
        tracing::info!(
            "PhpPoolManager: Generated PHP upstream configuration for {} workers at {}",
            pool_size,
            self.upstream_conf_path.display()
        );
        Ok(())
    }

    /// Query the running status of workers in the pool.
    pub fn get_pool_status(&self, pool_size: usize, base_port: u16) -> Result<Vec<PhpWorkerInfo>> {
        let clamped_size = pool_size.clamp(1, 8);
        let listeners = get_active_tcp_listeners()
            .map_err(|e| CoreError::Service(format!("Failed to query TCP listeners: {e}")))?;

        let mut workers = Vec::with_capacity(clamped_size);
        for i in 0..clamped_size {
            let port = base_port + i as u16;
            let active = listeners.iter().find(|l| l.port == port);
            let (pid, is_running) = match active {
                Some(entry) => (Some(entry.owning_pid), is_pid_alive(entry.owning_pid)),
                None => (None, false),
            };
            workers.push(PhpWorkerInfo {
                port,
                pid,
                is_running,
            });
        }

        Ok(workers)
    }

    /// Start a pool of PHP FastCGI worker processes.
    pub fn start_pool(
        &self,
        php_cgi_path: &Path,
        pool_size: usize,
        base_port: u16,
    ) -> Result<Vec<PhpWorkerInfo>> {
        if !php_cgi_path.is_file() {
            return Err(CoreError::Config(format!(
                "php-cgi executable not found at {}",
                php_cgi_path.display()
            )));
        }

        let clamped_size = pool_size.clamp(1, 8);
        self.save_upstream_config(clamped_size, base_port)?;

        let tracker = ProcessTracker::new();
        let listeners = get_active_tcp_listeners().unwrap_or_default();

        for i in 0..clamped_size {
            let port = base_port + i as u16;
            let service_id = if i == 0 {
                "php-cgi".to_string()
            } else {
                format!("php-cgi-{port}")
            };

            // Check if already running on this port
            if let Some(entry) = listeners.iter().find(|l| l.port == port) {
                if is_pid_alive(entry.owning_pid) {
                    tracing::info!(
                        "PhpPoolManager: Worker on port {port} already running (PID {})",
                        entry.owning_pid
                    );
                    let _ = tracker.record_process(&service_id, entry.owning_pid, php_cgi_path);
                    continue;
                }
            }

            // Spawn php-cgi worker
            let bind_arg = format!("127.0.0.1:{port}");
            match Command::new(php_cgi_path)
                .args(["-b", &bind_arg])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
            {
                Ok(child) => {
                    let pid = child.id();
                    let _ = tracker.record_process(&service_id, pid, php_cgi_path);
                    tracing::info!("PhpPoolManager: Spawned PHP worker on port {port} (PID {pid})");
                }
                Err(e) => {
                    tracing::error!(
                        "PhpPoolManager: Failed to spawn PHP worker on port {port}: {e}"
                    );
                }
            }
        }

        self.get_pool_status(clamped_size, base_port)
    }

    /// Stop all workers in the PHP FastCGI pool.
    pub fn stop_pool(&self, pool_size: usize, base_port: u16) -> Result<()> {
        let clamped_size = pool_size.clamp(1, 8);
        let tracker = ProcessTracker::new();
        let listeners = get_active_tcp_listeners().unwrap_or_default();

        for i in 0..clamped_size {
            let port = base_port + i as u16;
            let service_id = if i == 0 {
                "php-cgi".to_string()
            } else {
                format!("php-cgi-{port}")
            };

            // 1. Terminate tracked PID if alive
            if let Ok(Some(rec)) = tracker.get_process(&service_id) {
                if is_pid_alive(rec.pid) {
                    let _ = terminate_pid(rec.pid);
                }
                let _ = tracker.remove_process(&service_id);
            }

            // 2. Terminate any remaining listener on the port
            if let Some(entry) = listeners.iter().find(|l| l.port == port) {
                if is_pid_alive(entry.owning_pid) {
                    let _ = terminate_pid(entry.owning_pid);
                }
            }
        }

        tracing::info!("PhpPoolManager: Stopped PHP pool ({clamped_size} workers)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_upstream_block() {
        let block = PhpPoolManager::generate_upstream_block(3, 9000);
        assert!(block.contains("upstream php_pool {"));
        assert!(block.contains("server 127.0.0.1:9000;"));
        assert!(block.contains("server 127.0.0.1:9001;"));
        assert!(block.contains("server 127.0.0.1:9002;"));
        assert!(!block.contains("server 127.0.0.1:9003;"));
    }

    #[test]
    fn test_save_upstream_config() {
        let dir = tempdir().expect("tempdir");
        let conf_path = dir.path().join("upstream_php.conf");
        let manager = PhpPoolManager::with_path(conf_path.clone());

        manager.save_upstream_config(2, 9000).expect("save");
        assert!(conf_path.exists());

        let content = fs::read_to_string(&conf_path).expect("read");
        assert!(content.contains("server 127.0.0.1:9000;"));
        assert!(content.contains("server 127.0.0.1:9001;"));
    }
}
