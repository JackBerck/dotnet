use clap::{Parser, Subcommand};
use dotnet_core::services::{HostsFileManager, LocalCertManager, NginxManager};
use dotnet_windows::{get_active_tcp_listeners, get_service_status, TaskScheduler};
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
    /// Inspect Task Scheduler autostart status
    Autostart,
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
                .map(std::path::PathBuf::from)
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
        None => {
            println!("Dotnet Rust CLI Engine v{}", env!("CARGO_PKG_VERSION"));
            println!("Run with --help to view available commands.");
        }
    }

    Ok(())
}
