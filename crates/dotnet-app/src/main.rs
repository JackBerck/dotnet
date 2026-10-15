//! Dotnet Desktop GUI & System Tray runner

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Dotnet GUI & Tray Application (migration phase)...");
    println!("Dotnet Rust Desktop Engine initialized.");
    Ok(())
}
