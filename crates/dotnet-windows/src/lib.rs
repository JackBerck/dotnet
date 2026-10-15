//! Windows native API bindings and system integration for Dotnet

pub mod error;
pub mod ip_helper;
pub mod process;
pub mod registry;
pub mod service_ctrl;
pub mod task_scheduler;

pub use error::{Result, WindowsError};
pub use ip_helper::{get_active_tcp_listeners, get_owner_pid_for_port, TcpListenerEntry};
pub use process::{is_pid_alive, terminate_pid};
pub use registry::{broadcast_environment_change, get_system_path, get_user_path, set_user_path};
pub use service_ctrl::{get_service_status, start_service, stop_service, WindowsServiceState};
pub use task_scheduler::TaskScheduler;
