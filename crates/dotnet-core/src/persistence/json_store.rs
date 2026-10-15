//! Robust, atomic JSON storage with envelope compatibility and UTF-8 BOM handling.

use crate::error::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct JsonEnvelope<T> {
    #[serde(rename = "schemaVersion", alias = "SchemaVersion", default)]
    pub schema_version: u32,
    #[serde(rename = "data", alias = "Data")]
    pub data: T,
}

pub struct JsonStore;

impl JsonStore {
    /// Load and deserialize JSON file.
    /// Handles UTF-8 BOM (written by .NET), raw T, and enveloped `{ "schemaVersion": 1, "data": T }`.
    pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<Option<T>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        // Strip UTF-8 BOM if present (.NET default)
        let trimmed = content.trim_start_matches('\u{feff}').trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        // 1. Try deserializing raw T first
        if let Ok(direct) = serde_json::from_str::<T>(trimmed) {
            return Ok(Some(direct));
        }

        // 2. Try deserializing from envelope { schemaVersion, data }
        if let Ok(envelope) = serde_json::from_str::<JsonEnvelope<T>>(trimmed) {
            return Ok(Some(envelope.data));
        }

        // 3. If neither worked, emit the actual error
        let parsed: T = serde_json::from_str(trimmed)?;
        Ok(Some(parsed))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct SampleData {
        name: String,
        count: u32,
    }

    #[test]
    fn test_load_direct_enveloped_and_bom() {
        let temp_dir = tempfile::tempdir().unwrap();

        // 1. Direct raw format
        let raw_file = temp_dir.path().join("raw.json");
        let sample = SampleData {
            name: "test".into(),
            count: 42,
        };
        JsonStore::save_atomic(&raw_file, &sample).unwrap();
        let loaded_raw: SampleData = JsonStore::load(&raw_file).unwrap().unwrap();
        assert_eq!(loaded_raw, sample);

        // 2. C# envelope format with UTF-8 BOM
        let env_file = temp_dir.path().join("envelope.json");
        let env_json = "\u{feff}{ \"SchemaVersion\": 1, \"Data\": { \"name\": \"enveloped\", \"count\": 99 } }";
        fs::write(&env_file, env_json).unwrap();
        let loaded_env: SampleData = JsonStore::load(&env_file).unwrap().unwrap();
        assert_eq!(
            loaded_env,
            SampleData {
                name: "enveloped".into(),
                count: 99
            }
        );

        // 3. Empty file returns None
        let empty_file = temp_dir.path().join("empty.json");
        fs::write(&empty_file, "   ").unwrap();
        let loaded_empty: Option<SampleData> = JsonStore::load(&empty_file).unwrap();
        assert!(loaded_empty.is_none());
    }
}
