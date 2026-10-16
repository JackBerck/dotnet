#![windows_subsystem = "windows"]
//! Dotnet Desktop GUI, System Tray & API Engine runner.

pub mod dashboard;
pub mod service_manager;
pub mod ui;

use anyhow::{Context, Result};
use dashboard::{DashboardServer, DashboardState};
use dotnet_core::installation::adopt_scanner::AdoptExistingScanner;
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
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use wry::WebViewBuilder;

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

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = parse_args();
    info!("Parsed CLI arguments: {:?}", args);

    // Single-instance check
    let _single_instance = match SingleInstanceGuard::acquire(SINGLE_INSTANCE_MUTEX) {
        Ok(Some(guard)) => guard,
        Ok(None) => {
            info!("Another Dotnet instance is already running. Restoring window and exiting cleanly.");
            restore_existing_instance(args.port);
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

    // Initialize tokio runtime for background tasks and dashboard API server
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to build tokio runtime")?;

    // Initialize core stores & components
    let tool_store = Arc::new(Mutex::new(InstalledToolStore::new()));
    let profile_store = Arc::new(Mutex::new(ProfileStore::new()));
    let project_manager = Arc::new(Mutex::new(ProjectManager::new()));
    let php_config_manager = Arc::new(Mutex::new(PhpConfigManager::new()));
    let process_tracker = Arc::new(Mutex::new(ProcessTracker::new()));

    // Auto-scan and adopt pre-installed host developer tools on startup
    {
        let store = rt.block_on(tool_store.lock());
        match AdoptExistingScanner::adopt_all_discovered(&store) {
            Ok(count) => {
                info!("Auto-adopted {count} installed local developer tools.");
            }
            Err(e) => {
                warn!("Auto-adoption scanner warning: {e}");
            }
        }
    }

    let logs = Arc::new(Mutex::new(vec![
        format!("[{}] Dotnet Rust Native Desktop Engine started.", chrono_lite()),
        format!("[{}] Local Engine API Port: {}", chrono_lite(), args.port),
    ]));

    let service_manager = Arc::new(AppServiceManager::new(
        process_tracker.clone(),
        profile_store.clone(),
        Some(logs.clone()),
    ));

    // Build Tao Event Loop if not running in headless mode
    let (event_loop, proxy_opt) = if !args.headless {
        let el = EventLoopBuilder::<TrayEvent>::with_user_event().build();
        let pr = el.create_proxy();
        (Some(el), Some(pr))
    } else {
        (None, None)
    };

    let dashboard_state = DashboardState {
        service_manager: service_manager.clone(),
        project_manager: project_manager.clone(),
        profile_store: profile_store.clone(),
        tool_store: tool_store.clone(),
        php_config_manager: php_config_manager.clone(),
        logs: logs.clone(),
        port: args.port,
        event_loop_proxy: proxy_opt.clone(),
    };

    // Spawn background HTTP API server
    let server = DashboardServer::new(dashboard_state.clone());
    rt.spawn(async move {
        if let Err(e) = server.run().await {
            error!("Dashboard server error: {e}");
        }
    });

    if args.headless {
        info!("Running in headless mode. Port: {}", args.port);
        println!("Dotnet engine running headless on port {}", args.port);
        rt.block_on(async {
            let _ = tokio::signal::ctrl_c().await;
        });
        info!("Exiting Dotnet engine...");
        return Ok(());
    }

    let event_loop = event_loop.expect("EventLoop must be initialized in non-headless mode");
    let proxy = proxy_opt.expect("Proxy must be initialized in non-headless mode");

    // Initialize Win32 System Tray
    let (mut tray, tray_rx) = SystemTray::create("Dotnet - Local Dev Engine")
        .context("Failed to initialize Windows System Tray")?;

    info!("System Tray initialized successfully.");

    // Spawn tray event bridge thread
    let sm_tray = service_manager.clone();
    let ps_tray = profile_store.clone();
    let lg_tray = logs.clone();
    let rt_handle = rt.handle().clone();

    std::thread::spawn(move || {
        while let Ok(event) = tray_rx.recv() {
            match event {
                TrayEvent::StartAll => {
                    let sm = sm_tray.clone();
                    let lg = lg_tray.clone();
                    rt_handle.block_on(async {
                        let _ = sm.start_all().await;
                        lg.lock().await.push(format!(
                            "[{}] Memulai semua service dari System Tray...",
                            chrono_lite()
                        ));
                    });
                }
                TrayEvent::StopAll => {
                    let sm = sm_tray.clone();
                    let lg = lg_tray.clone();
                    rt_handle.block_on(async {
                        let _ = sm.stop_all().await;
                        lg.lock().await.push(format!(
                            "[{}] Menghentikan semua service dari System Tray...",
                            chrono_lite()
                        ));
                    });
                }
                TrayEvent::ReloadNginx => {
                    let lg = lg_tray.clone();
                    rt_handle.block_on(async {
                        let nginx_dir = AppPaths::tools_dir().join("nginx").join("current");
                        let _ = NginxManager::reload_nginx(&nginx_dir).await;
                        lg.lock().await.push(format!(
                            "[{}] Nginx reloaded dari System Tray.",
                            chrono_lite()
                        ));
                    });
                }
                TrayEvent::SwitchProfile => {
                    let ps = ps_tray.clone();
                    let lg = lg_tray.clone();
                    rt_handle.block_on(async {
                        let p = ps.lock().await;
                        if let Ok(current) = p.get_active_profile() {
                            let next = if current.id.eq_ignore_ascii_case("standalone") {
                                "docker"
                            } else {
                                "standalone"
                            };
                            let _ = p.set_active_profile(next);
                            lg.lock().await.push(format!(
                                "[{}] Profil diubah dari System Tray: {}",
                                chrono_lite(),
                                next
                            ));
                        }
                    });
                }
                TrayEvent::OpenDashboard | TrayEvent::CheckPorts | TrayEvent::Exit => {
                    let _ = proxy.send_event(event);
                }
            }
        }
    });

    // Create authentic Windows desktop window
    let window = WindowBuilder::new()
        .with_title("Dotnet - Development Environment Engine")
        .with_inner_size(tao::dpi::LogicalSize::new(1180.0, 780.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(850.0, 600.0))
        .with_visible(!args.minimized)
        .build(&event_loop)
        .context("Failed to create native desktop window")?;

    // Embed WebView2 directly inside the desktop window
    let dashboard_url = format!("http://127.0.0.1:{}", args.port);
    let _webview = WebViewBuilder::new()
        .with_url(&dashboard_url)
        .build(&window)
        .context("Failed to initialize embedded desktop webview")?;

    let _ = tray.show_balloon(
        "Dotnet Dev Engine",
        "Aplikasi Desktop Native Dotnet aktif.",
        false,
    );

    // Run Tao Event Loop on main thread
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(tray_ev) => match tray_ev {
                TrayEvent::OpenDashboard | TrayEvent::CheckPorts => {
                    window.set_visible(true);
                    window.set_focus();
                }
                TrayEvent::Exit => {
                    info!("Exit triggered from system tray. Shutting down.");
                    let sm = service_manager.clone();
                    rt.block_on(async {
                        let _ = sm.stop_all().await;
                    });
                    tray.remove();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // Minimize to system tray instead of exiting immediately
                window.set_visible(false);
                let _ = tray.show_balloon(
                    "Dotnet Dev Engine",
                    "Aplikasi berjalan di background (System Tray). Klik icon tray atau jalankan dotnet-app untuk memulihkan.",
                    false,
                );
            }
            _ => {}
        }
    });
}

fn restore_existing_instance(port: u16) {
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    // 1. Send HTTP restore signal to existing instance background API
    let addr_str = format!("127.0.0.1:{}", port);
    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(800)) {
            let req = format!(
                "POST /api/app/restore HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                port
            );
            let _ = stream.write_all(req.as_bytes());
            let _ = stream.flush();
        }
    }

    // 2. Win32 FindWindow fallback to restore and focus
    bring_window_to_front();
}

fn bring_window_to_front() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
    };

    let title: Vec<u16> = OsStr::new("Dotnet - Development Environment Engine\0")
        .encode_wide()
        .collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }
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
