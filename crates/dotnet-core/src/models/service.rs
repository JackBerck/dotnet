use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    NotInstalled,
    PortConflict,
    Error,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    ManagedProcess,
    WindowsService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServiceDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub port: u16,
    pub service_type: ServiceType,
    pub windows_service_name: Option<String>,
    pub binary_name: Option<String>,
    pub default_args: Vec<String>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningProcessInfo {
    pub service_id: String,
    pub pid: u32,
    pub start_time_secs: u64,
    pub binary_path: String,
    pub port: u16,
}
