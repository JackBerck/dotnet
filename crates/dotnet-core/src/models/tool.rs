use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    #[serde(rename = "schemaVersion", default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub category: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(rename = "allowedHosts", default)]
    pub allowed_hosts: Vec<String>,
    pub versions: Vec<ToolVersion>,
    #[serde(default)]
    pub layout: ToolLayout,
    #[serde(rename = "postInstall", default)]
    pub post_install: Vec<PostInstallOp>,
    pub verify: Option<ToolVerify>,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersion {
    pub version: String,
    pub variant: Option<String>,
    pub source: ToolSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub sha256: String,
    #[serde(rename = "stripRoot", default)]
    pub strip_root: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolLayout {
    #[serde(rename = "binDirs", default)]
    pub bin_dirs: Vec<String>,
    #[serde(rename = "envVars", default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInstallOp {
    pub op: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub file: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVerify {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(rename = "expectRegex")]
    pub expect_regex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledToolEntry {
    pub id: String,
    pub version: String,
    pub variant: Option<String>,
    pub install_path: String,
    pub is_active: bool,
    pub is_adopted: bool,
}
