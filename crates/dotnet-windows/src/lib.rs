//! Windows native API bindings and system integration for Dotnet

pub mod autostart;
pub mod error;
pub mod ip_helper;
pub mod native_window;
pub mod process;
pub mod registry;
pub mod service_ctrl;
pub mod shortcut;
pub mod single_instance;
pub mod task_scheduler;
pub mod tray;

pub use autostart::AutostartManager;
pub use error::{Result, WindowsError};
pub use ip_helper::{get_active_tcp_listeners, get_owner_pid_for_port, TcpListenerEntry};
pub use native_window::{NativeMainWindow, NativeWindowEvent};
pub use process::{elevate_self, is_elevated, is_pid_alive, terminate_pid};
pub use registry::{
    broadcast_environment_change, delete_run_key, get_run_key, get_system_path, get_user_path,
    set_run_key, set_user_path,
};
pub use service_ctrl::{
    get_service_status, resolve_installed_service_name, start_service, start_service_elevated,
    stop_service, stop_service_elevated, terminate_pid_elevated, WindowsServiceState,
};
pub use shortcut::ShortcutManager;
pub use single_instance::SingleInstanceGuard;
pub use task_scheduler::TaskScheduler;
pub use tray::{SystemTray, TrayEvent};
