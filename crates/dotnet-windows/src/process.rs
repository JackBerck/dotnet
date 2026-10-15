//! Native Windows Process inspection and lifecycle controls.

use crate::error::{Result, WindowsError};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_TERMINATE,
};

const STILL_ACTIVE: u32 = 259;

/// Check if a given process ID is currently alive on the system.
pub fn is_pid_alive(pid: u32) -> bool {
    if pid == 0 || pid == 4 {
        return true; // System / Idle
    }

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }

        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);

        ok != 0 && exit_code == STILL_ACTIVE
    }
}

/// Terminate process by PID.
pub fn terminate_pid(pid: u32) -> Result<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            let err = std::io::Error::last_os_error();
            return Err(WindowsError::ProcessError(format!(
                "Failed to open process {pid} for termination: {err}"
            )));
        }

        let ok = TerminateProcess(handle, 1);
        CloseHandle(handle);

        if ok == 0 {
            let err = std::io::Error::last_os_error();
            return Err(WindowsError::ProcessError(format!(
                "Failed to terminate process {pid}: {err}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_is_alive() {
        let current_pid = std::process::id();
        assert!(is_pid_alive(current_pid));
    }

    #[test]
    fn test_invalid_pid_is_not_alive() {
        assert!(!is_pid_alive(99999999));
    }
}
