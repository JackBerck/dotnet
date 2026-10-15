use crate::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::Path;

pub struct JsonStore;

impl JsonStore {
    /// Load and deserialize JSON file. Returns None if file does not exist.
    pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<Option<T>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let data: T = serde_json::from_str(&content)?;
        Ok(Some(data))
    }

    /// Atomically save data to JSON file via a temporary file and rename.
    pub fn save_atomic<T: Serialize, P: AsRef<Path>>(path: P, data: &T) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("tmp");
        let json_str = serde_json::to_string_pretty(data)?;

        fs::write(&tmp_path, json_str)?;
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}
