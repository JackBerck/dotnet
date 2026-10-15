use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub framework: String,
    pub local_domain: Option<String>,
    pub port: Option<u16>,
    pub ssl_enabled: bool,
    pub dev_command: Option<String>,
}
