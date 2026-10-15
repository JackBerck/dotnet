//! Portable non-elevated database initializer for MySQL and PostgreSQL.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

pub struct DatabaseInitializer;

impl DatabaseInitializer {
    /// Resolve data directory for a service ID (%LOCALAPPDATA%\Dotnet\data\<service_id>).
    #[must_use]
    pub fn get_data_directory(service_id: &str) -> PathBuf {
        AppPaths::get_path("data").join(service_id.to_lowercase())
    }

    /// Check if database is already initialized.
    #[must_use]
    pub fn is_initialized(service_id: &str) -> bool {
        let data_dir = Self::get_data_directory(service_id);
        if !data_dir.is_dir() {
            return false;
        }

        let id = service_id.to_lowercase();
        if id == "mysql" || id == "mariadb" {
            data_dir.join("mysql").is_dir() || data_dir.join("ibdata1").is_file()
        } else if id == "postgres" || id == "postgresql" {
            data_dir.join("PG_VERSION").is_file()
        } else {
            // Fallback: check if directory has any files/folders
            match fs::read_dir(&data_dir) {
                Ok(mut entries) => entries.next().is_some(),
                Err(_) => false,
            }
        }
    }

    /// Run initialization for MySQL or PostgreSQL.
    pub async fn initialize_database(service_id: &str, binary_path: &Path) -> Result<String> {
        let id = service_id.to_lowercase();
        let data_dir = Self::get_data_directory(&id);

        if Self::is_initialized(&id) {
            return Ok(format!(
                "Database '{id}' is already initialized in {}",
                data_dir.display()
            ));
        }

        fs::create_dir_all(&data_dir)?;

        if id == "mysql" || id == "mariadb" {
            Self::initialize_mysql(binary_path, &data_dir).await
        } else if id == "postgres" || id == "postgresql" {
            Self::initialize_postgres(binary_path, &data_dir).await
        } else {
            Err(CoreError::Config(format!(
                "Unknown database service type '{service_id}'"
            )))
        }
    }

    /// Initialize MySQL using `--initialize-insecure`.
    pub async fn initialize_mysql(mysqld_path: &Path, data_dir: &Path) -> Result<String> {
        if !mysqld_path.is_file() {
            return Err(CoreError::Config(format!(
                "mysqld binary not found at {}",
                mysqld_path.display()
            )));
        }

        let data_dir_arg = format!("--datadir={}", data_dir.display());
        let args = ["--initialize-insecure", &data_dir_arg, "--console"];

        tracing::info!(
            "DatabaseInitializer: Initializing MySQL in {} via {}",
            data_dir.display(),
            mysqld_path.display()
        );

        let (success, output) =
            Self::run_init_command(mysqld_path, &args, Duration::from_secs(60)).await?;
        if !success {
            return Err(CoreError::Service(format!(
                "MySQL initialization failed:\n{output}"
            )));
        }

        tracing::info!(
            "DatabaseInitializer: MySQL successfully initialized in {}",
            data_dir.display()
        );
        Ok(format!(
            "MySQL successfully initialized in {}",
            data_dir.display()
        ))
    }

    /// Initialize PostgreSQL using `initdb.exe`.
    pub async fn initialize_postgres(
        postgres_or_initdb_path: &Path,
        data_dir: &Path,
    ) -> Result<String> {
        let initdb_path = if postgres_or_initdb_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| {
                name.eq_ignore_ascii_case("initdb.exe") || name.eq_ignore_ascii_case("initdb")
            }) {
            postgres_or_initdb_path.to_path_buf()
        } else {
            let parent = postgres_or_initdb_path.parent().unwrap_or(Path::new(""));
            let candidate = parent.join("initdb.exe");
            if candidate.is_file() {
                candidate
            } else {
                which::which("initdb.exe")
                    .or_else(|_| which::which("initdb"))
                    .unwrap_or(candidate)
            }
        };

        if !initdb_path.is_file() {
            return Err(CoreError::Config(format!(
                "initdb binary not found at {}",
                initdb_path.display()
            )));
        }

        let data_dir_str = data_dir.to_string_lossy();
        let args = [
            "-D",
            &data_dir_str,
            "-U",
            "postgres",
            "-A",
            "trust",
            "--encoding=UTF8",
        ];

        tracing::info!(
            "DatabaseInitializer: Initializing PostgreSQL in {} via {}",
            data_dir.display(),
            initdb_path.display()
        );

        let (success, output) =
            Self::run_init_command(&initdb_path, &args, Duration::from_secs(60)).await?;
        if !success {
            return Err(CoreError::Service(format!(
                "PostgreSQL initdb failed:\n{output}"
            )));
        }

        tracing::info!(
            "DatabaseInitializer: PostgreSQL successfully initialized in {}",
            data_dir.display()
        );
        Ok(format!(
            "PostgreSQL successfully initialized in {}",
            data_dir.display()
        ))
    }

    async fn run_init_command(
        exe: &Path,
        args: &[&str],
        timeout: Duration,
    ) -> Result<(bool, String)> {
        let mut child = Command::new(exe)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()?;

        let mut stdout_pipe = child
            .stdout
            .take()
            .ok_or_else(|| CoreError::Service("Failed to open child process stdout".to_string()))?;
        let mut stderr_pipe = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::Service("Failed to open child process stderr".to_string()))?;

        let read_pipes = async {
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();
            let _ = tokio::join!(
                stdout_pipe.read_to_end(&mut stdout_buf),
                stderr_pipe.read_to_end(&mut stderr_buf)
            );
            (
                String::from_utf8_lossy(&stdout_buf).to_string(),
                String::from_utf8_lossy(&stderr_buf).to_string(),
            )
        };

        tokio::select! {
            output_res = read_pipes => {
                let status = child.wait().await?;
                let (stdout, stderr) = output_res;
                let full = format!("{}\n{}", stdout, stderr).trim().to_string();
                Ok((status.success(), full))
            }
            _ = tokio::time::sleep(timeout) => {
                let _ = child.kill().await;
                Err(CoreError::Service("Database initialization command timed out".to_string()))
            }
        }
    }

    /// Build standard runtime CLI arguments for launching managed database servers.
    #[must_use]
    pub fn build_runtime_arguments(service_id: &str, port: u16) -> Vec<String> {
        let id = service_id.to_lowercase();
        let data_dir = Self::get_data_directory(&id);
        let data_dir_str = data_dir.to_string_lossy().to_string();

        match id.as_str() {
            "mysql" | "mariadb" => vec![
                format!("--datadir={data_dir_str}"),
                format!("--port={port}"),
                "--console".to_string(),
            ],
            "postgres" | "postgresql" => vec![
                "-D".to_string(),
                data_dir_str,
                "-p".to_string(),
                port.to_string(),
            ],
            "redis" | "memurai" => vec!["--port".to_string(), port.to_string()],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_runtime_arguments() {
        let mysql_args = DatabaseInitializer::build_runtime_arguments("mysql", 3306);
        assert!(mysql_args.iter().any(|a| a.starts_with("--datadir=")));
        assert!(mysql_args.contains(&"--port=3306".to_string()));
        assert!(mysql_args.contains(&"--console".to_string()));

        let pg_args = DatabaseInitializer::build_runtime_arguments("postgres", 5432);
        assert!(pg_args.contains(&"-D".to_string()));
        assert!(pg_args.contains(&"-p".to_string()));
        assert!(pg_args.contains(&"5432".to_string()));

        let redis_args = DatabaseInitializer::build_runtime_arguments("redis", 6379);
        assert_eq!(redis_args, vec!["--port".to_string(), "6379".to_string()]);
    }

    #[test]
    fn test_is_initialized_detection() {
        let dir = tempdir().expect("tempdir");
        std::env::set_var("DOTNET_DATA_DIR", dir.path().to_str().unwrap());

        // Initially not initialized
        assert!(!DatabaseInitializer::is_initialized("mysql"));
        assert!(!DatabaseInitializer::is_initialized("postgres"));

        // Create mock MySQL data directory
        let mysql_dir = DatabaseInitializer::get_data_directory("mysql");
        fs::create_dir_all(mysql_dir.join("mysql")).expect("mkdir mysql");
        assert!(DatabaseInitializer::is_initialized("mysql"));

        // Create mock PostgreSQL data directory
        let pg_dir = DatabaseInitializer::get_data_directory("postgres");
        fs::create_dir_all(&pg_dir).expect("mkdir pg");
        fs::write(pg_dir.join("PG_VERSION"), "16\n").expect("write PG_VERSION");
        assert!(DatabaseInitializer::is_initialized("postgres"));
    }
}
