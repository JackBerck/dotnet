//! Application Service Manager orchestrating all developer runtime services.

use anyhow::{anyhow, Result};
use dotnet_core::models::{DevServiceDescriptor, ServiceStatus, ServiceType};
use dotnet_core::persistence::AppPaths;
use dotnet_core::services::process_service::ProcessSupervisor;
use dotnet_core::services::process_tracker::ProcessTracker;
use dotnet_core::services::profile_store::ProfileStore;
use dotnet_core::services::service_orchestrator::ServiceOrchestrator;
use dotnet_windows::is_pid_alive;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
}

impl AppServiceManager {
    pub fn new(
        tracker: Arc<Mutex<ProcessTracker>>,
        profile_store: Arc<Mutex<ProfileStore>>,
    ) -> Self {
        let supervisor = ProcessSupervisor::new();
        let services = Self::default_descriptors();
        Self {
            supervisor,
            tracker,
            profile_store,
            services,
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
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("mysqld.exe".into()),
                default_args: vec!["--console".into()],
                auto_start: true,
            },
            DevServiceDescriptor {
                id: "postgres".into(),
                name: "PostgreSQL Database".into(),
                description: "Object-relational database system".into(),
                port: 5432,
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("postgres.exe".into()),
                default_args: vec![],
                auto_start: false,
            },
            DevServiceDescriptor {
                id: "redis".into(),
                name: "Redis In-Memory Store".into(),
                description: "High-performance cache and message broker".into(),
                port: 6379,
                service_type: ServiceType::ManagedProcess,
                windows_service_name: None,
                binary_name: Some("redis-server.exe".into()),
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

    /// Read all service statuses.
    pub async fn get_all_statuses(&self) -> Vec<ServiceItem> {
        let tracker = self.tracker.lock().await;
        let mut items = Vec::new();

        for s in &self.services {
            let status = self.supervisor.get_service_status(s);
            let mut is_running = status == ServiceStatus::Running;
            let mut pid = None;

            // Cross-check with persistent tracker
            if let Ok(Some(record)) = tracker.get_process(&s.id) {
                if is_pid_alive(record.pid) {
                    is_running = true;
                    pid = Some(record.pid);
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

    fn resolve_binary_path(&self, service: &DevServiceDescriptor) -> Option<PathBuf> {
        let bin_name = service.binary_name.as_deref()?;
        let candidate = AppPaths::tools_dir()
            .join(&service.id)
            .join("current")
            .join(bin_name);

        if candidate.is_file() {
            Some(candidate)
        } else {
            which::which(bin_name).ok()
        }
    }

    /// Start a service by ID.
    pub async fn start_service(&self, id: &str) -> Result<()> {
        let svc = self
            .services
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(id))
            .ok_or_else(|| anyhow!("Unknown service '{id}'"))?;

        let exe_path = self.resolve_binary_path(svc).ok_or_else(|| {
            anyhow!(
                "Service binary '{:?}' not found in tools directory or PATH",
                svc.binary_name
            )
        })?;

        let work_dir = exe_path.parent().map(|p| p.to_path_buf());
        let pid = self
            .supervisor
            .start_service(svc, exe_path.clone(), work_dir)
            .await?;

        let tracker = self.tracker.lock().await;
        let _ = tracker.record_process(&svc.id, pid, &exe_path);

        info!("Service '{}' successfully started (PID: {pid})", svc.name);
        Ok(())
    }

    /// Stop a service by ID.
    pub async fn stop_service(&self, id: &str) -> Result<()> {
        let svc = self
            .services
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(id))
            .ok_or_else(|| anyhow!("Unknown service '{id}'"))?;

        let _ = self.supervisor.stop_service(&svc.id).await;

        let tracker = self.tracker.lock().await;
        let _ = tracker.remove_process(&svc.id);

        info!("Service '{}' stopped", svc.name);
        Ok(())
    }

    /// Restart a service by ID.
    pub async fn restart_service(&self, id: &str) -> Result<()> {
        let _ = self.stop_service(id).await;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        self.start_service(id).await
    }

    /// Start all services associated with the currently active profile.
    pub async fn start_all(&self) -> Result<()> {
        let profile = {
            let p_store = self.profile_store.lock().await;
            p_store.get_active_profile()?
        };

        let ordered = ServiceOrchestrator::order_for_startup(&self.services, &profile);
        info!(
            "Starting all {} services for active profile '{}'",
            ordered.len(),
            profile.name
        );

        for svc in ordered {
            if let Err(e) = self.start_service(&svc.id).await {
                warn!("Could not start service '{}': {e}", svc.id);
            }
        }

        Ok(())
    }

    /// Stop all services in reverse dependency order.
    pub async fn stop_all(&self) -> Result<()> {
        let profile = {
            let p_store = self.profile_store.lock().await;
            p_store.get_active_profile()?
        };

        let ordered = ServiceOrchestrator::order_for_shutdown(&self.services, &profile);
        info!("Stopping all services in reverse dependency order...");

        for svc in ordered {
            if let Err(e) = self.stop_service(&svc.id).await {
                warn!("Error stopping service '{}': {e}", svc.id);
            }
        }

        Ok(())
    }
}
