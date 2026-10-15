use crate::error::Result;
use dotnet_windows::get_active_tcp_listeners;

#[derive(Debug, Clone)]
pub struct PortStatus {
    pub port: u16,
    pub is_in_use: bool,
    pub owning_pid: Option<u32>,
}

pub struct PortMonitor;

impl PortMonitor {
    /// Inspect status of a specific port.
    pub fn check_port(port: u16) -> Result<PortStatus> {
        let listeners = get_active_tcp_listeners()?;
        if let Some(entry) = listeners.into_iter().find(|e| e.port == port) {
            Ok(PortStatus {
                port,
                is_in_use: true,
                owning_pid: Some(entry.owning_pid),
            })
        } else {
            Ok(PortStatus {
                port,
                is_in_use: false,
                owning_pid: None,
            })
        }
    }
}
