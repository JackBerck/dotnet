use clap::{Parser, Subcommand};
use dotnet_core::catalog::CatalogLoader;
use dotnet_core::installation::{InstalledToolStore, ToolInstaller};
use dotnet_core::models::ProjectInfo;
use dotnet_core::services::{
    FrameworkDetector, HostsFileManager, LocalCertManager, NginxManager, NginxSiteGenerator,
    PathShadowAnalyzer, PhpPoolManager, ProcessTracker, ProfileStore, ProjectManager,
};
use dotnet_windows::{get_active_tcp_listeners, get_service_status, TaskScheduler};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Parser, Debug)]
#[command(
    name = "dotnet",
    version,
    about = "High-performance standalone dev manager in Rust"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect active ports and detect conflicts (IPv4 + IPv6)
    Ports,
    /// Manage Windows services and background processes
    Services {
        #[arg(short, long)]
        name: String,
    },
    /// Manage Windows hosts file and managed block
    Hosts {
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Generate and manage local SSL certificates
    Ssl {
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// Test or reload Nginx configuration
    Nginx {
        #[arg(short, long)]
        action: String,
    },
    /// List, install, or inspect developer tools
    Tools {
        #[arg(short, long)]
        action: Option<String>,
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Manage service orchestration profiles (standalone vs docker)
    Profile {
        #[arg(short, long)]
        switch: Option<String>,
    },
    /// Analyze PATH shadow collisions between System and User PATH
    Shadow,
    /// Display full system and daemon status overview
    Status,
    /// Inspect Task Scheduler autostart status
    Autostart,
    /// Manage local projects, frameworks, dev commands, and virtual hosts
    Projects {
        #[command(subcommand)]
        action: Option<ProjectAction>,
    },
    /// Manage PHP FastCGI multi-worker pool
    PhpPool {
        #[arg(short, long, default_value = "status")]
        action: String,
        #[arg(short, long, default_value_t = 2)]
        size: usize,
        #[arg(short, long, default_value_t = 9000)]
        port: u16,
    },
}

#[derive(Subcommand, Debug)]
enum ProjectAction {
    /// List all registered projects
    List,
    /// Add and configure a new project
    Add {
        /// Project directory path
        path: String,
        /// Project name (defaults to folder name)
        #[arg(short, long)]
        name: Option<String>,
        /// Local test domain (e.g. myapp.test)
        #[arg(short, long)]
        host: Option<String>,
        /// Dev server port (for Node/Vite/Next)
        #[arg(short, long)]
        port: Option<u16>,
        /// Framework type (laravel, php, nextjs, vite, node, static)
        #[arg(short, long)]
        framework: Option<String>,
        /// Enable HTTPS with local SAN SSL certificate
        #[arg(long)]
        ssl: bool,
    },
    /// Remove a registered project
    Remove {
        /// Project ID, name, or domain
        target: String,
        /// Also delete Nginx virtual host and hosts entry
        #[arg(long)]
        delete_site: bool,
    },
    /// Launch project dev server command
    Run {
        /// Project ID, name, or domain
        target: String,
    },
    /// Open project directory in Windows Explorer
    Explore {
        /// Project ID, name, or domain
        target: String,
    },
    /// Open project in Windows Terminal / PowerShell
    Terminal {
        /// Project ID, name, or domain
        target: String,
    },
    /// Open project in Visual Studio Code
    Code {
        /// Project ID, name, or domain
        target: String,
    },
    /// Update project domain and regenerate Nginx vhost
    Domain {
        /// Project ID, name, or domain
        target: String,
        /// New domain (e.g. newdomain.test)
        domain: String,
        /// Enable SSL for new domain
        #[arg(long)]
        ssl: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ports) => {
            info!("Inspecting active TCP ports via native Win32 IP helper...");
            let listeners = get_active_tcp_listeners()?;
            println!("{:<10} {:<10} {:<30}", "PORT", "PID", "LOCAL ADDRESS");
            println!("{}", "-".repeat(52));
            for entry in listeners {
                println!(
                    "{:<10} {:<10} {:<30}",
                    entry.port, entry.owning_pid, entry.local_addr
                );
            }
        }
        Some(Commands::Services { name }) => {
            let status = get_service_status(&name);
            println!("Service '{}' status: {:?}", name, status);
        }
        Some(Commands::Hosts { path }) => {
            let hosts_path = path
                .map(PathBuf::from)
                .unwrap_or_else(HostsFileManager::default_hosts_path);
            println!("Reading hosts from: {}", hosts_path.display());
            if hosts_path.exists() {
                let content = std::fs::read_to_string(&hosts_path)?;
                let entries = HostsFileManager::parse_content(&content);
                println!(
                    "{:<16} {:<30} {:<10} {:<10}",
                    "IP", "HOSTNAME", "ENABLED", "MANAGED"
                );
                println!("{}", "-".repeat(70));
                for e in entries {
                    println!(
                        "{:<16} {:<30} {:<10} {:<10}",
                        e.ip, e.hostname, e.is_enabled, e.is_managed
                    );
                }
            } else {
                println!("Hosts file does not exist at {}", hosts_path.display());
            }
        }
        Some(Commands::Ssl { domain }) => {
            if let Some(d) = domain {
                println!("Generating SAN SSL certificate for '{}'...", d);
                let (cert, key) = LocalCertManager::generate_domain_cert(&d)?;
                println!("Domain Certificate: {}", cert.display());
                println!("Domain Private Key: {}", key.display());
            } else {
                println!("Checking / Generating Root CA...");
                let (cert_pem, _) = LocalCertManager::get_or_create_root_ca()?;
                println!(
                    "Root CA Location: {}",
                    LocalCertManager::root_ca_cert_path().display()
                );
                println!(
                    "Root CA Cert (preview):\n{}",
                    cert_pem.lines().take(4).collect::<Vec<_>>().join("\n")
                );
                println!("...");
            }
        }
        Some(Commands::Nginx { action }) => {
            let nginx_dir = NginxManager::default_nginx_dir();
            match action.to_lowercase().as_str() {
                "test" => {
                    println!("Testing Nginx config at {}...", nginx_dir.display());
                    let (ok, output) = NginxManager::validate_config(&nginx_dir).await?;
                    println!(
                        "Result: {}\n{}",
                        if ok { "PASSED" } else { "FAILED" },
                        output
                    );
                }
                "reload" => {
                    println!("Reloading Nginx...");
                    let (ok, output) = NginxManager::reload_nginx(&nginx_dir).await?;
                    println!(
                        "Result: {}\n{}",
                        if ok { "SUCCESS" } else { "FAILED" },
                        output
                    );
                }
                "stop" => {
                    println!("Stopping Nginx gracefully...");
                    let (ok, output) = NginxManager::stop_graceful(&nginx_dir).await?;
                    println!(
                        "Result: {}\n{}",
                        if ok { "STOPPED" } else { "FAILED" },
                        output
                    );
                }
                _ => {
                    println!("Unknown nginx action: {}. Use: test, reload, stop", action);
                }
            }
        }
        Some(Commands::Tools { action, id }) => {
            let store = InstalledToolStore::new();
            let manifests = CatalogLoader::load_embedded()?;

            match action.as_deref() {
                Some("install") => {
                    let tool_id = id.ok_or_else(|| {
                        anyhow::anyhow!("Please provide --id <tool_id> to install")
                    })?;
                    let manifest = manifests
                        .into_iter()
                        .find(|m| m.id.eq_ignore_ascii_case(&tool_id))
                        .ok_or_else(|| {
                            anyhow::anyhow!("Tool '{}' not found in catalog", tool_id)
                        })?;
                    let version = manifest
                        .versions
                        .first()
                        .ok_or_else(|| anyhow::anyhow!("No version available for {}", tool_id))?;

                    println!(
                        "Installing {} v{}...",
                        manifest.display_name, version.version
                    );
                    ToolInstaller::install(&manifest, version, &store, |pct| {
                        print!(
                            "\rProgress: [{:<20}] {:.1}%",
                            "=".repeat((pct / 5.0) as usize),
                            pct
                        );
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    })
                    .await?;
                    println!("\nInstallation complete.");
                }
                Some("uninstall") => {
                    let tool_id = id.ok_or_else(|| {
                        anyhow::anyhow!("Please provide --id <tool_id> to uninstall")
                    })?;
                    if let Some(record) = store.get_active_tool(&tool_id)? {
                        println!(
                            "Uninstalling {} v{}...",
                            record.display_name, record.version
                        );
                        ToolInstaller::uninstall(&record, &store)?;
                        println!("Uninstall complete.");
                    } else {
                        println!("Tool '{}' is not installed.", tool_id);
                    }
                }
                _ => {
                    println!(
                        "{:<12} {:<32} {:<12} {:<15}",
                        "ID", "DISPLAY NAME", "CATEGORY", "INSTALLED"
                    );
                    println!("{}", "-".repeat(75));
                    for m in manifests {
                        let installed_opt = store.get_active_tool(&m.id)?;
                        let status_str = installed_opt
                            .map(|i| format!("v{}", i.version))
                            .unwrap_or_else(|| "Not Installed".into());
                        println!(
                            "{:<12} {:<32} {:<12} {:<15}",
                            m.id, m.display_name, m.category, status_str
                        );
                    }
                }
            }
        }
        Some(Commands::Profile { switch }) => {
            let store = ProfileStore::new();
            if let Some(target) = switch {
                if store.set_active_profile(&target)? {
                    println!("Switched active profile to '{}'.", target);
                } else {
                    println!("Profile '{}' not found.", target);
                }
            } else {
                let config = store.load_config()?;
                println!("Active Profile: {}\n", config.active_profile_id);
                println!("{:<15} {:<35} {:<30}", "ID", "NAME", "SERVICES");
                println!("{}", "-".repeat(80));
                for p in config.profiles {
                    let marker = if p.id.eq_ignore_ascii_case(&config.active_profile_id) {
                        "* "
                    } else {
                        "  "
                    };
                    println!(
                        "{}{:<13} {:<35} {:<30}",
                        marker,
                        p.id,
                        p.name,
                        p.service_ids.join(", ")
                    );
                }
            }
        }
        Some(Commands::Shadow) => {
            println!("Analyzing PATH Shadow collisions...\n");
            let results = PathShadowAnalyzer::analyze(None, None)?;
            if results.is_empty() {
                println!("No managed binaries detected in User PATH.");
            } else {
                for r in results {
                    let status = if r.is_shadowed { "[COLLISION]" } else { "[OK]" };
                    println!("{} {}", status, r.binary_name);
                    println!(
                        "  User Location:   {}",
                        r.user_path_location.unwrap_or_default()
                    );
                    println!(
                        "  System Location: {}",
                        r.system_path_location.unwrap_or_default()
                    );
                    println!("  Advice:          {}\n", r.recommendation);
                }
            }
        }
        Some(Commands::Status) => {
            let profile_store = ProfileStore::new();
            let active_profile = profile_store.get_active_profile()?;
            let tracker = ProcessTracker::new();
            let tracked = tracker.get_tracked_processes()?;

            println!("=== Dotnet Development Engine Status ===");
            println!(
                "Active Profile: {} ({})",
                active_profile.name, active_profile.id
            );
            println!(
                "Managed Services: {}",
                active_profile.service_ids.join(", ")
            );
            println!("\nTracked Background Processes: {}", tracked.len());
            for t in tracked {
                println!(
                    "  • {:<12} (PID: {:<6}) at {}",
                    t.service_id, t.pid, t.executable_path
                );
            }
        }
        Some(Commands::Autostart) => {
            let is_scheduled = TaskScheduler::is_task_scheduled(
                dotnet_windows::task_scheduler::DEFAULT_TASK_NAME,
            )?;
            println!(
                "Task Scheduler elevated logon autostart: {}",
                if is_scheduled {
                    "ACTIVE"
                } else {
                    "NOT CONFIGURED"
                }
            );
        }
        Some(Commands::Projects { action }) => {
            let manager = ProjectManager::new();
            let site_gen = NginxSiteGenerator::new();

            match action.unwrap_or(ProjectAction::List) {
                ProjectAction::List => {
                    let projects = manager.get_projects()?;
                    if projects.is_empty() {
                        println!("No registered projects found. Use 'dotnet projects add <path>' to register one.");
                    } else {
                        println!(
                            "{:<20} {:<12} {:<24} {:<8} {:<30}",
                            "NAME", "FRAMEWORK", "DOMAIN", "SSL", "PATH"
                        );
                        println!("{}", "-".repeat(94));
                        for p in projects {
                            println!(
                                "{:<20} {:<12} {:<24} {:<8} {:<30}",
                                p.name,
                                p.framework,
                                p.effective_host(),
                                if p.ssl_enabled { "Yes" } else { "No" },
                                p.path
                            );
                        }
                    }
                }
                ProjectAction::Add {
                    path,
                    name,
                    host,
                    port,
                    framework,
                    ssl,
                } => {
                    let proj_path = Path::new(&path);
                    if !proj_path.is_dir() {
                        return Err(anyhow::anyhow!("Directory '{}' does not exist", path));
                    }

                    let detected = FrameworkDetector::detect(proj_path);
                    let final_name = name.unwrap_or(detected.suggested_name);
                    let final_framework = framework.unwrap_or(detected.framework);
                    let final_host = host.unwrap_or(detected.suggested_host);
                    let final_port = port.or(detected.suggested_port);
                    let final_dev_cmd = if !detected.dev_command.is_empty() {
                        Some(detected.dev_command)
                    } else {
                        None
                    };

                    let mut project = ProjectInfo::new(&final_name, &path);
                    project.framework = final_framework;
                    project.nginx_host = Some(final_host.clone());
                    project.dev_port = final_port;
                    project.public_directory = detected.public_directory;
                    project.dev_command = final_dev_cmd;
                    project.ssl_enabled = ssl;

                    manager.add_or_update_project(project.clone())?;
                    println!("Project '{}' registered successfully.", final_name);

                    // Create Nginx site & register hosts
                    match site_gen.create_or_update_site(&project, ssl, true).await {
                        Ok(res) => println!("{}", res.message),
                        Err(e) => {
                            eprintln!("Notice: Could not automatically route Nginx site: {e}")
                        }
                    }
                }
                ProjectAction::Remove {
                    target,
                    delete_site,
                } => {
                    if let Some(proj) = manager.get_project_by_identifier(&target)? {
                        if delete_site {
                            let _ = site_gen.remove_site(&proj).await;
                            println!("Removed Nginx site and hosts entry for '{}'.", proj.name);
                        }
                        manager.delete_project(&proj.id)?;
                        println!("Project '{}' removed.", proj.name);
                    } else {
                        println!("Project '{}' not found.", target);
                    }
                }
                ProjectAction::Run { target } => {
                    let proj = manager
                        .get_project_by_identifier(&target)?
                        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", target))?;
                    println!(
                        "Launching dev server for '{}' ({:?})...",
                        proj.name, proj.dev_command
                    );
                    ProjectManager::launch_dev_command(&proj)?;
                }
                ProjectAction::Explore { target } => {
                    let proj = manager
                        .get_project_by_identifier(&target)?
                        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", target))?;
                    ProjectManager::open_in_explorer(Path::new(&proj.path))?;
                }
                ProjectAction::Terminal { target } => {
                    let proj = manager
                        .get_project_by_identifier(&target)?
                        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", target))?;
                    ProjectManager::open_terminal(Path::new(&proj.path))?;
                }
                ProjectAction::Code { target } => {
                    let proj = manager
                        .get_project_by_identifier(&target)?
                        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", target))?;
                    ProjectManager::open_in_vscode(Path::new(&proj.path))?;
                }
                ProjectAction::Domain {
                    target,
                    domain,
                    ssl,
                } => {
                    let mut proj = manager
                        .get_project_by_identifier(&target)?
                        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", target))?;

                    // Remove old site config
                    let _ = site_gen.remove_site(&proj).await;

                    // Update domain
                    proj.nginx_host = Some(domain.clone());
                    proj.ssl_enabled = ssl;
                    manager.add_or_update_project(proj.clone())?;

                    // Generate new site config
                    let res = site_gen.create_or_update_site(&proj, ssl, true).await?;
                    println!("{}", res.message);
                }
            }
        }
        Some(Commands::PhpPool { action, size, port }) => {
            let pool_mgr = PhpPoolManager::new();
            match action.to_lowercase().as_str() {
                "status" => {
                    let workers = pool_mgr.get_pool_status(size, port)?;
                    println!("{:<10} {:<10} {:<15}", "PORT", "PID", "STATUS");
                    println!("{}", "-".repeat(35));
                    for w in workers {
                        let pid_str = w
                            .pid
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "-".to_string());
                        let status_str = if w.is_running { "RUNNING" } else { "STOPPED" };
                        println!("{:<10} {:<10} {:<15}", w.port, pid_str, status_str);
                    }
                }
                "stop" => {
                    pool_mgr.stop_pool(size, port)?;
                    println!("Stopped PHP FastCGI pool ({} workers).", size);
                }
                "start" => {
                    // Try to locate php-cgi.exe in PATH or tools directory
                    let tools_php = dotnet_core::persistence::AppPaths::tools_dir()
                        .join("php")
                        .join("current")
                        .join("php-cgi.exe");
                    let php_cgi = if tools_php.is_file() {
                        tools_php
                    } else {
                        which::which("php-cgi.exe")
                            .or_else(|_| which::which("php-cgi"))
                            .map_err(|_| {
                                anyhow::anyhow!("php-cgi.exe not found in PATH or tools directory")
                            })?
                    };

                    let workers = pool_mgr.start_pool(&php_cgi, size, port)?;
                    println!("Started PHP FastCGI pool with {} workers:", workers.len());
                    for w in workers {
                        println!("  • Port {}: PID {:?}", w.port, w.pid.unwrap_or_default());
                    }
                }
                _ => {
                    println!("Unknown action '{}'. Use: status, start, stop", action);
                }
            }
        }
        None => {
            println!("Dotnet Rust CLI Engine v{}", env!("CARGO_PKG_VERSION"));
            println!("Run with --help to view available commands.");
        }
    }

    Ok(())
}
