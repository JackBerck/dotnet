//! Single instance mutex enforcement for Windows applications.

use crate::error::{Result, WindowsError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

/// RAII Guard for Windows Single Instance Named Mutex.
pub struct SingleInstanceGuard {
    handle: HANDLE,
}

unsafe impl Send for SingleInstanceGuard {}
unsafe impl Sync for SingleInstanceGuard {}

impl SingleInstanceGuard {
    /// Try acquiring the single instance mutex.
    /// Returns `Ok(Some(guard))` if this is the first running instance.
    /// Returns `Ok(None)` if another instance is already running.
    pub fn acquire(name: &str) -> Result<Option<Self>> {
        let wide_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 1, wide_name.as_ptr());
            if handle.is_null() {
                return Err(WindowsError::SystemError(
                    "Failed to create application single-instance mutex".to_string(),
                ));
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                Ok(None)
            } else {
                Ok(Some(Self { handle }))
            }
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_mutex_lifecycle() {
        let mutex_name = "Global\\Dotnet_Test_Single_Instance_Mutex_12345";
        let guard1 =
            SingleInstanceGuard::acquire(mutex_name).expect("First acquire should succeed");
        assert!(guard1.is_some());

        let guard2 =
            SingleInstanceGuard::acquire(mutex_name).expect("Second acquire should succeed");
        assert!(
            guard2.is_none(),
            "Second instance must be rejected while first is held"
        );

        drop(guard1);

        let guard3 = SingleInstanceGuard::acquire(mutex_name)
            .expect("Third acquire after drop should succeed");
        assert!(guard3.is_some(), "After drop, acquire must succeed");
    }
}
