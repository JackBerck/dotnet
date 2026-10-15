use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "dotnet", version, about = "High-performance standalone dev manager in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect active ports and detect conflicts
    Ports,
    /// List available and installed tools
    Tools,
    /// Manage development services
    Services {
        #[arg(short, long)]
        action: String,
    },
    /// Manage virtual hosts and domains
    Domains,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ports) => {
            info!("Inspecting active TCP ports via native Windows IP helper...");
            let listeners = dotnet_windows::get_active_tcp_listeners()?;
            println!("{:<10} {:<10} {:<25}", "PORT", "PID", "ADDRESS");
            println!("{}", "-".repeat(45));
            for entry in listeners {
                println!("{:<10} {:<10} {:<25}", entry.port, entry.owning_pid, entry.local_addr);
            }
        }
        Some(Commands::Tools) => {
            println!("Tool catalog listing (stub - migration in progress)");
        }
        Some(Commands::Services { action }) => {
            println!("Service command: {}", action);
        }
        Some(Commands::Domains) => {
            println!("Domain management (stub - migration in progress)");
        }
        None => {
            println!("Dotnet Rust CLI engine v{}", env!("CARGO_PKG_VERSION"));
            println!("Run with --help to see available commands.");
        }
    }

    Ok(())
}
