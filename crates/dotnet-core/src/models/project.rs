use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn default_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_framework() -> String {
    "custom".to_string()
}

fn default_public_dir() -> String {
    "public".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectInfo {
    #[serde(default = "default_id", alias = "id")]
    pub id: String,

    #[serde(default, alias = "name")]
    pub name: String,

    #[serde(default, alias = "path")]
    pub path: String,

    #[serde(default = "default_framework", alias = "framework")]
    pub framework: String, // laravel, nextjs, vite, node, php, static, custom

    #[serde(default, alias = "php_version", alias = "PhpVersion")]
    pub php_version: Option<String>,

    #[serde(default, alias = "node_version", alias = "NodeVersion")]
    pub node_version: Option<String>,

    #[serde(default, alias = "database_type", alias = "DatabaseType")]
    pub database_type: Option<String>,

    #[serde(default, alias = "database_name", alias = "DatabaseName")]
    pub database_name: Option<String>,

    #[serde(default, alias = "requires_redis", alias = "RequiresRedis")]
    pub requires_redis: bool,

    #[serde(default, alias = "dev_command", alias = "DevCommand")]
    pub dev_command: Option<String>,

    #[serde(
        default,
        alias = "dev_port",
        alias = "DevPort",
        alias = "port",
        alias = "Port"
    )]
    pub dev_port: Option<u16>,

    #[serde(
        default,
        alias = "nginx_host",
        alias = "NginxHost",
        alias = "local_domain",
        alias = "LocalDomain"
    )]
    pub nginx_host: Option<String>,

    #[serde(default, alias = "nginx_site_mode", alias = "NginxSiteMode")]
    pub nginx_site_mode: Option<String>, // auto, manual, disabled

    #[serde(
        default = "default_public_dir",
        alias = "public_directory",
        alias = "PublicDirectory",
        alias = "public_dir"
    )]
    pub public_directory: String,

    #[serde(default, alias = "ssl_enabled", alias = "SslEnabled")]
    pub ssl_enabled: bool,

    #[serde(default, alias = "notes", alias = "Notes")]
    pub notes: Option<String>,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            id: default_id(),
            name: String::new(),
            path: String::new(),
            framework: default_framework(),
            php_version: Some("8.5".to_string()),
            node_version: None,
            database_type: Some("MySQL".to_string()),
            database_name: None,
            requires_redis: false,
            dev_command: None,
            dev_port: None,
            nginx_host: None,
            nginx_site_mode: Some("auto".to_string()),
            public_directory: default_public_dir(),
            ssl_enabled: false,
            notes: None,
        }
    }
}

impl ProjectInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn effective_host(&self) -> &str {
        match &self.nginx_host {
            Some(h) if !h.trim().is_empty() => h.trim(),
            _ => "localhost",
        }
    }

    #[must_use]
    pub fn resolved_public_path(&self) -> PathBuf {
        let base = Path::new(&self.path);
        let pub_dir = self.public_directory.trim();
        if pub_dir.is_empty() || pub_dir == "." {
            base.to_path_buf()
        } else {
            base.join(pub_dir)
        }
    }

    #[must_use]
    pub fn is_node_based(&self) -> bool {
        matches!(
            self.framework.to_lowercase().as_str(),
            "nextjs" | "vite" | "node"
        )
    }

    #[must_use]
    pub fn is_php_based(&self) -> bool {
        matches!(self.framework.to_lowercase().as_str(), "laravel" | "php")
    }

    #[must_use]
    pub fn is_static(&self) -> bool {
        self.framework.eq_ignore_ascii_case("static")
    }
}
