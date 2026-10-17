//! Windows Registry manipulation (User and System PATH) and environment broadcast.

use crate::error::{Result, WindowsError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HWND};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn read_registry_string(root: HKEY, subkey_name: &str, val_name: &str) -> Result<String> {
    let subkey = to_wide(subkey_name);
    let val = to_wide(val_name);

    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(root, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) };

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    let mut buf_len: u32 = 0;
    let mut val_type: u32 = 0;

    // Probe length
    let status = unsafe {
        RegQueryValueExW(
            hkey,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            std::ptr::null_mut(),
            &mut buf_len,
        )
    };

    if status != ERROR_SUCCESS {
        unsafe {
            RegCloseKey(hkey);
        }
        if status == 2 {
            // ERROR_FILE_NOT_FOUND
            return Ok(String::new());
        }
        return Err(WindowsError::RegistryError(status));
    }

    let mut buffer: Vec<u16> = vec![0u16; (buf_len / 2) as usize];
    let status = unsafe {
        RegQueryValueExW(
            hkey,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            buffer.as_mut_ptr() as *mut _,
            &mut buf_len,
        )
    };

    unsafe {
        RegCloseKey(hkey);
    }

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    // Strip trailing null characters
    while let Some(&0) = buffer.last() {
        buffer.pop();
    }

    Ok(String::from_utf16_lossy(&buffer))
}

/// Reads user PATH directly from HKCU\Environment without expanding variable references.
pub fn get_user_path() -> Result<String> {
    read_registry_string(HKEY_CURRENT_USER, "Environment", "Path")
}

/// Reads system PATH directly from HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment.
pub fn get_system_path() -> Result<String> {
    read_registry_string(
        HKEY_LOCAL_MACHINE,
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        "Path",
    )
}

/// Writes user PATH to HKCU\Environment as REG_EXPAND_SZ and broadcasts WM_SETTINGCHANGE.
pub fn set_user_path(new_path: &str) -> Result<()> {
    let subkey = to_wide("Environment");
    let val_name = to_wide("Path");
    let wide_val = to_wide(new_path);

    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;
    let status = unsafe {
        RegSetValueExW(
            hkey,
            val_name.as_ptr(),
            0,
            REG_EXPAND_SZ,
            wide_val.as_ptr() as *const _,
            byte_len,
        )
    };

    unsafe {
        RegCloseKey(hkey);
    }

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    broadcast_environment_change()
}

/// Broadcasts WM_SETTINGCHANGE to notify running applications of environment variable updates.
pub fn broadcast_environment_change() -> Result<()> {
    let env_str = to_wide("Environment");
    let mut result_code: usize = 0;

    let res = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST as HWND,
            WM_SETTINGCHANGE,
            0,
            env_str.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            2000, // 2 second timeout
            &mut result_code,
        )
    };

    if res == 0 {
        tracing::warn!("WM_SETTINGCHANGE broadcast timed out or failed (non-fatal)");
    }

    Ok(())
}

const RUN_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Reads a command from HKCU\Software\Microsoft\Windows\CurrentVersion\Run.
pub fn get_run_key(val_name: &str) -> Result<String> {
    read_registry_string(HKEY_CURRENT_USER, RUN_SUBKEY, val_name)
}

/// Sets a command in HKCU\Software\Microsoft\Windows\CurrentVersion\Run (REG_SZ).
pub fn set_run_key(val_name: &str, command: &str) -> Result<()> {
    let subkey = to_wide(RUN_SUBKEY);
    let val = to_wide(val_name);
    let wide_val = to_wide(command);

    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;
    let status = unsafe {
        RegSetValueExW(
            hkey,
            val.as_ptr(),
            0,
            REG_SZ,
            wide_val.as_ptr() as *const _,
            byte_len,
        )
    };

    unsafe {
        RegCloseKey(hkey);
    }

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    Ok(())
}

/// Deletes a command from HKCU\Software\Microsoft\Windows\CurrentVersion\Run.
pub fn delete_run_key(val_name: &str) -> Result<()> {
    let subkey = to_wide(RUN_SUBKEY);
    let val = to_wide(val_name);

    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(WindowsError::RegistryError(status));
    }

    let status = unsafe { RegDeleteValueW(hkey, val.as_ptr()) };

    unsafe {
        RegCloseKey(hkey);
    }

    if status != ERROR_SUCCESS && status != 2 {
        return Err(WindowsError::RegistryError(status));
    }

    Ok(())
}
