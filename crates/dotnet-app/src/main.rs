//! Dotnet Desktop GUI, System Tray & API Engine runner.

pub mod dashboard;
pub mod service_manager;
pub mod ui;

use anyhow::{Context, Result};
use dashboard::{DashboardServer, DashboardState};
use dotnet_core::installation::store::InstalledToolStore;
use dotnet_core::persistence::AppPaths;
use dotnet_core::services::nginx_manager::NginxManager;
use dotnet_core::services::php_config_manager::PhpConfigManager;
use dotnet_core::services::process_tracker::ProcessTracker;
use dotnet_core::services::profile_store::ProfileStore;
use dotnet_core::services::project_manager::ProjectManager;
use dotnet_windows::single_instance::SingleInstanceGuard;
use dotnet_windows::tray::{SystemTray, TrayEvent};
use service_manager::AppServiceManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage;

const SINGLE_INSTANCE_MUTEX: &str = "Global\\Dotnet_SingleInstance_Mutex";
const DEFAULT_PORT: u16 = 4040;

#[derive(Debug)]
struct CliArgs {
    minimized: bool,
    headless: bool,
    port: u16,
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let minimized = args
        .iter()
        .any(|a| a == "--minimized" || a == "--autostart");
    let headless = args.iter().any(|a| a == "--headless");

    let mut port = DEFAULT_PORT;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                port = p;
            }
        }
        i += 1;
    }

    CliArgs {
        minimized,
        headless,
        port,
    }
}

fn open_in_browser(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .spawn();
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = parse_args();
    info!("Parsed CLI arguments: {:?}", args);

    // Single-instance check
    let _single_instance = match SingleInstanceGuard::acquire(SINGLE_INSTANCE_MUTEX) {
        Ok(Some(guard)) => guard,
        Ok(None) => {
            info!("Another Dotnet instance is already running.");
            if !args.minimized {
                open_in_browser(&format!("http://127.0.0.1:{}", args.port));
            }
            return Ok(());
        }
        Err(e) => {
            warn!("Could not enforce single instance: {e}. Continuing anyway.");
            SingleInstanceGuard::acquire("Local\\Dotnet_Fallback_Mutex")
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    panic!("Fatal: unable to allocate fallback mutex");
                })
        }
    };

    // Initialize core stores & components
    let tool_store = Arc::new(Mutex::new(InstalledToolStore::new()));
    let profile_store = Arc::new(Mutex::new(ProfileStore::new()));
    let project_manager = Arc::new(Mutex::new(ProjectManager::new()));
    let php_config_manager = Arc::new(Mutex::new(PhpConfigManager::new()));
    let process_tracker = Arc::new(Mutex::new(ProcessTracker::new()));

    let service_manager = Arc::new(AppServiceManager::new(
        process_tracker.clone(),
        profile_store.clone(),
    ));

    let logs = Arc::new(Mutex::new(vec![
        format!("[{}] Dotnet Rust Native Engine started.", chrono_lite()),
        format!("[{}] Dashboard HTTP port: {}", chrono_lite(), args.port),
    ]));

    let dashboard_state = DashboardState {
        service_manager: service_manager.clone(),
        project_manager: project_manager.clone(),
        profile_store: profile_store.clone(),
        tool_store: tool_store.clone(),
        php_config_manager: php_config_manager.clone(),
        logs: logs.clone(),
        port: args.port,
    };

    // Spawn HTTP dashboard server
    let server = DashboardServer::new(dashboard_state.clone());
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            error!("Dashboard server error: {e}");
        }
    });

    let dashboard_url = format!("http://127.0.0.1:{}", args.port);

    if args.headless {
        info!(
            "Running in headless mode. Open dashboard at: {}",
            dashboard_url
        );
        println!("Dotnet engine running headless at {}", dashboard_url);
        tokio::signal::ctrl_c()
            .await
            .context("Failed to listen for ctrl-c")?;
        info!("Exiting Dotnet engine...");
        return Ok(());
    }

    // Initialize Win32 System Tray
    let (mut tray, tray_rx) = SystemTray::create("Dotnet - Local Dev Engine")
        .context("Failed to initialize Windows System Tray")?;

    info!("System Tray initialized successfully.");
    let _ = tray.show_balloon(
        "Dotnet Dev Engine",
        &format!("Dashboard siap di {}", dashboard_url),
        false,
    );

    if !args.minimized {
        open_in_browser(&dashboard_url);
    }

    // Spawn tray event handler thread
    let sm_clone = service_manager.clone();
    let p_store_clone = profile_store.clone();
    let url_clone = dashboard_url.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tray tokio runtime");

        while let Ok(event) = tray_rx.recv() {
            match event {
                TrayEvent::OpenDashboard => {
                    open_in_browser(&url_clone);
                }
                TrayEvent::StartAll => {
                    let sm = sm_clone.clone();
                    rt.block_on(async {
                        if let Err(e) = sm.start_all().await {
                            warn!("Tray StartAll error: {e}");
                        }
                    });
                }
                TrayEvent::StopAll => {
                    let sm = sm_clone.clone();
                    rt.block_on(async {
                        if let Err(e) = sm.stop_all().await {
                            warn!("Tray StopAll error: {e}");
                        }
                    });
                }
                TrayEvent::ReloadNginx => {
                    rt.block_on(async {
                        let nginx_dir = AppPaths::tools_dir().join("nginx").join("current");
                        if let Err(e) = NginxManager::reload_nginx(&nginx_dir).await {
                            warn!("Tray ReloadNginx error: {e}");
                        }
                    });
                }
                TrayEvent::SwitchProfile => {
                    let p_store = p_store_clone.clone();
                    rt.block_on(async {
                        let p = p_store.lock().await;
                        if let Ok(current) = p.get_active_profile() {
                            let next = if current.id.eq_ignore_ascii_case("standalone") {
                                "docker"
                            } else {
                                "standalone"
                            };
                            let _ = p.set_active_profile(next);
                        }
                    });
                }
                TrayEvent::CheckPorts => {
                    info!("Tray check ports triggered.");
                }
                TrayEvent::Exit => {
                    info!("Exit triggered from system tray. Shutting down.");
                    let sm = sm_clone.clone();
                    rt.block_on(async {
                        let _ = sm.stop_all().await;
                    });
                    unsafe {
                        PostQuitMessage(0);
                    }
                    break;
                }
            }
        }
    });

    // Run Win32 Message Loop on main thread
    tray.run_message_loop();
    tray.remove();
    info!("Dotnet application cleanly stopped.");

    Ok(())
}

fn chrono_lite() -> String {
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
