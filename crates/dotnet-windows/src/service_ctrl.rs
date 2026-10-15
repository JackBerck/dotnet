//! Native Windows Service Control Manager (SCM) integration.

use crate::error::{Result, WindowsError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SUCCESS,
};
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx,
    StartServiceW, SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_CONTROL_STOP,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START, SERVICE_START_PENDING,
    SERVICE_STATUS_PROCESS, SERVICE_STOP, SERVICE_STOPPED, SERVICE_STOP_PENDING,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsServiceState {
    Running,
    Stopped,
    StartPending,
    StopPending,
    NotInstalled,
    AccessDenied,
    Unknown,
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Query real-time status of a Windows Service.
pub fn get_service_status(service_name: &str) -> WindowsServiceState {
    if service_name.trim().is_empty() {
        return WindowsServiceState::NotInstalled;
    }

    let wide_name = to_wide(service_name);

    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            if err == ERROR_ACCESS_DENIED {
                return WindowsServiceState::AccessDenied;
            }
            return WindowsServiceState::Unknown;
        }

        let svc = OpenServiceW(scm, wide_name.as_ptr(), SERVICE_QUERY_STATUS);
        if svc.is_null() {
            let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            CloseServiceHandle(scm);
            if err == ERROR_SERVICE_DOES_NOT_EXIST {
                return WindowsServiceState::NotInstalled;
            } else if err == ERROR_ACCESS_DENIED {
                return WindowsServiceState::AccessDenied;
            }
            return WindowsServiceState::Unknown;
        }

        let mut ssp: SERVICE_STATUS_PROCESS = std::mem::zeroed();
        let mut bytes_needed: u32 = 0;

        let ok = QueryServiceStatusEx(
            svc,
            SC_STATUS_PROCESS_INFO,
            &mut ssp as *mut _ as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut bytes_needed,
        );

        CloseServiceHandle(svc);
        CloseServiceHandle(scm);

        if ok == 0 {
            return WindowsServiceState::Unknown;
        }

        match ssp.dwCurrentState {
            SERVICE_RUNNING => WindowsServiceState::Running,
            SERVICE_STOPPED => WindowsServiceState::Stopped,
            SERVICE_START_PENDING => WindowsServiceState::StartPending,
            SERVICE_STOP_PENDING => WindowsServiceState::StopPending,
            _ => WindowsServiceState::Unknown,
        }
    }
}

/// Start a Windows Service and await until RUNNING or timeout.
pub fn start_service(service_name: &str, timeout_secs: u64) -> Result<()> {
    let wide_name = to_wide(service_name);

    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(WindowsError::ProcessError(format!(
                "Failed to open SCM: {}",
                std::io::Error::last_os_error()
            )));
        }

        let svc = OpenServiceW(
            scm,
            wide_name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );
        if svc.is_null() {
            CloseServiceHandle(scm);
            return Err(WindowsError::ProcessError(format!(
                "Failed to open service '{}': {}",
                service_name,
                std::io::Error::last_os_error()
            )));
        }

        let started = StartServiceW(svc, 0, std::ptr::null_mut());
        if started == 0 {
            let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            if err != ERROR_SUCCESS {
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                return Err(WindowsError::ProcessError(format!(
                    "StartService failed for '{}' (code: {})",
                    service_name, err
                )));
            }
        }

        // Poll for running status
        let start_time = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let mut ssp: SERVICE_STATUS_PROCESS = std::mem::zeroed();
            let mut bytes_needed: u32 = 0;
            let ok = QueryServiceStatusEx(
                svc,
                SC_STATUS_PROCESS_INFO,
                &mut ssp as *mut _ as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            );

            if ok != 0 && ssp.dwCurrentState == SERVICE_RUNNING {
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                return Ok(());
            }

            if start_time.elapsed() > timeout {
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                return Err(WindowsError::ProcessError(format!(
                    "Timed out waiting for service '{}' to start",
                    service_name
                )));
            }

            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

/// Stop a Windows Service and await until STOPPED or timeout.
pub fn stop_service(service_name: &str, timeout_secs: u64) -> Result<()> {
    let wide_name = to_wide(service_name);

    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(WindowsError::ProcessError(format!(
                "Failed to open SCM: {}",
                std::io::Error::last_os_error()
            )));
        }

        let svc = OpenServiceW(scm, wide_name.as_ptr(), SERVICE_STOP | SERVICE_QUERY_STATUS);
        if svc.is_null() {
            CloseServiceHandle(scm);
            return Err(WindowsError::ProcessError(format!(
                "Failed to open service '{}': {}",
                service_name,
                std::io::Error::last_os_error()
            )));
        }

        let mut status: windows_sys::Win32::System::Services::SERVICE_STATUS = std::mem::zeroed();
        let stopped = ControlService(svc, SERVICE_CONTROL_STOP, &mut status);
        if stopped == 0 {
            let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            CloseServiceHandle(svc);
            CloseServiceHandle(scm);
            return Err(WindowsError::ProcessError(format!(
                "ControlService stop failed for '{}' (code: {})",
                service_name, err
            )));
        }

        // Poll for stopped status
        let start_time = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let mut ssp: SERVICE_STATUS_PROCESS = std::mem::zeroed();
            let mut bytes_needed: u32 = 0;
            let ok = QueryServiceStatusEx(
                svc,
                SC_STATUS_PROCESS_INFO,
                &mut ssp as *mut _ as *mut u8,
                std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                &mut bytes_needed,
            );

            if ok != 0 && ssp.dwCurrentState == SERVICE_STOPPED {
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                return Ok(());
            }

            if start_time.elapsed() > timeout {
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                return Err(WindowsError::ProcessError(format!(
                    "Timed out waiting for service '{}' to stop",
                    service_name
                )));
            }

            std::thread::sleep(Duration::from_millis(200));
        }
    }
}
