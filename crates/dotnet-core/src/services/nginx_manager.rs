//! Asynchronous Nginx Manager: validation, graceful reload, stop, and log tailing.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

pub struct NginxManager;

impl NginxManager {
    /// Resolve default Nginx root directory.
    pub fn default_nginx_dir() -> PathBuf {
        let tools_nginx = AppPaths::tools_dir().join("nginx").join("current");
        if tools_nginx.exists() {
            tools_nginx
        } else {
            PathBuf::from(r"C:\tools\nginx")
        }
    }

    /// Resolve nginx.exe path.
    pub fn get_nginx_exe<P: AsRef<Path>>(nginx_dir: P) -> PathBuf {
        nginx_dir.as_ref().join("nginx.exe")
    }

    /// Run nginx command asynchronously with timeout and parallel stream draining.
    pub async fn run_command<P: AsRef<Path>>(
        nginx_dir: P,
        args: &[&str],
        timeout: Duration,
    ) -> Result<(bool, String)> {
        let nginx_dir = nginx_dir.as_ref();
        let exe = Self::get_nginx_exe(nginx_dir);

        if !exe.exists() {
            return Err(CoreError::Service(format!(
                "nginx.exe not found at {}",
                exe.display()
            )));
        }

        let mut child = Command::new(&exe)
            .args(args)
            .current_dir(nginx_dir)
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

        let result = tokio::select! {
            output_res = read_pipes => {
                let status = child.wait().await?;
                let (stdout, stderr) = output_res;
                let full_output = format!("{}\n{}", stdout, stderr).trim().to_string();
                Ok((status.success(), full_output))
            }
            _ = tokio::time::sleep(timeout) => {
                let _ = child.kill().await;
                Err(CoreError::Service("Nginx command timed out".to_string()))
            }
        };

        result
    }

    /// Run `nginx -t` to validate configuration syntax.
    pub async fn validate_config<P: AsRef<Path>>(nginx_dir: P) -> Result<(bool, String)> {
        let dir = nginx_dir.as_ref();
        let prefix = dir.to_string_lossy();
        let args = ["-p", &prefix, "-t"];

        let (success, output) = Self::run_command(dir, &args, Duration::from_secs(8)).await?;
        let is_ok = success
            && (output.to_lowercase().contains("syntax is ok")
                || output.to_lowercase().contains("test is successful"));

        Ok((is_ok, output))
    }

    /// Reload Nginx configuration gracefully (`nginx -s reload`).
    pub async fn reload_nginx<P: AsRef<Path>>(nginx_dir: P) -> Result<(bool, String)> {
        let dir = nginx_dir.as_ref();

        // 1. Always validate before reload
        let (valid, val_output) = Self::validate_config(dir).await?;
        if !valid {
            return Ok((
                false,
                format!("Validation failed. Reload aborted:\n{}", val_output),
            ));
        }

        // 2. Perform reload
        let prefix = dir.to_string_lossy();
        let args = ["-p", &prefix, "-s", "reload"];
        Self::run_command(dir, &args, Duration::from_secs(8)).await
    }

    /// Stop Nginx gracefully (`nginx -s quit`).
    pub async fn stop_graceful<P: AsRef<Path>>(nginx_dir: P) -> Result<(bool, String)> {
        let dir = nginx_dir.as_ref();
        let prefix = dir.to_string_lossy();
        let args = ["-p", &prefix, "-s", "quit"];
        Self::run_command(dir, &args, Duration::from_secs(8)).await
    }

    /// Tail the last `max_lines` from the Nginx error log file.
    pub fn tail_error_log<P: AsRef<Path>>(nginx_dir: P, max_lines: usize) -> Result<Vec<String>> {
        let log_file = nginx_dir.as_ref().join("logs").join("error.log");
        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&log_file)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        if lines.len() <= max_lines {
            Ok(lines)
        } else {
            Ok(lines[lines.len() - max_lines..].to_vec())
        }
    }
}
