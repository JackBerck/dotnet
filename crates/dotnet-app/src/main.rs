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
use dotnet_windows::{NativeMainWindow, NativeWindowEvent};
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

async fn render_dashboard_content(
    tab: usize,
    sm: &AppServiceManager,
    tool_store: &Arc<Mutex<InstalledToolStore>>,
    profile_store: &Arc<Mutex<ProfileStore>>,
    logs: &Arc<Mutex<Vec<String>>>,
) -> String {
    let mut out = String::with_capacity(4096);

    let active_profile = {
        let p = profile_store.lock().await;
        p.get_active_profile()
            .map(|p| p.name)
            .unwrap_or_else(|_| "Standalone (Portable Native)".to_string())
    };

    out.push_str("========================================================================================================\r\n");
    out.push_str(&format!(
        " DOTNET - NATIVE WINDOWS DEV RUNTIME MANAGER (RUST v{}) | PROFIL: {}\r\n",
        env!("CARGO_PKG_VERSION"),
        active_profile.to_uppercase()
    ));
    out.push_str("========================================================================================================\r\n\r\n");

    match tab {
        0 => {
            out.push_str("[ TAB: STATUS LAYANAN RUNTIME (SERVICES) ]\r\n");
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str(&format!(
                "{:<14} {:<24} {:<8} {:<10} {:<18} {}\r\n",
                "STATUS", "LAYANAN", "PORT", "PID", "TIPE", "ID PROSES"
            ));
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");

            let statuses = sm.get_all_statuses().await;
            for s in &statuses {
                let status_str = if s.is_running { "[RUNNING]" } else { "[STOPPED]" };
                let pid_str = s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
                let port_str = if s.port > 0 { s.port.to_string() } else { "-".to_string() };
                out.push_str(&format!(
                    "{:<14} {:<24} {:<8} {:<10} {:<18} {}\r\n",
                    status_str,
                    s.name,
                    port_str,
                    pid_str,
                    s.r#type,
                    s.id
                ));
            }
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str("Aksi Cepat Toolbar Atas:\r\n");
            out.push_str(" [▶ Start All]     : Jalankan semua service runtime pengembang.\r\n");
            out.push_str(" [⏹ Stop All]      : Hentikan seluruh background service.\r\n");
            out.push_str(" [🔄 Reload Nginx] : Muat ulang konfigurasi virtual host Nginx & sertifikat SSL.\r\n");
            out.push_str(" [🔍 Scan & Adopt] : Pindai ulang tools terpasang di host PC.\r\n");
            out.push_str(" [⚡ Switch Profile]: Ganti profil antara Standalone (Native) dan Docker.\r\n");
            out.push_str(" [⟳ Refresh]       : Periksa kembali status port dan proses saat ini.\r\n");
        }
        1 => {
            out.push_str("[ TAB: TOOLS PENGEMBANG TERPASANG (DIADOPSI DARI SISTEM WINDOWS) ]\r\n");
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str(&format!(
                "{:<12} {:<26} {:<12} {:<10} {}\r\n",
                "TOOL ID", "NAMA TOOL", "VERSI", "STATUS", "DIREKTORI INSTALASI"
            ));
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");

            let store = tool_store.lock().await;
            let tools = store.load_all().unwrap_or_default();
            if tools.is_empty() {
                out.push_str("Belum ada tool terdaftar. Klik [🔍 Scan & Adopt] untuk memindai environment Anda.\r\n");
            } else {
                for t in &tools {
                    let st = if t.is_active { "[AKTIF]" } else { "[NONAKTIF]" };
                    out.push_str(&format!(
                        "{:<12} {:<26} {:<12} {:<10} {}\r\n",
                        t.tool_id,
                        t.display_name,
                        t.version,
                        st,
                        t.install_path
                    ));
                }
            }
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str(&format!("Total Tools Terdaftar: {} tools.\r\n", tools.len()));
            out.push_str("Semua tool di atas otomatis disinkronkan ke PATH dan lingkungan runtime aplikasi lokal Anda.\r\n");
        }
        2 => {
            out.push_str("[ TAB: PORT TCP AKTIF & DETEKSI KONFLIK (WIN32 IP HELPER API) ]\r\n");
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str(&format!(
                "{:<10} {:<12} {:<32} {}\r\n",
                "PORT", "PID", "ALAMAT LOKAL", "PROSES PEMILIK"
            ));
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");

            match dotnet_windows::get_active_tcp_listeners() {
                Ok(listeners) => {
                    let mut relevant = listeners
                        .into_iter()
                        .filter(|l| {
                            l.port == 80
                                || l.port == 443
                                || l.port == 3306
                                || l.port == 5432
                                || l.port == 6379
                                || l.port == 8025
                                || l.port == 9000
                                || l.port == 4040
                                || (3000..=9999).contains(&l.port)
                        })
                        .collect::<Vec<_>>();
                    relevant.sort_by_key(|l| l.port);

                    for l in &relevant {
                        out.push_str(&format!(
                            "{:<10} {:<12} {:<32} PID {}\r\n",
                            l.port, l.owning_pid, l.local_addr, l.owning_pid
                        ));
                    }
                }
                Err(e) => {
                    out.push_str(&format!("Gagal membaca tabel TCP: {}\r\n", e));
                }
            }
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            out.push_str("Data port diperiksa langsung via Windows GetExtendedTcpTable (IPv4 & IPv6).\r\n");
        }
        _ => {
            out.push_str("[ TAB: LOG PERISTIWA RUNTIME ENGINE ]\r\n");
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
            let log_guard = logs.lock().await;
            for line in log_guard.iter().rev().take(30).rev() {
                out.push_str(line);
                out.push_str("\r\n");
            }
            out.push_str("--------------------------------------------------------------------------------------------------------\r\n");
        }
    }

    out
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
            info!("Another Dotnet instance is already running. Exiting cleanly.");
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

    // Auto-scan and adopt pre-installed host developer tools on startup
    {
        let store = tool_store.lock().await;
        match AdoptExistingScanner::adopt_all_discovered(&store) {
            Ok(count) => {
                info!("Auto-adopted {count} installed local developer tools.");
            }
            Err(e) => {
                warn!("Auto-adoption scanner warning: {e}");
            }
        }
    }

    let service_manager = Arc::new(AppServiceManager::new(
        process_tracker.clone(),
        profile_store.clone(),
    ));

    let logs = Arc::new(Mutex::new(vec![
        format!("[{}] Dotnet Rust Native Desktop Engine started.", chrono_lite()),
        format!("[{}] Headless API port: {}", chrono_lite(), args.port),
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

    // Spawn background HTTP API server
    let server = DashboardServer::new(dashboard_state.clone());
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            error!("Dashboard server error: {e}");
        }
    });

    if args.headless {
        info!("Running in headless mode. Port: {}", args.port);
        println!("Dotnet engine running headless on port {}", args.port);
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

    // Initialize Native Windows Desktop GUI Window
    let (native_window, window_rx) =
        NativeMainWindow::create("Dotnet - Native Dev Environment Manager", 980, 680)
            .context("Failed to create native desktop window")?;
    let native_window = Arc::new(native_window);

    // Initial render into native window
    let initial_content = render_dashboard_content(0, &service_manager, &tool_store, &profile_store, &logs).await;
    native_window.update_content(&initial_content);

    if !args.minimized {
        native_window.show();
    } else {
        native_window.hide();
    }

    let _ = tray.show_balloon(
        "Dotnet Dev Engine",
        "Aplikasi Desktop Native Dotnet siap digunakan.",
        false,
    );

    // Spawn native window event handler thread
    let nw_win = native_window.clone();
    let sm_win = service_manager.clone();
    let ts_win = tool_store.clone();
    let ps_win = profile_store.clone();
    let lg_win = logs.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build window tokio runtime");

        while let Ok(event) = window_rx.recv() {
            match event {
                NativeWindowEvent::StartAll => {
                    let sm = sm_win.clone();
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        let _ = sm.start_all().await;
                        lg.lock()
                            .await
                            .push(format!("[{}] Memulai semua service runtime...", chrono_lite()));
                    });
                }
                NativeWindowEvent::StopAll => {
                    let sm = sm_win.clone();
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        let _ = sm.stop_all().await;
                        lg.lock()
                            .await
                            .push(format!("[{}] Menghentikan semua service runtime...", chrono_lite()));
                    });
                }
                NativeWindowEvent::ReloadNginx => {
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        let nginx_dir = AppPaths::tools_dir().join("nginx").join("current");
                        let res = NginxManager::reload_nginx(&nginx_dir).await;
                        let msg = match res {
                            Ok((success, output)) => {
                                if success {
                                    format!("[{}] Nginx configuration reloaded successfully.", chrono_lite())
                                } else {
                                    format!("[{}] Nginx reload output: {}", chrono_lite(), output)
                                }
                            }
                            Err(e) => format!("[{}] Nginx reload failed: {}", chrono_lite(), e),
                        };
                        lg.lock().await.push(msg);
                    });
                }
                NativeWindowEvent::ScanAndAdoptTools => {
                    let ts = ts_win.clone();
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        let store = ts.lock().await;
                        match AdoptExistingScanner::adopt_all_discovered(&store) {
                            Ok(c) => {
                                lg.lock().await.push(format!(
                                    "[{}] Pemindaian selesai: {} tool baru diadopsi.",
                                    chrono_lite(),
                                    c
                                ));
                            }
                            Err(e) => {
                                lg.lock().await.push(format!(
                                    "[{}] Gagal memindai tools: {}",
                                    chrono_lite(),
                                    e
                                ));
                            }
                        }
                    });
                }
                NativeWindowEvent::SwitchProfile => {
                    let ps = ps_win.clone();
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        let p = ps.lock().await;
                        if let Ok(current) = p.get_active_profile() {
                            let next = if current.id.eq_ignore_ascii_case("standalone") {
                                "docker"
                            } else {
                                "standalone"
                            };
                            let _ = p.set_active_profile(next);
                            lg.lock().await.push(format!(
                                "[{}] Profil dialihkan ke: {}",
                                chrono_lite(),
                                next
                            ));
                        }
                    });
                }
                NativeWindowEvent::RefreshStatus => {
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        lg.lock()
                            .await
                            .push(format!("[{}] Status runtime diperbarui.", chrono_lite()));
                    });
                }
                NativeWindowEvent::SwitchTab(_) => {}
                NativeWindowEvent::WindowHiddenToTray => {
                    let lg = lg_win.clone();
                    rt.block_on(async {
                        lg.lock()
                            .await
                            .push(format!("[{}] Window diminimalkan ke System Tray.", chrono_lite()));
                    });
                }
                NativeWindowEvent::WindowClosed => {
                    break;
                }
                NativeWindowEvent::ToggleService(_) => {}
            }

            let tab = NativeMainWindow::get_active_tab();
            let sm = sm_win.clone();
            let ts = ts_win.clone();
            let ps = ps_win.clone();
            let lg = lg_win.clone();
            let nw = nw_win.clone();

            rt.block_on(async move {
                let content = render_dashboard_content(tab, &sm, &ts, &ps, &lg).await;
                nw.update_content(&content);
            });
        }
    });

    // Spawn system tray event handler thread
    let nw_tray = native_window.clone();
    let sm_tray = service_manager.clone();
    let ts_tray = tool_store.clone();
    let ps_tray = profile_store.clone();
    let lg_tray = logs.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tray tokio runtime");

        while let Ok(event) = tray_rx.recv() {
            match event {
                TrayEvent::OpenDashboard => {
                    nw_tray.show();
                }
                TrayEvent::StartAll => {
                    let sm = sm_tray.clone();
                    let lg = lg_tray.clone();
                    rt.block_on(async {
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
                    rt.block_on(async {
                        let _ = sm.stop_all().await;
                        lg.lock().await.push(format!(
                            "[{}] Menghentikan semua service dari System Tray...",
                            chrono_lite()
                        ));
                    });
                }
                TrayEvent::ReloadNginx => {
                    let lg = lg_tray.clone();
                    rt.block_on(async {
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
                    rt.block_on(async {
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
                TrayEvent::CheckPorts => {
                    nw_tray.show();
                }
                TrayEvent::Exit => {
                    info!("Exit triggered from system tray. Shutting down.");
                    let sm = sm_tray.clone();
                    rt.block_on(async {
                        let _ = sm.stop_all().await;
                    });
                    unsafe {
                        PostQuitMessage(0);
                    }
                    break;
                }
            }

            let tab = NativeMainWindow::get_active_tab();
            let sm = sm_tray.clone();
            let ts = ts_tray.clone();
            let ps = ps_tray.clone();
            let lg = lg_tray.clone();
            let nw = nw_tray.clone();

            rt.block_on(async move {
                let content = render_dashboard_content(tab, &sm, &ts, &ps, &lg).await;
                nw.update_content(&content);
            });
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
