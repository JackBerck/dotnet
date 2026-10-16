//! Embedded Lightweight HTTP Dashboard Server for Dotnet Desktop UI.

use crate::service_manager::AppServiceManager;
use crate::ui::DASHBOARD_HTML;
use anyhow::{anyhow, Context, Result};
use dotnet_core::catalog::CatalogLoader;
use dotnet_core::installation::store::InstalledToolStore;
use dotnet_core::installation::ToolInstaller;
use dotnet_core::models::ProjectInfo;
use dotnet_core::persistence::AppPaths;
use dotnet_core::services::framework_detector::FrameworkDetector;
use dotnet_core::services::hosts_manager::HostsFileManager;
use dotnet_core::services::nginx_manager::NginxManager;
use dotnet_core::services::path_shadow_analyzer::PathShadowAnalyzer;
use dotnet_core::services::php_config_manager::PhpConfigManager;
use dotnet_core::services::profile_store::ProfileStore;
use dotnet_core::services::project_manager::ProjectManager;
use dotnet_windows::get_active_tcp_listeners;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Shared state across dashboard API handlers.
#[derive(Clone)]
pub struct DashboardState {
    pub service_manager: Arc<AppServiceManager>,
    pub project_manager: Arc<Mutex<ProjectManager>>,
    pub profile_store: Arc<Mutex<ProfileStore>>,
    pub tool_store: Arc<Mutex<InstalledToolStore>>,
    pub php_config_manager: Arc<Mutex<PhpConfigManager>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub port: u16,
    pub event_loop_proxy: Option<tao::event_loop::EventLoopProxy<dotnet_windows::tray::TrayEvent>>,
}

impl DashboardState {
    pub async fn add_log(&self, msg: impl Into<String>) {
        let mut logs = self.logs.lock().await;
        let line = format!("[{}] {}", chrono_lite_timestamp(), msg.into());
        logs.push(line);
        if logs.len() > 500 {
            logs.remove(0);
        }
    }
}

fn chrono_lite_timestamp() -> String {
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

/// Lightweight HTTP request struct.
#[derive(Debug, Default)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP response struct.
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok_html(html: &str) -> Self {
        Self {
            status_code: 200,
            status_text: "OK",
            content_type: "text/html; charset=utf-8",
            body: html.as_bytes().to_vec(),
        }
    }

    pub fn ok_json<T: Serialize>(data: &T) -> Self {
        match serde_json::to_vec(data) {
            Ok(body) => Self {
                status_code: 200,
                status_text: "OK",
                content_type: "application/json",
                body,
            },
            Err(e) => Self::error(500, &format!("JSON serialize error: {e}")),
        }
    }

    pub fn ok_msg(msg: &str) -> Self {
        let json = serde_json::json!({ "status": "ok", "message": msg });
        Self::ok_json(&json)
    }

    pub fn error(code: u16, msg: &str) -> Self {
        let json = serde_json::json!({ "status": "error", "message": msg });
        let body = serde_json::to_vec(&json).unwrap_or_else(|_| b"{}".to_vec());
        Self {
            status_code: code,
            status_text: match code {
                400 => "Bad Request",
                404 => "Not Found",
                _ => "Internal Server Error",
            },
            content_type: "application/json",
            body,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let header = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             Connection: close\r\n\r\n",
            self.status_code,
            self.status_text,
            self.content_type,
            self.body.len()
        );
        let mut res = header.into_bytes();
        res.extend_from_slice(&self.body);
        res
    }
}

#[derive(Serialize)]
pub struct ToolDashboardItem {
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub latest_version: String,
    pub is_installed: bool,
    pub installed_version: String,
}

#[derive(Deserialize)]
pub struct AddProjectPayload {
    pub path: String,
}

/// Background HTTP Dashboard Server.
pub struct DashboardServer {
    state: DashboardState,
}

impl DashboardServer {
    pub fn new(state: DashboardState) -> Self {
        Self { state }
    }

    /// Run the server loop indefinitely.
    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.state.port));
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind dashboard server to {addr}"))?;

        info!(
            "Embedded HTTP dashboard running on http://127.0.0.1:{}",
            self.state.port
        );
        self.state
            .add_log(format!(
                "Embedded HTTP dashboard running on http://127.0.0.1:{}",
                self.state.port
            ))
            .await;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, state).await {
                            warn!("Dashboard connection error: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("Dashboard accept error: {e}");
                }
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, state: DashboardState) -> Result<()> {
    let mut buffer = [0u8; 8192];
    let bytes_read = stream.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = parse_http_request(&buffer[..bytes_read])?;

    // Handle CORS preflight
    let response = if request.method == "OPTIONS" {
        HttpResponse {
            status_code: 204,
            status_text: "No Content",
            content_type: "text/plain",
            body: Vec::new(),
        }
    } else {
        route_request(&request, &state).await
    };

    stream.write_all(&response.to_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

fn parse_http_request(bytes: &[u8]) -> Result<HttpRequest> {
    let text = std::str::from_utf8(bytes).map_err(|e| anyhow!("Invalid UTF-8 request: {e}"))?;
    let mut lines = text.lines();

    let request_line = lines.next().ok_or_else(|| anyhow!("Empty request"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let full_path = parts.next().unwrap_or("/").to_string();

    let (path, query) = parse_path_and_query(&full_path);

    let mut headers = HashMap::new();
    let mut content_length = 0usize;

    for line in lines.by_ref() {
        if line.is_empty() || line == "\r" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            if key == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers.insert(key, val);
        }
    }

    let body = if content_length > 0 {
        if let Some(pos) = text.find("\r\n\r\n") {
            let start = pos + 4;
            bytes
                .get(start..start + content_length)
                .unwrap_or_default()
                .to_vec()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(HttpRequest {
        method,
        path,
        query,
        headers,
        body,
    })
}

fn parse_path_and_query(full_path: &str) -> (String, HashMap<String, String>) {
    let mut query_map = HashMap::new();
    if let Some((p, q)) = full_path.split_once('?') {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                query_map.insert(k.to_string(), v.to_string());
            } else if !pair.is_empty() {
                query_map.insert(pair.to_string(), String::new());
            }
        }
        (p.to_string(), query_map)
    } else {
        (full_path.to_string(), query_map)
    }
}

async fn route_request(req: &HttpRequest, state: &DashboardState) -> HttpResponse {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => HttpResponse::ok_html(DASHBOARD_HTML),

        ("GET", "/api/status") => {
            let p_store = state.profile_store.lock().await;
            let active_profile = p_store
                .get_active_profile()
                .map(|p| p.name)
                .unwrap_or_else(|_| "Standalone".to_string());

            let statuses = state.service_manager.get_all_statuses().await;
            let running = statuses.iter().filter(|s| s.is_running).count();

            let res = serde_json::json!({
                "status": "online",
                "profile": active_profile,
                "services_running": running,
                "services_total": statuses.len(),
            });
            HttpResponse::ok_json(&res)
        }

        ("GET", "/api/services") => {
            let statuses = state.service_manager.get_all_statuses().await;
            HttpResponse::ok_json(&statuses)
        }

        ("POST", "/api/services/action") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            let action = req.query.get("action").map(|s| s.as_str()).unwrap_or("");

            match action {
                "start" => match state.service_manager.start_service(id).await {
                    Ok(_) => {
                        state.add_log(format!("[ACTION] Service '{id}' start succeeded.")).await;
                        HttpResponse::ok_msg(&format!("Service '{id}' started"))
                    }
                    Err(e) => {
                        state.add_log(format!("[ACTION] Failed to start service '{id}': {e}")).await;
                        HttpResponse::error(500, &format!("Failed to start '{id}': {e}"))
                    }
                },
                "stop" => match state.service_manager.stop_service(id).await {
                    Ok(_) => {
                        state.add_log(format!("[ACTION] Service '{id}' stop succeeded.")).await;
                        HttpResponse::ok_msg(&format!("Service '{id}' stopped"))
                    }
                    Err(e) => {
                        state.add_log(format!("[ACTION] Failed to stop service '{id}': {e}")).await;
                        HttpResponse::error(500, &format!("Failed to stop '{id}': {e}"))
                    }
                },
                "restart" => match state.service_manager.restart_service(id).await {
                    Ok(_) => {
                        state.add_log(format!("[ACTION] Service '{id}' restart succeeded.")).await;
                        HttpResponse::ok_msg(&format!("Service '{id}' restarted"))
                    }
                    Err(e) => {
                        state.add_log(format!("[ACTION] Failed to restart service '{id}': {e}")).await;
                        HttpResponse::error(500, &format!("Failed to restart '{id}': {e}"))
                    }
                },
                _ => HttpResponse::error(400, "Invalid service action"),
            }
        }

        ("POST", "/api/services/start-all") => match state.service_manager.start_all().await {
            Ok(_) => {
                state.add_log("[BATCH] All services started successfully.").await;
                HttpResponse::ok_msg("All services started successfully")
            }
            Err(e) => {
                state.add_log(format!("[BATCH] Failed to start all services: {e}")).await;
                HttpResponse::error(500, &format!("Error starting all services: {e}"))
            }
        },

        ("POST", "/api/services/stop-all") => match state.service_manager.stop_all().await {
            Ok(_) => {
                state.add_log("[BATCH] All services stopped.").await;
                HttpResponse::ok_msg("All services stopped successfully")
            }
            Err(e) => {
                state.add_log(format!("[BATCH] Failed to stop all services: {e}")).await;
                HttpResponse::error(500, &format!("Error stopping all services: {e}"))
            }
        },

        ("POST", "/api/nginx/reload") => {
            let nginx_dir = AppPaths::tools_dir().join("nginx").join("current");
            match NginxManager::reload_nginx(&nginx_dir).await {
                Ok((true, output)) => {
                    state.add_log(format!("Nginx reloaded: {output}")).await;
                    HttpResponse::ok_msg(&format!("Nginx reloaded: {output}"))
                }
                Ok((false, err)) => {
                    HttpResponse::error(500, &format!("Nginx reload failed: {err}"))
                }
                Err(e) => HttpResponse::error(500, &format!("Nginx reload error: {e}")),
            }
        }

        ("POST", "/api/profile/switch") => {
            let id = req
                .query
                .get("id")
                .map(|s| s.as_str())
                .unwrap_or("standalone");
            let p_store = state.profile_store.lock().await;

            match p_store.set_active_profile(id) {
                Ok(_) => {
                    state
                        .add_log(format!("Switched active profile to '{id}'."))
                        .await;
                    HttpResponse::ok_msg(&format!("Profile switched to '{id}'"))
                }
                Err(e) => HttpResponse::error(500, &format!("Failed to switch profile: {e}")),
            }
        }

        ("GET", "/api/projects") => {
            let pm = state.project_manager.lock().await;
            match pm.get_projects() {
                Ok(projects) => HttpResponse::ok_json(&projects),
                Err(e) => HttpResponse::error(500, &format!("Error reading projects: {e}")),
            }
        }

        ("POST", "/api/projects/add") => {
            match serde_json::from_slice::<AddProjectPayload>(&req.body) {
                Ok(payload) => {
                    let path = Path::new(&payload.path);
                    if !path.is_dir() {
                        return HttpResponse::error(400, "Provided path is not a valid directory");
                    }

                    let detected = FrameworkDetector::detect(path);

                    let project = ProjectInfo {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: detected.suggested_name,
                        path: path.to_string_lossy().to_string(),
                        framework: detected.framework,
                        dev_command: if detected.dev_command.is_empty() {
                            None
                        } else {
                            Some(detected.dev_command)
                        },
                        public_directory: detected.public_directory,
                        nginx_host: Some(detected.suggested_host),
                        ssl_enabled: true,
                        dev_port: detected.suggested_port,
                        php_version: Some("8.3".to_string()),
                        node_version: None,
                        database_type: Some("MySQL".to_string()),
                        database_name: None,
                        requires_redis: false,
                        nginx_site_mode: Some("auto".to_string()),
                        notes: None,
                    };

                    let pm = state.project_manager.lock().await;
                    match pm.add_or_update_project(project.clone()) {
                        Ok(_) => {
                            state
                                .add_log(format!(
                                    "Project '{}' registered at '{}'.",
                                    project.name, project.path
                                ))
                                .await;
                            HttpResponse::ok_json(&project)
                        }
                        Err(e) => {
                            HttpResponse::error(500, &format!("Failed to persist project: {e}"))
                        }
                    }
                }
                Err(e) => HttpResponse::error(400, &format!("Invalid JSON payload: {e}")),
            }
        }

        ("POST", "/api/projects/delete") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            let pm = state.project_manager.lock().await;
            match pm.delete_project(id) {
                Ok(deleted) => {
                    if deleted {
                        state.add_log(format!("Project '{id}' deleted.")).await;
                        HttpResponse::ok_msg("Project deleted")
                    } else {
                        HttpResponse::error(404, "Project not found")
                    }
                }
                Err(e) => HttpResponse::error(500, &format!("Error deleting project: {e}")),
            }
        }

        ("POST", "/api/projects/launch") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            let action = req.query.get("action").map(|s| s.as_str()).unwrap_or("");
            let pm = state.project_manager.lock().await;

            let proj = match pm.get_project_by_id(id) {
                Ok(Some(p)) => p,
                Ok(None) => return HttpResponse::error(404, "Project not found"),
                Err(e) => return HttpResponse::error(500, &format!("Project lookup error: {e}")),
            };

            let proj_path = PathBuf::from(&proj.path);
            let result = match action {
                "run" => ProjectManager::launch_dev_command(&proj),
                "explorer" => ProjectManager::open_in_explorer(&proj_path),
                "terminal" => ProjectManager::open_terminal(&proj_path),
                "code" => ProjectManager::open_in_vscode(&proj_path),
                _ => Err(dotnet_core::CoreError::Config(format!(
                    "Unknown launch action '{action}'"
                ))),
            };

            match result {
                Ok(_) => {
                    state
                        .add_log(format!("Launched '{action}' for project '{}'.", proj.name))
                        .await;
                    HttpResponse::ok_msg(&format!("Launched '{action}'"))
                }
                Err(e) => HttpResponse::error(500, &format!("Failed to launch '{action}': {e}")),
            }
        }

        ("GET", "/api/tools") => {
            let tool_store = state.tool_store.lock().await;
            let installed: Vec<dotnet_core::installation::InstalledToolRecord> =
                tool_store.load_all().unwrap_or_default();
            let manifests = CatalogLoader::load_embedded().unwrap_or_default();

            let mut items: Vec<ToolDashboardItem> = Vec::new();
            for m in manifests {
                let inst = installed
                    .iter()
                    .find(|i| i.tool_id.eq_ignore_ascii_case(&m.id));
                items.push(ToolDashboardItem {
                    id: m.id.clone(),
                    display_name: m.display_name.clone(),
                    category: m.category.clone(),
                    latest_version: m
                        .versions
                        .first()
                        .map(|v| v.version.clone())
                        .unwrap_or_default(),
                    is_installed: inst.is_some(),
                    installed_version: inst.map(|i| i.version.clone()).unwrap_or_default(),
                });
            }
            for inst in installed {
                if !items.iter().any(|i| i.id.eq_ignore_ascii_case(&inst.tool_id)) {
                    items.push(ToolDashboardItem {
                        id: inst.tool_id.clone(),
                        display_name: inst.display_name.clone(),
                        category: "system".into(),
                        latest_version: inst.version.clone(),
                        is_installed: true,
                        installed_version: inst.version.clone(),
                    });
                }
            }

            HttpResponse::ok_json(&items)
        }

        ("POST", "/api/tools/scan") => {
            let tool_store = state.tool_store.lock().await;
            match dotnet_core::installation::AdoptExistingScanner::adopt_all_discovered(&tool_store) {
                Ok(count) => {
                    state
                        .add_log(format!("Adopted {count} local tools from system."))
                        .await;
                    HttpResponse::ok_msg(&format!("Adopted {count} tools"))
                }
                Err(e) => HttpResponse::error(500, &format!("Scan error: {e}")),
            }
        }

        ("POST", "/api/tools/install") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            let manifest_opt = match CatalogLoader::find_embedded(id) {
                Ok(m) => m,
                Err(e) => return HttpResponse::error(500, &format!("Catalog error: {e}")),
            };

            let manifest = match manifest_opt {
                Some(m) => m,
                None => return HttpResponse::error(404, "Tool not found in catalog"),
            };

            let version = match manifest.versions.first() {
                Some(v) => v.clone(),
                None => return HttpResponse::error(400, "No versions available in manifest"),
            };

            let tool_store = state.tool_store.lock().await;
            match ToolInstaller::install(&manifest, &version, &tool_store, |_| {}).await {
                Ok(_) => {
                    state
                        .add_log(format!("Tool '{id}' installed successfully."))
                        .await;
                    HttpResponse::ok_msg(&format!("Tool '{id}' installed"))
                }
                Err(e) => HttpResponse::error(500, &format!("Tool install error: {e}")),
            }
        }

        ("POST", "/api/tools/uninstall") => {
            let id = req.query.get("id").map(|s| s.as_str()).unwrap_or("");
            let tool_store = state.tool_store.lock().await;
            let record_opt = match tool_store.get_active_tool(id) {
                Ok(r) => r,
                Err(e) => return HttpResponse::error(500, &format!("Tool query error: {e}")),
            };

            let record = match record_opt {
                Some(r) => r,
                None => return HttpResponse::error(404, "Installed tool not found"),
            };

            match ToolInstaller::uninstall(&record, &tool_store) {
                Ok(_) => {
                    state.add_log(format!("Tool '{id}' uninstalled.")).await;
                    HttpResponse::ok_msg(&format!("Tool '{id}' uninstalled"))
                }
                Err(e) => HttpResponse::error(500, &format!("Tool uninstall error: {e}")),
            }
        }

        ("GET", "/api/hosts") => match HostsFileManager::read_entries_from_default() {
            Ok(entries) => HttpResponse::ok_json(&entries),
            Err(e) => HttpResponse::error(500, &format!("Hosts read error: {e}")),
        },

        ("GET", "/api/php/extensions") => {
            let pcm = state.php_config_manager.lock().await;
            match pcm.get_available_extensions() {
                Ok(exts) => HttpResponse::ok_json(&exts),
                Err(e) => HttpResponse::error(500, &format!("PHP extensions read error: {e}")),
            }
        }

        ("POST", "/api/php/toggle-extension") => {
            let name = req.query.get("name").map(|s| s.as_str()).unwrap_or("");
            let enable = req
                .query
                .get("enable")
                .map(|s| s == "true")
                .unwrap_or(false);
            let pcm = state.php_config_manager.lock().await;

            match pcm.toggle_extension(name, enable) {
                Ok(_) => {
                    state
                        .add_log(format!(
                            "PHP extension '{name}' set to {}.",
                            if enable { "enabled" } else { "disabled" }
                        ))
                        .await;
                    HttpResponse::ok_msg(&format!("Extension '{name}' updated"))
                }
                Err(e) => HttpResponse::error(500, &format!("Extension toggle error: {e}")),
            }
        }

        ("POST", "/api/php/preset") => {
            let name = req
                .query
                .get("name")
                .map(|s| s.as_str())
                .unwrap_or("development");
            let pcm = state.php_config_manager.lock().await;

            match pcm.apply_preset(name) {
                Ok(msg) => {
                    state
                        .add_log(format!("PHP preset '{name}' applied: {msg}."))
                        .await;
                    HttpResponse::ok_msg(&format!("PHP preset '{name}' applied"))
                }
                Err(e) => HttpResponse::error(500, &format!("PHP preset error: {e}")),
            }
        }

        ("GET", "/api/ports") => match get_active_tcp_listeners() {
            Ok(table) => HttpResponse::ok_json(&table),
            Err(e) => HttpResponse::error(500, &format!("TCP table error: {e}")),
        },

        ("GET", "/api/shadow") => match PathShadowAnalyzer::analyze(None, None) {
            Ok(results) => HttpResponse::ok_json(&results),
            Err(e) => HttpResponse::error(500, &format!("Shadow analysis error: {e}")),
        },

        ("GET", "/api/logs") => {
            let logs = state.logs.lock().await;
            HttpResponse::ok_json(&*logs)
        }

        ("POST", "/api/logs/clear") => {
            let mut logs = state.logs.lock().await;
            logs.clear();
            HttpResponse::ok_msg("Logs cleared")
        }

        ("POST", "/api/app/restore") => {
            if let Some(ref proxy) = state.event_loop_proxy {
                let _ = proxy.send_event(dotnet_windows::tray::TrayEvent::OpenDashboard);
            }
            state.add_log("[APP] Duplicate instance launched - restored main window to foreground.").await;
            HttpResponse::ok_msg("Application window restored")
        }

        _ => HttpResponse::error(404, "Endpoint not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_request_get() {
        let raw = b"GET /api/status HTTP/1.1\r\nHost: localhost:4040\r\n\r\n";
        let req = parse_http_request(raw).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/status");
        assert!(req.query.is_empty());
    }

    #[test]
    fn test_parse_http_request_query_params() {
        let raw = b"POST /api/services/action?id=nginx&action=start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n";
        let req = parse_http_request(raw).unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/services/action");
        assert_eq!(req.query.get("id").map(|s| s.as_str()), Some("nginx"));
        assert_eq!(req.query.get("action").map(|s| s.as_str()), Some("start"));
    }

    #[test]
    fn test_http_response_bytes() {
        let res = HttpResponse::ok_msg("hello world");
        let bytes = res.to_bytes();
        let str_res = String::from_utf8(bytes).unwrap();
        assert!(str_res.starts_with("HTTP/1.1 200 OK"));
        assert!(str_res.contains("application/json"));
        assert!(str_res.contains("hello world"));
    }

    #[test]
    fn test_parse_http_post_with_body() {
        let raw = b"POST /api/projects/add HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"path\":\"C:\\\\app\"}";
        let req = parse_http_request(raw).unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/projects/add");
        assert_eq!(req.body, b"{\"path\":\"C:\\\\app\"}");
    }

    #[tokio::test]
    async fn test_route_get_dashboard_html() {
        let state = create_test_dashboard_state();
        let req = HttpRequest {
            method: "GET".to_string(),
            path: "/".to_string(),
            ..Default::default()
        };
        let res = route_request(&req, &state).await;
        assert_eq!(res.status_code, 200);
        assert_eq!(res.content_type, "text/html; charset=utf-8");
        assert!(res.body.len() > 1000);
    }

    #[tokio::test]
    async fn test_route_get_status_json() {
        let state = create_test_dashboard_state();
        let req = HttpRequest {
            method: "GET".to_string(),
            path: "/api/status".to_string(),
            ..Default::default()
        };
        let res = route_request(&req, &state).await;
        assert_eq!(res.status_code, 200);
        assert_eq!(res.content_type, "application/json");

        let body_str = String::from_utf8(res.body).unwrap();
        assert!(body_str.contains("\"status\":\"online\""));
        assert!(body_str.contains("\"services_total\":6"));
    }

    #[tokio::test]
    async fn test_route_get_services() {
        let state = create_test_dashboard_state();
        let req = HttpRequest {
            method: "GET".to_string(),
            path: "/api/services".to_string(),
            ..Default::default()
        };
        let res = route_request(&req, &state).await;
        assert_eq!(res.status_code, 200);

        let body_str = String::from_utf8(res.body).unwrap();
        assert!(body_str.contains("Nginx Web Server"));
        assert!(body_str.contains("PHP FastCGI Engine"));
    }

    #[tokio::test]
    async fn test_route_not_found() {
        let state = create_test_dashboard_state();
        let req = HttpRequest {
            method: "GET".to_string(),
            path: "/api/nonexistent_endpoint".to_string(),
            ..Default::default()
        };
        let res = route_request(&req, &state).await;
        assert_eq!(res.status_code, 404);
    }

    fn create_test_dashboard_state() -> DashboardState {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path().to_path_buf();

        let process_tracker = Arc::new(Mutex::new(
            dotnet_core::services::process_tracker::ProcessTracker::with_path(
                base.join("running-processes.json"),
            ),
        ));
        let profile_store = Arc::new(Mutex::new(
            dotnet_core::services::profile_store::ProfileStore::with_path(
                base.join("profiles.json"),
            ),
        ));
        let service_manager = Arc::new(AppServiceManager::new(
            process_tracker,
            profile_store.clone(),
            None,
        ));
        let project_manager = Arc::new(Mutex::new(
            dotnet_core::services::project_manager::ProjectManager::with_paths(
                base.join("projects.json"),
                base.join("legacy.json"),
            ),
        ));
        let tool_store = Arc::new(Mutex::new(
            dotnet_core::installation::store::InstalledToolStore::with_path(
                base.join("installed-tools.json"),
            ),
        ));
        let php_config_manager = Arc::new(Mutex::new(
            dotnet_core::services::php_config_manager::PhpConfigManager::with_paths(
                base.join("php"),
                base.join("php").join("php.ini"),
            ),
        ));

        DashboardState {
            service_manager,
            project_manager,
            profile_store,
            tool_store,
            php_config_manager,
            logs: Arc::new(Mutex::new(Vec::new())),
            port: 4040,
            event_loop_proxy: None,
        }
    }
}
