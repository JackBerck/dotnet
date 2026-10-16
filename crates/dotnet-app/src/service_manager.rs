//! Application Service Manager orchestrating all developer runtime services.

use anyhow::{anyhow, Result};
use dotnet_core::models::{DevServiceDescriptor, ServiceStatus, ServiceType};
use dotnet_core::persistence::AppPaths;
use dotnet_core::services::process_service::ProcessSupervisor;
use dotnet_core::services::process_tracker::ProcessTracker;
use dotnet_core::services::profile_store::ProfileStore;
use dotnet_core::services::service_orchestrator::ServiceOrchestrator;
use dotnet_windows::service_ctrl::{
    get_service_status, resolve_installed_service_name, start_service, start_service_elevated,
    stop_service, stop_service_elevated, terminate_pid_elevated, WindowsServiceState,
};
use dotnet_windows::{get_owner_pid_for_port, is_pid_alive, terminate_pid};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceItem {
    pub id: String,
    pub name: String,
    pub is_running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub r#type: String,
}

pub struct AppServiceManager {
    supervisor: ProcessSupervisor,
    tracker: Arc<Mutex<ProcessTracker>>,
    profile_store: Arc<Mutex<ProfileStore>>,
    services: Vec<DevServiceDescriptor>,
    logs: Option<Arc<Mutex<Vec<String>>>>,
}

impl AppServiceManager {
    pub fn new(
        tracker: Arc<Mutex<ProcessTracker>>,
        profile_store: Arc<Mutex<ProfileStore>>,
        logs: Option<Arc<Mutex<Vec<String>>>>,
    ) -> Self {
        let supervisor = ProcessSupervisor::new();
        let services = Self::default_descriptors();
        Self {
            supervisor,
            tracker,
            profile_store,
            services,
            logs,
        }
    }

    async fn log(&self, msg: impl Into<String>) {
        let text = msg.into();
        info!("{text}");
        if let Some(ref logs) = self.logs {
            let mut l = logs.lock().await;
            let line = format!("[{}] {}", chrono_lite_timestamp(), text);
            l.push(line);
            if l.len() > 1000 {
                l.remove(0);
            }
        }
    }

    pub fn default_descriptors() -> Vec<DevServiceDescriptor> {
        vec![
            DevServiceDescriptor {
                id: "nginx".into(),
                name: "Nginx Web Server".into(),
                description: "Reverse proxy and virtual host manager".into(),
                port: 80,
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("nginx.exe".into()),
                default_args: vec![],
                auto_start: true,
            },
            DevServiceDescriptor {
                id: "php-cgi".into(),
                name: "PHP FastCGI Engine".into(),
                description: "PHP FastCGI Worker Process".into(),
                port: 9000,
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("php-cgi.exe".into()),
                default_args: vec!["-b".into(), "127.0.0.1:9000".into()],
                auto_start: true,
            },
            DevServiceDescriptor {
                id: "mysql".into(),
                name: "MySQL Database Server".into(),
                description: "Relational SQL database".into(),
                port: 3306,
                service_type: ServiceType::WindowsService,
                windows_service_name: Some("MySQL84".into()),
                binary_name: Some("mysqld.exe".into()),
                default_args: vec!["--console".into()],
                auto_start: true,
            },
            DevServiceDescriptor {
                id: "postgres".into(),
                name: "PostgreSQL Database".into(),
                description: "Object-relational database system".into(),
                port: 5432,
                service_type: ServiceType::WindowsService,
                windows_service_name: Some("postgresql-x64-17".into()),
                binary_name: Some("postgres.exe".into()),
                default_args: vec![],
                auto_start: false,
            },
            DevServiceDescriptor {
                id: "redis".into(),
                name: "Redis / Memurai Store".into(),
                description: "High-performance cache and message broker".into(),
                port: 6379,
                service_type: ServiceType::WindowsService,
                windows_service_name: Some("Memurai".into()),
                binary_name: Some("memurai.exe".into()),
                default_args: vec![],
                auto_start: false,
            },
            DevServiceDescriptor {
                id: "mailpit".into(),
                name: "Mailpit SMTP & Web Inbox".into(),
                description: "Local email capture and inspection tool".into(),
                port: 8025,
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("mailpit.exe".into()),
                default_args: vec![],
                auto_start: false,
            },
        ]
    }

    /// Read all service statuses with real-time port inspection & Windows Service integration.
    pub async fn get_all_statuses(&self) -> Vec<ServiceItem> {
        let tracker = self.tracker.lock().await;
        let mut items = Vec::new();

        for s in &self.services {
            let mut is_running = false;
            let mut pid = None;

            // 1. Direct TCP port listener check (most accurate across Windows services & portable tools)
            if s.port > 0 {
                if let Ok(Some(owner_pid)) = get_owner_pid_for_port(s.port) {
                    if is_pid_alive(owner_pid) {
                        is_running = true;
                        pid = Some(owner_pid);
                    }
                }
            }

            // 2. Windows Service SCM check
            if !is_running {
                let resolved_name = resolve_windows_service_name(&s.id, s.windows_service_name.as_deref());
                if let Some(ref svc_name) = resolved_name {
                    let state = get_service_status(svc_name);
                    if state == WindowsServiceState::Running {
                        is_running = true;
                    }
                }
            }

            // 3. Process supervisor check
            if !is_running {
                let status = self.supervisor.get_service_status(s);
                if status == ServiceStatus::Running {
                    is_running = true;
                }
            }

            // 4. Cross-check with persistent tracker
            if !is_running {
                if let Ok(Some(record)) = tracker.get_process(&s.id) {
                    if is_pid_alive(record.pid) {
                        is_running = true;
                        pid = Some(record.pid);
                    }
                }
            }

            items.push(ServiceItem {
                id: s.id.clone(),
                name: s.name.clone(),
                is_running,
                port: s.port,
                pid,
                r#type: match s.service_type {
                    ServiceType::ManagedProcess => "Managed Process".into(),
                    ServiceType::WindowsService => "Windows Service".into(),
                },
            });
        }

        items
    }

    pub fn resolve_binary_path(&self, service: &DevServiceDescriptor) -> Option<PathBuf> {
        let bin_name = service.binary_name.as_deref()?;

        // 1. Check InstalledToolStore (records from adopt_scanner or tool installer)
        if let Ok(Some(tool)) = dotnet_core::installation::store::InstalledToolStore::new().get_active_tool(&service.id) {
            let p = PathBuf::from(&tool.install_path);
            if p.is_file() {
                return Some(p);
            }
            let cand = p.join(bin_name);
            if cand.is_file() {
                return Some(cand);
            }
            let cand_bin = p.join("bin").join(bin_name);
            if cand_bin.is_file() {
                return Some(cand_bin);
            }
        }

        // 2. Check Dotnet managed tools dir (%LOCALAPPDATA%\Dotnet\tools\<id>\current\<bin>)
        let candidate = AppPaths::tools_dir()
            .join(&service.id)
            .join("current")
            .join(bin_name);
        if candidate.is_file() {
            return Some(candidate);
        }

        // 3. Check well-known host locations per service
        let known_candidates: &[&str] = match service.id.as_str() {
            "mailpit" => &[
                r"D:\Apps\Mailpit\mailpit.exe",
                r"C:\Apps\Mailpit\mailpit.exe",
                r"C:\tools\mailpit\mailpit.exe",
                r"D:\tools\mailpit\mailpit.exe",
                r"C:\Program Files\Mailpit\mailpit.exe",
                r"D:\Program Files\Mailpit\mailpit.exe",
            ],
            "mysql" => &[
                r"C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe",
                r"C:\Program Files\MySQL\MySQL Server 8.0\bin\mysqld.exe",
                r"C:\tools\mysql\bin\mysqld.exe",
                r"D:\Apps\MySQL\bin\mysqld.exe",
                r"C:\Apps\MySQL\bin\mysqld.exe",
            ],
            "postgres" => &[
                r"C:\Program Files\PostgreSQL\17\bin\postgres.exe",
                r"C:\Program Files\PostgreSQL\16\bin\postgres.exe",
                r"C:\tools\postgres\bin\postgres.exe",
                r"D:\Apps\PostgreSQL\bin\postgres.exe",
            ],
            "nginx" => &[
                r"C:\tools\nginx\nginx.exe",
                r"C:\nginx\nginx.exe",
                r"D:\Apps\nginx\nginx.exe",
                r"C:\Apps\nginx\nginx.exe",
            ],
            "php-cgi" => &[
                r"C:\tools\php85\php-cgi.exe",
                r"C:\tools\php\php-cgi.exe",
                r"D:\Apps\php\php-cgi.exe",
                r"C:\Apps\php\php-cgi.exe",
            ],
            "redis" => &[
                r"C:\Program Files\Memurai\memurai.exe",
                r"C:\tools\redis\redis-server.exe",
                r"D:\Apps\Redis\redis-server.exe",
            ],
            _ => &[],
        };

        for path_str in known_candidates {
            let p = Path::new(path_str);
            if p.is_file() {
                return Some(p.to_path_buf());
            }
        }

        // 4. Drive scan for D, C, E (\Apps\<id>\<bin_name> or \Apps\<name>\<bin_name>)
        for drive in ['D', 'C', 'E'] {
            let p1 = PathBuf::from(format!(r"{drive}:\Apps\{}\{}", service.id, bin_name));
            if p1.is_file() {
                return Some(p1);
            }
            let p2 = PathBuf::from(format!(r"{drive}:\tools\{}\{}", service.id, bin_name));
            if p2.is_file() {
                return Some(p2);
            }
        }

        // 5. Fallback to system PATH
        which::which(bin_name).ok()
    }

    /// Start a service by ID (handles Windows Services, adopted binaries, and managed processes).
    pub async fn start_service(&self, id: &str) -> Result<()> {
        let svc = self
            .services
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(id))
            .ok_or_else(|| anyhow!("Unknown service '{id}'"))?;

        self.log(format!("[SERVICE] Starting '{}' on port {}...", svc.name, svc.port)).await;

        // 1. Check if already listening on port
        if svc.port > 0 {
            if let Ok(Some(pid)) = get_owner_pid_for_port(svc.port) {
                if is_pid_alive(pid) {
                    self.log(format!(
                        "[SERVICE] '{}' is already active on port {} (PID: {})",
                        svc.name, svc.port, pid
                    )).await;
                    return Ok(());
                }
            }
        }

        // 2. Try Windows Service start if configured
        let resolved_svc_name = resolve_windows_service_name(&svc.id, svc.windows_service_name.as_deref());
        if let Some(ref svc_name) = resolved_svc_name {
            let state = get_service_status(svc_name);
            if state == WindowsServiceState::Running {
                self.log(format!(
                    "[SERVICE] Windows Service '{}' is already running.",
                    svc_name
                )).await;
                return Ok(());
            } else if state != WindowsServiceState::NotInstalled {
                self.log(format!(
                    "[SERVICE] Starting Windows Service '{}' via SCM...",
                    svc_name
                )).await;

                let mut started = start_service(svc_name, 5).is_ok();
                if !started {
                    self.log(format!(
                        "[SERVICE] Direct SCM start for '{}' failed (Access Denied). Requesting Administrator elevation (UAC)...",
                        svc_name
                    )).await;
                    let _ = start_service_elevated(svc_name);

                    // Poll for up to 6 seconds to confirm status
                    for _ in 0..12 {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        if get_service_status(svc_name) == WindowsServiceState::Running {
                            started = true;
                            break;
                        }
                    }
                }

                if started {
                    self.log(format!(
                        "[SERVICE] SUCCESS: Windows Service '{}' is now running.",
                        svc_name
                    )).await;
                    return Ok(());
                } else {
                    self.log(format!(
                        "[SERVICE] WARNING: Windows Service '{}' could not be started. Attempting standalone binary fallback...",
                        svc_name
                    )).await;
                }
            }
        }

        // 3. Resolve executable binary
        let exe_path = match self.resolve_binary_path(svc) {
            Some(p) => {
                self.log(format!(
                    "[SERVICE] Located executable for '{}': {}",
                    svc.name, p.display()
                )).await;
                p
            }
            None => {
                let err = format!(
                    "Executable binary '{:?}' not found in tools directory, D:\\Apps, C:\\Apps, Program Files, or system PATH",
                    svc.binary_name
                );
                self.log(format!("[SERVICE] ERROR: {}", err)).await;
                return Err(anyhow!(err));
            }
        };

        let work_dir = exe_path.parent().map(|p| p.to_path_buf());
        match self.supervisor.start_service(svc, exe_path.clone(), work_dir).await {
            Ok(pid) => {
                let tracker = self.tracker.lock().await;
                let _ = tracker.record_process(&svc.id, pid, &exe_path);
                self.log(format!(
                    "[SERVICE] SUCCESS: Started '{}' process (PID: {}, Port: {})",
                    svc.name, pid, svc.port
                )).await;
                Ok(())
            }
            Err(e) => {
                self.log(format!(
                    "[SERVICE] ERROR: Failed to start process '{}': {}",
                    svc.name, e
                )).await;
                Err(anyhow!(e))
            }
        }
    }

    /// Stop a service by ID (supports Windows Services and process termination).
    pub async fn stop_service(&self, id: &str) -> Result<()> {
        let svc = self
            .services
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(id))
            .ok_or_else(|| anyhow!("Unknown service '{id}'"))?;

        self.log(format!("[SERVICE] Stopping '{}'...", svc.name)).await;

        // 1. Try Windows Service stop if applicable
        let resolved_svc_name = resolve_windows_service_name(&svc.id, svc.windows_service_name.as_deref());
        if let Some(ref svc_name) = resolved_svc_name {
            let state = get_service_status(svc_name);
            if state == WindowsServiceState::Running {
                self.log(format!("[SERVICE] Stopping Windows Service '{}' via SCM...", svc_name)).await;
                let scm_res = stop_service(svc_name, 3);
                if let Err(e) = scm_res {
                    self.log(format!(
                        "[SERVICE] SCM stop for '{}' returned: {}. Requesting Administrator elevation (UAC)...",
                        svc_name, e
                    )).await;
                    let _ = stop_service_elevated(svc_name);

                    // Await up to 6 seconds for elevated stop to take effect
                    for _ in 0..12 {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        if get_service_status(svc_name) != WindowsServiceState::Running {
                            break;
                        }
                    }
                }
            }
        }

        // 2. Stop managed supervisor process
        let _ = self.supervisor.stop_service(&svc.id).await;

        // 3. Stop port owner if still listening
        if svc.port > 0 {
            if let Ok(Some(pid)) = get_owner_pid_for_port(svc.port) {
                if is_pid_alive(pid) {
                    self.log(format!(
                        "[SERVICE] Terminating process holding port {} (PID: {})...",
                        svc.port, pid
                    )).await;
                    if let Err(e) = terminate_pid(pid) {
                        self.log(format!(
                            "[SERVICE] Direct process termination failed: {}. Requesting elevated taskkill...",
                            e
                        )).await;
                        let _ = terminate_pid_elevated(pid);
                        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    }
                }
            }
        }

        // 4. Strict real-status verification
        let port_still_open = if svc.port > 0 {
            get_owner_pid_for_port(svc.port).ok().flatten().is_some()
        } else {
            false
        };

        let scm_still_running = if let Some(ref svc_name) = resolved_svc_name {
            get_service_status(svc_name) == WindowsServiceState::Running
        } else {
            false
        };

        if port_still_open || scm_still_running {
            let pid_info = if svc.port > 0 {
                get_owner_pid_for_port(svc.port)
                    .ok()
                    .flatten()
                    .map(|p| format!("PID {}", p))
                    .unwrap_or_else(|| "Unknown PID".into())
            } else {
                "Service process".into()
            };
            let err = format!(
                "Failed to stop '{}': process is still active ({}, SCM: {:?}). Administrator privilege may be required.",
                svc.name, pid_info, resolved_svc_name.as_deref().map(get_service_status)
            );
            self.log(format!("[SERVICE] ERROR: {}", err)).await;
            return Err(anyhow!(err));
        }

        let tracker = self.tracker.lock().await;
        let _ = tracker.remove_process(&svc.id);

        self.log(format!("[SERVICE] SUCCESS: '{}' stopped cleanly.", svc.name)).await;
        Ok(())
    }

    /// Restart a service by ID.
    pub async fn restart_service(&self, id: &str) -> Result<()> {
        self.log(format!("[SERVICE] Restarting service '{}'...", id)).await;
        let _ = self.stop_service(id).await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        self.start_service(id).await
    }

    /// Start all services associated with the currently active profile.
    pub async fn start_all(&self) -> Result<()> {
        let profile = {
            let p_store = self.profile_store.lock().await;
            p_store.get_active_profile()?
        };

        let ordered = ServiceOrchestrator::order_for_startup(&self.services, &profile);
        self.log(format!(
            "[BATCH] Starting {} services for active profile '{}'...",
            ordered.len(),
            profile.name
        )).await;

        let mut success_count = 0;
        let mut fail_count = 0;

        for svc in ordered {
            match self.start_service(&svc.id).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    fail_count += 1;
                    warn!("Could not start service '{}': {e}", svc.id);
                }
            }
        }

        self.log(format!(
            "[BATCH] Batch startup summary: {} succeeded, {} failed.",
            success_count, fail_count
        )).await;

        Ok(())
    }

    /// Stop all services in reverse dependency order.
    pub async fn stop_all(&self) -> Result<()> {
        let profile = {
            let p_store = self.profile_store.lock().await;
            p_store.get_active_profile()?
        };

        let ordered = ServiceOrchestrator::order_for_shutdown(&self.services, &profile);
        self.log(format!(
            "[BATCH] Stopping all services (total: {})...",
            ordered.len()
        )).await;

        for svc in ordered {
            if let Err(e) = self.stop_service(&svc.id).await {
                warn!("Error stopping service '{}': {e}", svc.id);
            }
        }

        self.log("[BATCH] All services have been stopped.").await;
        Ok(())
    }
}

fn resolve_windows_service_name(id: &str, configured: Option<&str>) -> Option<String> {
    let candidates: &[&str] = match id {
        "mysql" => &["MySQL84", "MySQL80", "MySQL", "mysql"],
        "postgres" => &[
            "postgresql-x64-17",
            "postgresql-x64-16",
            "postgresql-x64-15",
            "postgresql-x64-14",
            "PostgreSQL",
            "postgres",
        ],
        "redis" => &["Memurai", "Redis", "redis"],
        _ => &[],
    };

    if let Some(cfg) = configured {
        let status = get_service_status(cfg);
        if status != WindowsServiceState::NotInstalled {
            return Some(cfg.to_string());
        }
    }

    resolve_installed_service_name(candidates)
}

fn chrono_lite_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}
