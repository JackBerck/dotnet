//! Installed tools persistent state store (%LOCALAPPDATA%\Dotnet\state\installed-tools.json).

use crate::error::Result;
use crate::persistence::{AppPaths, JsonStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledToolRecord {
    pub tool_id: String,
    pub display_name: String,
    pub version: String,
    pub install_path: String,
    pub is_adopted: bool,
    pub is_active: bool,
    pub added_path_entries: Vec<String>,
    pub installed_at_secs: u64,
}

pub struct InstalledToolStore {
    state_file: PathBuf,
}

impl Default for InstalledToolStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InstalledToolStore {
    pub fn new() -> Self {
        let state_file = AppPaths::get_path("state").join("installed-tools.json");
        Self { state_file }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { state_file: path }
    }

    /// Load all installed tool records from persistent JSON store.
    pub fn load_all(&self) -> Result<Vec<InstalledToolRecord>> {
        let records =
            JsonStore::load::<Vec<InstalledToolRecord>, _>(&self.state_file)?.unwrap_or_default();
        Ok(records)
    }

    /// Retrieve the currently active record for a tool ID.
    pub fn get_active_tool(&self, tool_id: &str) -> Result<Option<InstalledToolRecord>> {
        let records = self.load_all()?;
        Ok(records
            .into_iter()
            .find(|t| t.tool_id.eq_ignore_ascii_case(tool_id) && t.is_active))
    }

    /// Retrieve all installed versions for a tool ID.
    pub fn get_tool_versions(&self, tool_id: &str) -> Result<Vec<InstalledToolRecord>> {
        let records = self.load_all()?;
        Ok(records
            .into_iter()
            .filter(|t| t.tool_id.eq_ignore_ascii_case(tool_id))
            .collect())
    }

    /// Register or update an installed tool record.
    pub fn register_tool(&self, record: InstalledToolRecord) -> Result<()> {
        let mut records = self.load_all()?;

        // If newly registered version is active, deactivate other versions of this tool
        if record.is_active {
            for existing in &mut records {
                if existing.tool_id.eq_ignore_ascii_case(&record.tool_id) {
                    existing.is_active = false;
                }
            }
        }

        if let Some(pos) = records.iter().position(|r| {
            r.tool_id.eq_ignore_ascii_case(&record.tool_id)
                && r.version.eq_ignore_ascii_case(&record.version)
        }) {
            records[pos] = record;
        } else {
            records.push(record);
        }

        JsonStore::save_atomic(&self.state_file, &records)?;
        Ok(())
    }

    /// Unregister a specific tool version.
    pub fn unregister_tool(&self, tool_id: &str, version: &str) -> Result<bool> {
        let mut records = self.load_all()?;
        let initial_len = records.len();

        records.retain(|r| {
            !(r.tool_id.eq_ignore_ascii_case(tool_id) && r.version.eq_ignore_ascii_case(version))
        });

        if records.len() != initial_len {
            JsonStore::save_atomic(&self.state_file, &records)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_register_and_query() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_path = temp_dir.path().join("installed-tools.json");
        let store = InstalledToolStore::with_path(state_path);

        let record_1 = InstalledToolRecord {
            tool_id: "node".into(),
            display_name: "Node.js".into(),
            version: "20.10.0".into(),
            install_path: "C:\\tools\\node\\20.10.0".into(),
            is_adopted: false,
            is_active: true,
            added_path_entries: vec!["C:\\tools\\node\\current".into()],
            installed_at_secs: 1000,
        };

        store.register_tool(record_1).unwrap();

        let active = store.get_active_tool("node").unwrap().unwrap();
        assert_eq!(active.version, "20.10.0");

        // Register newer version
        let record_2 = InstalledToolRecord {
            tool_id: "node".into(),
            display_name: "Node.js".into(),
            version: "22.0.0".into(),
            install_path: "C:\\tools\\node\\22.0.0".into(),
            is_adopted: false,
            is_active: true,
            added_path_entries: vec!["C:\\tools\\node\\current".into()],
            installed_at_secs: 2000,
        };

        store.register_tool(record_2).unwrap();

        let active_new = store.get_active_tool("node").unwrap().unwrap();
        assert_eq!(active_new.version, "22.0.0");

        // Older version should now be inactive
        let all_node = store.get_tool_versions("node").unwrap();
        assert_eq!(all_node.len(), 2);
        let old = all_node.iter().find(|r| r.version == "20.10.0").unwrap();
        assert!(!old.is_active);

        // Unregister old
        assert!(store.unregister_tool("node", "20.10.0").unwrap());
        assert_eq!(store.get_tool_versions("node").unwrap().len(), 1);
    }
}
