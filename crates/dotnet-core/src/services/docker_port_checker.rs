//! Docker Compose port conflict inspector and YAML fallback analyzer.

use crate::error::{CoreError, Result};
use dotnet_windows::get_active_tcp_listeners;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

static PORT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":(\d+)->").expect("valid regex"));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockerPortConflict {
    pub service_name: String,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
    pub host_ip: String,
    pub is_conflicting: bool,
    pub conflict_owner: String,
    pub is_same_project_container: bool,
    pub suggested_override_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeReport {
    pub compose_file: PathBuf,
    pub is_docker_available: bool,
    pub message: String,
    pub conflicts: Vec<DockerPortConflict>,
}

impl DockerComposeReport {
    #[must_use]
    pub fn has_conflicts(&self) -> bool {
        self.conflicts
            .iter()
            .any(|c| c.is_conflicting && !c.is_same_project_container)
    }

    #[must_use]
    pub fn total_ports_checked(&self) -> usize {
        self.conflicts.len()
    }
}

pub struct DockerPortChecker;

impl DockerPortChecker {
    /// Check whether Docker CLI binary is installed and reachable via PATH.
    #[must_use]
    pub fn is_docker_installed() -> bool {
        which::which("docker").is_ok()
    }

    /// Analyze a docker-compose.yml file for active host port collisions.
    pub async fn check_compose_file(compose_path: &Path) -> Result<DockerComposeReport> {
        let mut report = DockerComposeReport {
            compose_file: compose_path.to_path_buf(),
            is_docker_available: Self::is_docker_installed(),
            message: String::new(),
            conflicts: Vec::new(),
        };

        if !compose_path.is_file() {
            return Err(CoreError::Config(format!(
                "Compose file not found at {}",
                compose_path.display()
            )));
        }

        let work_dir = compose_path.parent().unwrap_or(Path::new("."));

        if !report.is_docker_available {
            report.message = "Docker CLI not found in PATH. Using native fallback parser.".into();
            report.conflicts = Self::parse_compose_fallback(compose_path)?;
            return Ok(report);
        }

        // Run `docker compose config --format json`
        let config_args = [
            "compose",
            "-f",
            &compose_path.to_string_lossy(),
            "config",
            "--format",
            "json",
        ];

        match Self::run_command("docker", &config_args, work_dir, Duration::from_secs(10)).await {
            Ok((true, config_json)) if !config_json.trim().is_empty() => {
                // Get running project ports to exclude self-containers
                let ps_args = [
                    "compose",
                    "-f",
                    &compose_path.to_string_lossy(),
                    "ps",
                    "--format",
                    "json",
                ];
                let own_ports =
                    match Self::run_command("docker", &ps_args, work_dir, Duration::from_secs(5))
                        .await
                    {
                        Ok((true, ps_json)) => Self::parse_running_project_ports(&ps_json),
                        _ => HashSet::new(),
                    };

                report.conflicts = Self::parse_compose_json(&config_json, &own_ports)?;
                report.message = if report.has_conflicts() {
                    "Port conflicts detected.".into()
                } else {
                    "All ports available.".into()
                };
            }
            _ => {
                report.message =
                    "docker compose config command failed or returned empty. Used native fallback parser."
                        .into();
                report.conflicts = Self::parse_compose_fallback(compose_path)?;
            }
        }

        Ok(report)
    }

    /// Parse ports specification string into `(host_port, container_port, protocol, ip)`.
    pub fn parse_port_string(spec: &str) -> Vec<(u16, u16, String, String)> {
        let mut list = Vec::new();
        let trimmed = spec.trim().trim_matches(|c| c == '\'' || c == '"');
        if trimmed.is_empty() {
            return list;
        }

        let (proto, port_spec) = if let Some((p_spec, p_proto)) = trimmed.split_once('/') {
            (p_proto.to_lowercase(), p_spec)
        } else {
            ("tcp".to_string(), trimmed)
        };

        let parts: Vec<&str> = port_spec.split(':').collect();
        let (ip, host_part, cont_part) = match parts.len() {
            3 => (parts[0], parts[1], parts[2]),
            2 => ("0.0.0.0", parts[0], parts[1]),
            1 => ("0.0.0.0", parts[0], parts[0]),
            _ => return list,
        };

        let host_range = Self::expand_port_range(host_part);
        let cont_range = Self::expand_port_range(cont_part);

        for (i, &h) in host_range.iter().enumerate() {
            let c = if i < cont_range.len() {
                cont_range[i]
            } else if !cont_range.is_empty() {
                cont_range[0]
            } else {
                h
            };
            list.push((h, c, proto.clone(), ip.to_string()));
        }

        list
    }

    /// Expand port range like `"3000-3002"` into `[3000, 3001, 3002]` or single `"80"` into `[80]`.
    pub fn expand_port_range(range_str: &str) -> Vec<u16> {
        let mut result = Vec::new();
        let trimmed = range_str.trim();
        if trimmed.is_empty() {
            return result;
        }

        if let Some((start_s, end_s)) = trimmed.split_once('-') {
            if let (Ok(start), Ok(end)) =
                (start_s.trim().parse::<u16>(), end_s.trim().parse::<u16>())
            {
                if start <= end {
                    for p in start..=end {
                        result.push(p);
                    }
                    return result;
                }
            }
        }

        if let Ok(single) = trimmed.parse::<u16>() {
            result.push(single);
        }

        result
    }

    /// Line-by-line YAML parser for docker-compose.yml files when Docker CLI is absent.
    pub fn parse_compose_fallback(compose_file: &Path) -> Result<Vec<DockerPortConflict>> {
        let mut results = Vec::new();
        if !compose_file.is_file() {
            return Ok(results);
        }

        let content = fs::read_to_string(compose_file)?;
        let mut current_service = String::new();
        let mut in_ports_section = false;
        let mut ports_indent = 0;

        let empty_set = HashSet::new();

        for raw_line in content.lines() {
            let line = raw_line;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = line.chars().take_while(|c| c.is_whitespace()).count();

            // Service declaration: e.g. "  web:"
            if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                current_service = trimmed.trim_end_matches(':').to_string();
                in_ports_section = false;
                continue;
            }

            // Ports section: e.g. "    ports:"
            if trimmed == "ports:" {
                in_ports_section = true;
                ports_indent = indent;
                continue;
            }

            if in_ports_section {
                if indent <= ports_indent && !trimmed.starts_with('-') {
                    in_ports_section = false;
                    continue;
                }

                if trimmed.starts_with('-') {
                    let port_spec = trimmed
                        .trim_start_matches('-')
                        .trim()
                        .trim_matches(|c| c == '\'' || c == '"');

                    let parsed = Self::parse_port_string(port_spec);
                    for (h, c, proto, ip) in parsed {
                        results.push(Self::evaluate_port_conflict(
                            &current_service,
                            h,
                            c,
                            &proto,
                            &ip,
                            &empty_set,
                        ));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Parse official JSON format returned by `docker compose config --format json`.
    pub fn parse_compose_json(
        json_str: &str,
        own_ports: &HashSet<u16>,
    ) -> Result<Vec<DockerPortConflict>> {
        let mut results = Vec::new();
        let v: serde_json::Value = serde_json::from_str(json_str)?;

        let services = match v.get("services").and_then(|s| s.as_object()) {
            Some(s) => s,
            None => return Ok(results),
        };

        for (service_name, service_obj) in services {
            let ports = match service_obj.get("ports").and_then(|p| p.as_array()) {
                Some(p) => p,
                None => continue,
            };

            for port_elem in ports {
                if let Some(obj) = port_elem.as_object() {
                    let host_ip = obj
                        .get("host_ip")
                        .and_then(|i| i.as_str())
                        .unwrap_or("0.0.0.0");
                    let proto = obj
                        .get("protocol")
                        .and_then(|pr| pr.as_str())
                        .unwrap_or("tcp");
                    let container_port = obj
                        .get("target")
                        .and_then(|t| t.as_u64())
                        .map(|t| t as u16)
                        .unwrap_or(0);

                    if let Some(pub_val) = obj.get("published") {
                        let pub_str = pub_val.to_string();
                        let clean_pub = pub_str.trim_matches('"');
                        for port in Self::expand_port_range(clean_pub) {
                            results.push(Self::evaluate_port_conflict(
                                service_name,
                                port,
                                container_port,
                                proto,
                                host_ip,
                                own_ports,
                            ));
                        }
                    }
                } else if let Some(s) = port_elem.as_str() {
                    for (h, c, proto, ip) in Self::parse_port_string(s) {
                        results.push(Self::evaluate_port_conflict(
                            service_name,
                            h,
                            c,
                            &proto,
                            &ip,
                            own_ports,
                        ));
                    }
                }
            }
        }

        Ok(results)
    }

    fn parse_running_project_ports(ps_json: &str) -> HashSet<u16> {
        let mut ports = HashSet::new();
        for cap in PORT_REGEX.captures_iter(ps_json) {
            if let Some(m) = cap.get(1) {
                if let Ok(p) = m.as_str().parse::<u16>() {
                    ports.insert(p);
                }
            }
        }
        ports
    }

    fn evaluate_port_conflict(
        service_name: &str,
        host_port: u16,
        container_port: u16,
        proto: &str,
        host_ip: &str,
        own_ports: &HashSet<u16>,
    ) -> DockerPortConflict {
        let is_own = own_ports.contains(&host_port);

        // Query active Win32 TCP listeners
        let listeners = get_active_tcp_listeners().unwrap_or_default();
        let active_listener = listeners.iter().find(|l| l.port == host_port);

        let in_use = active_listener.is_some();
        let is_conflicting = in_use && !is_own;

        let conflict_owner = match active_listener {
            Some(l) if !is_own => format!("PID {}", l.owning_pid),
            Some(_) => "Current Docker Compose Container".to_string(),
            None => "None".to_string(),
        };

        let suggested = if is_conflicting {
            Self::find_next_free_port(host_port)
        } else {
            host_port
        };

        DockerPortConflict {
            service_name: service_name.to_string(),
            host_port,
            container_port,
            protocol: proto.to_string(),
            host_ip: host_ip.to_string(),
            is_conflicting,
            conflict_owner,
            is_same_project_container: is_own,
            suggested_override_port: suggested,
        }
    }

    /// Search for the next free port starting from `start_port + 1`.
    #[must_use]
    pub fn find_next_free_port(start_port: u16) -> u16 {
        let listeners = get_active_tcp_listeners().unwrap_or_default();
        let occupied: HashSet<u16> = listeners.into_iter().map(|l| l.port).collect();

        for p in (start_port + 1)..=start_port.saturating_add(1000) {
            if !occupied.contains(&p) {
                return p;
            }
        }

        start_port.saturating_add(1000)
    }

    async fn run_command(
        exe: &str,
        args: &[&str],
        work_dir: &Path,
        timeout: Duration,
    ) -> Result<(bool, String)> {
        let mut child = Command::new(exe)
            .args(args)
            .current_dir(work_dir)
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
                Err(CoreError::Service("Docker command timed out".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_port_string_variations() {
        // Simple host:container
        let p1 = DockerPortChecker::parse_port_string("8080:80");
        assert_eq!(
            p1,
            vec![(8080, 80, "tcp".to_string(), "0.0.0.0".to_string())]
        );

        // IP:host:container
        let p2 = DockerPortChecker::parse_port_string("127.0.0.1:3306:3306");
        assert_eq!(
            p2,
            vec![(3306, 3306, "tcp".to_string(), "127.0.0.1".to_string())]
        );

        // Protocol suffix
        let p3 = DockerPortChecker::parse_port_string("53:53/udp");
        assert_eq!(p3, vec![(53, 53, "udp".to_string(), "0.0.0.0".to_string())]);

        // Range
        let p4 = DockerPortChecker::parse_port_string("3000-3002:3000-3002");
        assert_eq!(p4.len(), 3);
        assert_eq!(p4[0].0, 3000);
        assert_eq!(p4[1].0, 3001);
        assert_eq!(p4[2].0, 3002);
    }

    #[test]
    fn test_expand_port_range() {
        assert_eq!(DockerPortChecker::expand_port_range("80"), vec![80]);
        assert_eq!(
            DockerPortChecker::expand_port_range("9000-9003"),
            vec![9000, 9001, 9002, 9003]
        );
        assert!(DockerPortChecker::expand_port_range("invalid").is_empty());
    }

    #[test]
    fn test_parse_compose_fallback() {
        let dir = tempdir().expect("tempdir");
        let compose_file = dir.path().join("docker-compose.yml");

        let yaml = r#"
version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
      - "8443:443"
  db:
    image: postgres:16
    ports:
      - "127.0.0.1:5432:5432"
"#;
        fs::write(&compose_file, yaml).expect("write yaml");

        let conflicts =
            DockerPortChecker::parse_compose_fallback(&compose_file).expect("fallback parse");
        assert_eq!(conflicts.len(), 3);

        assert_eq!(conflicts[0].service_name, "web");
        assert_eq!(conflicts[0].host_port, 8080);
        assert_eq!(conflicts[0].container_port, 80);

        assert_eq!(conflicts[1].service_name, "web");
        assert_eq!(conflicts[1].host_port, 8443);

        assert_eq!(conflicts[2].service_name, "db");
        assert_eq!(conflicts[2].host_port, 5432);
        assert_eq!(conflicts[2].host_ip, "127.0.0.1");
    }
}
