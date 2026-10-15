//! Service profiles store (standalone vs docker orchestration profiles).

use crate::error::Result;
use crate::persistence::{AppPaths, JsonStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceProfile {
    pub id: String,
    pub name: String,
    pub service_ids: Vec<String>,
    pub optional_service_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfiguration {
    pub active_profile_id: String,
    pub profiles: Vec<ServiceProfile>,
}

pub struct ProfileStore {
    state_file: PathBuf,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileStore {
    pub fn new() -> Self {
        let state_file = AppPaths::get_path("state").join("profiles.json");
        Self { state_file }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { state_file: path }
    }

    pub fn load_config(&self) -> Result<ProfileConfiguration> {
        if let Some(config) = JsonStore::load::<ProfileConfiguration, _>(&self.state_file)? {
            if !config.profiles.is_empty() {
                return Ok(config);
            }
        }

        // Return default configuration
        let default_config = ProfileConfiguration {
            active_profile_id: "standalone".to_string(),
            profiles: vec![
                ServiceProfile {
                    id: "standalone".to_string(),
                    name: "Standalone (Local Databases & Web)".to_string(),
                    service_ids: vec![
                        "mysql".into(),
                        "postgres".into(),
                        "redis".into(),
                        "php-cgi".into(),
                        "nginx".into(),
                    ],
                    optional_service_ids: vec![],
                },
                ServiceProfile {
                    id: "docker".to_string(),
                    name: "Docker Coexistence (Nginx & PHP only)".to_string(),
                    service_ids: vec!["php-cgi".into(), "nginx".into()],
                    optional_service_ids: vec!["mysql".into(), "postgres".into(), "redis".into()],
                },
            ],
        };

        // Persist defaults
        let _ = JsonStore::save_atomic(&self.state_file, &default_config);
        Ok(default_config)
    }

    /// Retrieve the currently active profile.
    pub fn get_active_profile(&self) -> Result<ServiceProfile> {
        let config = self.load_config()?;
        let active = config
            .profiles
            .into_iter()
            .find(|p| p.id.eq_ignore_ascii_case(&config.active_profile_id))
            .unwrap_or_else(|| ServiceProfile {
                id: "standalone".into(),
                name: "Standalone".into(),
                service_ids: vec![],
                optional_service_ids: vec![],
            });

        Ok(active)
    }

    /// Set the active profile ID and persist.
    pub fn set_active_profile(&self, profile_id: &str) -> Result<bool> {
        let mut config = self.load_config()?;
        let exists = config
            .profiles
            .iter()
            .any(|p| p.id.eq_ignore_ascii_case(profile_id));

        if exists {
            config.active_profile_id = profile_id.to_string();
            JsonStore::save_atomic(&self.state_file, &config)?;
            tracing::info!("ProfileStore: Active profile set to '{}'", profile_id);
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
    fn test_profile_store_defaults_and_switch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_file = temp_dir.path().join("profiles.json");
        let store = ProfileStore::with_path(state_file);

        let active = store.get_active_profile().unwrap();
        assert_eq!(active.id, "standalone");
        assert!(active.service_ids.contains(&"mysql".to_string()));

        // Switch to docker profile
        assert!(store.set_active_profile("docker").unwrap());
        let new_active = store.get_active_profile().unwrap();
        assert_eq!(new_active.id, "docker");
        assert_eq!(new_active.service_ids, vec!["php-cgi", "nginx"]);

        // Switch to unknown profile
        assert!(!store.set_active_profile("unknown").unwrap());
    }
}
