//! Process Tracker: surveillance, startup re-adoption, and persistence to running-processes.json.

use crate::error::Result;
use crate::persistence::{AppPaths, JsonStore};
use dotnet_windows::is_pid_alive;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessRecord {
    #[serde(alias = "ServiceId")]
    pub service_id: String,
    #[serde(alias = "Pid")]
    pub pid: u32,
    #[serde(alias = "StartTime", default)]
    pub start_time_secs: u64,
    #[serde(alias = "ExecutablePath", default)]
    pub executable_path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessTrackerState {
    #[serde(alias = "Records", default)]
    pub records: Vec<ProcessRecord>,
}

pub struct ProcessTracker {
    state_file: PathBuf,
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTracker {
    pub fn new() -> Self {
        let state_file = AppPaths::get_path("state").join("running-processes.json");
        Self { state_file }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { state_file: path }
    }

    fn load_state(&self) -> Result<ProcessTrackerState> {
        let state =
            JsonStore::load::<ProcessTrackerState, _>(&self.state_file)?.unwrap_or_default();
        Ok(state)
    }

    fn save_state(&self, state: &ProcessTrackerState) -> Result<()> {
        JsonStore::save_atomic(&self.state_file, state)
    }

    /// Record a running service process.
    pub fn record_process<P: AsRef<Path>>(
        &self,
        service_id: &str,
        pid: u32,
        executable_path: P,
    ) -> Result<()> {
        let mut state = self.load_state()?;
        state
            .records
            .retain(|r| !r.service_id.eq_ignore_ascii_case(service_id));

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        state.records.push(ProcessRecord {
            service_id: service_id.to_string(),
            pid,
            start_time_secs: now,
            executable_path: executable_path.as_ref().to_string_lossy().to_string(),
        });

        self.save_state(&state)?;
        tracing::info!("ProcessTracker: Recorded PID {} for '{}'", pid, service_id);
        Ok(())
    }

    /// Remove a service process from tracking.
    pub fn remove_process(&self, service_id: &str) -> Result<()> {
        let mut state = self.load_state()?;
        let initial_len = state.records.len();
        state
            .records
            .retain(|r| !r.service_id.eq_ignore_ascii_case(service_id));

        if state.records.len() != initial_len {
            self.save_state(&state)?;
            tracing::info!("ProcessTracker: Removed '{}' from tracking", service_id);
        }
        Ok(())
    }

    /// Query all currently tracked processes.
    pub fn get_tracked_processes(&self) -> Result<Vec<ProcessRecord>> {
        let state = self.load_state()?;
        Ok(state.records)
    }

    /// Retrieve a single tracked process record by service ID.
    pub fn get_process(&self, service_id: &str) -> Result<Option<ProcessRecord>> {
        let state = self.load_state()?;
        Ok(state
            .records
            .into_iter()
            .find(|r| r.service_id.eq_ignore_ascii_case(service_id)))
    }

    /// Attempt to re-adopt a previously running process on application startup.
    /// Discards stale records if PID is no longer alive or reused.
    pub fn try_adopt_process(
        &self,
        service_id: &str,
        expected_executable_path: Option<&str>,
    ) -> Result<Option<ProcessRecord>> {
        let mut state = self.load_state()?;
        let record_opt = state
            .records
            .iter()
            .find(|r| r.service_id.eq_ignore_ascii_case(service_id))
            .cloned();

        let record = match record_opt {
            Some(r) => r,
            None => return Ok(None),
        };

        if !is_pid_alive(record.pid) {
            // Process exited while application was offline
            tracing::info!(
                "ProcessTracker: PID {} for '{}' is no longer alive. Discarding.",
                record.pid,
                service_id
            );
            state
                .records
                .retain(|r| !r.service_id.eq_ignore_ascii_case(service_id));
            self.save_state(&state)?;
            return Ok(None);
        }

        // Validate executable match if expected path provided
        if let Some(expected) = expected_executable_path {
            let expected_norm = expected.trim().trim_end_matches('\\').to_lowercase();
            let rec_norm = record
                .executable_path
                .trim()
                .trim_end_matches('\\')
                .to_lowercase();

            if expected_norm != rec_norm {
                tracing::warn!(
                    "ProcessTracker: Executable mismatch for '{}'. Expected: '{}', Recorded: '{}'. Discarding.",
                    service_id, expected, record.executable_path
                );
                state
                    .records
                    .retain(|r| !r.service_id.eq_ignore_ascii_case(service_id));
                self.save_state(&state)?;
                return Ok(None);
            }
        }

        tracing::info!(
            "ProcessTracker: Successfully adopted existing process for '{}' (PID {})",
            service_id,
            record.pid
        );

        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_tracker_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_file = temp_dir.path().join("running-processes.json");
        let tracker = ProcessTracker::with_path(state_file);

        let current_pid = std::process::id();
        tracker
            .record_process("test-daemon", current_pid, "C:\\test\\daemon.exe")
            .unwrap();

        let tracked = tracker.get_tracked_processes().unwrap();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].service_id, "test-daemon");
        assert_eq!(tracked[0].pid, current_pid);

        // Try adopt current running process
        let adopted = tracker
            .try_adopt_process("test-daemon", Some("C:\\test\\daemon.exe"))
            .unwrap();
        assert!(adopted.is_some());
        assert_eq!(adopted.unwrap().pid, current_pid);

        // Try adopt nonexistent service
        assert!(tracker
            .try_adopt_process("unknown-svc", None)
            .unwrap()
            .is_none());

        // Remove process
        tracker.remove_process("test-daemon").unwrap();
        assert_eq!(tracker.get_tracked_processes().unwrap().len(), 0);
    }
}
