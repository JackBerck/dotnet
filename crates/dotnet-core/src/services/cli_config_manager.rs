//! CLI Configuration Manager for global settings (Git, npm, Composer).

use crate::error::{CoreError, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

pub struct CliConfigManager;

impl CliConfigManager {
    async fn run_cli_command(
        exe: &str,
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
            .ok_or_else(|| CoreError::Service("Failed to open child stdout".to_string()))?;
        let mut stderr_pipe = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::Service("Failed to open child stderr".to_string()))?;

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
                Err(CoreError::Service("CLI command timed out".to_string()))
            }
        }
    }

    /// Read global Git configuration value.
    pub async fn get_git_config(key: &str) -> Result<Option<String>> {
        let args = ["config", "--global", key];
        match Self::run_cli_command("git.exe", &args, Duration::from_secs(5)).await {
            Ok((true, out)) if !out.trim().is_empty() => Ok(Some(out.trim().to_string())),
            _ => Ok(None),
        }
    }

    /// Set global Git configuration value.
    pub async fn set_git_config(key: &str, value: &str) -> Result<()> {
        let args = ["config", "--global", key, value];
        let (ok, out) = Self::run_cli_command("git.exe", &args, Duration::from_secs(5)).await?;
        if !ok {
            return Err(CoreError::Service(format!(
                "Failed to set Git config: {out}"
            )));
        }
        Ok(())
    }

    /// Read global npm configuration value.
    pub async fn get_npm_config(key: &str) -> Result<Option<String>> {
        let args = ["config", "get", key];
        match Self::run_cli_command("npm.cmd", &args, Duration::from_secs(5)).await {
            Ok((true, out)) if !out.trim().is_empty() && out.trim() != "undefined" => {
                Ok(Some(out.trim().to_string()))
            }
            _ => Ok(None),
        }
    }

    /// Set global npm configuration value.
    pub async fn set_npm_config(key: &str, value: &str) -> Result<()> {
        let args = ["config", "set", key, value];
        let (ok, out) = Self::run_cli_command("npm.cmd", &args, Duration::from_secs(5)).await?;
        if !ok {
            return Err(CoreError::Service(format!(
                "Failed to set npm config: {out}"
            )));
        }
        Ok(())
    }
}
