//! Native Windows Process inspection and lifecycle controls.

use crate::error::{Result, WindowsError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, OpenProcess, OpenProcessToken, TerminateProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_NORMAL;

const STILL_ACTIVE: u32 = 259;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Check if current process is running with Administrator / elevated privileges.
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let res = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );
        CloseHandle(token);

        res != 0 && elevation.TokenIsElevated != 0
    }
}

/// Relaunch the current executable with Administrator elevation via ShellExecuteW ("runas").
pub fn elevate_self() -> bool {
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_str = args
        .iter()
        .map(|a| {
            if a.contains(' ') {
                format!("\"{}\"", a)
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let wide_verb = to_wide("runas");
    let wide_file = to_wide(&current_exe.to_string_lossy());
    let wide_params = to_wide(&args_str);

    unsafe {
        let res = ShellExecuteW(
            std::ptr::null_mut(),
            wide_verb.as_ptr(),
            wide_file.as_ptr(),
            wide_params.as_ptr(),
            std::ptr::null(),
            SW_NORMAL,
        );
        (res as usize) > 32
    }
}

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
