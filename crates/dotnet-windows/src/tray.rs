//! Native Win32 System Tray implementation using Shell_NotifyIconW.

use crate::error::{Result, WindowsError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIIF_WARNING, NIM_ADD,
    NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetMessageW, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    TrackPopupMenu, TranslateMessage, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
    WNDCLASSW,
};

const WM_TRAY_CALLBACK: u32 = WM_USER + 1;

pub const CMD_OPEN_DASHBOARD: usize = 101;
pub const CMD_START_ALL: usize = 102;
pub const CMD_STOP_ALL: usize = 103;
pub const CMD_RELOAD_NGINX: usize = 104;
pub const CMD_SWITCH_PROFILE: usize = 105;
pub const CMD_CHECK_PORTS: usize = 106;
pub const CMD_EXIT: usize = 107;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    OpenDashboard,
    StartAll,
    StopAll,
    ReloadNginx,
    SwitchProfile,
    CheckPorts,
    Exit,
}

static mut EVENT_SENDER: Option<Sender<TrayEvent>> = None;

pub struct SystemTray {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    is_active: Arc<AtomicBool>,
}

unsafe impl Send for SystemTray {}
unsafe impl Sync for SystemTray {}

impl SystemTray {
    #[must_use]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Initialize native Windows System Tray and return handle with event Receiver.
    pub fn create(tooltip: &str) -> Result<(Self, Receiver<TrayEvent>)> {
        let (tx, rx) = channel();
        unsafe {
            EVENT_SENDER = Some(tx);
        }

        let class_name: Vec<u16> = "DotnetTrayClass\0".encode_utf16().collect();
        let window_name: Vec<u16> = "DotnetTrayWindow\0".encode_utf16().collect();

        unsafe {
            let wc = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(tray_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: std::ptr::null_mut(),
                hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if hwnd.is_null() {
                return Err(WindowsError::SystemError(
                    "Failed to create hidden window for tray icon".to_string(),
                ));
            }

            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY_CALLBACK;
            nid.hIcon = LoadIconW(std::ptr::null_mut(), IDI_APPLICATION);

            // Set tooltip
            let tooltip_u16: Vec<u16> = tooltip.encode_utf16().collect();
            let copy_len = tooltip_u16.len().min(nid.szTip.len() - 1);
            nid.szTip[..copy_len].copy_from_slice(&tooltip_u16[..copy_len]);

            if Shell_NotifyIconW(NIM_ADD, &nid) == 0 {
                return Err(WindowsError::SystemError(
                    "Shell_NotifyIconW(NIM_ADD) failed".to_string(),
                ));
            }

            Ok((
                Self {
                    hwnd,
                    nid,
                    is_active: Arc::new(AtomicBool::new(true)),
                },
                rx,
            ))
        }
    }

    /// Display balloon notification from the system tray icon.
    pub fn show_balloon(&mut self, title: &str, message: &str, is_warning: bool) -> Result<()> {
        unsafe {
            self.nid.uFlags |= NIF_INFO;
            self.nid.dwInfoFlags = if is_warning { NIIF_WARNING } else { NIIF_INFO };

            // Reset buffers
            self.nid.szInfoTitle = [0; 64];
            self.nid.szInfo = [0; 256];

            let title_u16: Vec<u16> = title.encode_utf16().collect();
            let copy_title = title_u16.len().min(self.nid.szInfoTitle.len() - 1);
            self.nid.szInfoTitle[..copy_title].copy_from_slice(&title_u16[..copy_title]);

            let msg_u16: Vec<u16> = message.encode_utf16().collect();
            let copy_msg = msg_u16.len().min(self.nid.szInfo.len() - 1);
            self.nid.szInfo[..copy_msg].copy_from_slice(&msg_u16[..copy_msg]);

            if Shell_NotifyIconW(NIM_MODIFY, &self.nid) == 0 {
                return Err(WindowsError::SystemError(
                    "Failed to show tray balloon notification".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Pump Windows messages. Blocking call until exit.
    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    /// Remove the icon from Windows notification area.
    pub fn remove(&mut self) {
        if self.is_active.swap(false, Ordering::SeqCst) {
            unsafe {
                Shell_NotifyIconW(NIM_DELETE, &self.nid);
            }
        }
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        self.remove();
    }
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY_CALLBACK => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP {
                show_tray_popup_menu(hwnd);
            } else if event == WM_LBUTTONUP {
                if let Some(ref sender) = EVENT_SENDER {
                    let _ = sender.send(TrayEvent::OpenDashboard);
                }
            }
            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            let tray_event = match id {
                CMD_OPEN_DASHBOARD => Some(TrayEvent::OpenDashboard),
                CMD_START_ALL => Some(TrayEvent::StartAll),
                CMD_STOP_ALL => Some(TrayEvent::StopAll),
                CMD_RELOAD_NGINX => Some(TrayEvent::ReloadNginx),
                CMD_SWITCH_PROFILE => Some(TrayEvent::SwitchProfile),
                CMD_CHECK_PORTS => Some(TrayEvent::CheckPorts),
                CMD_EXIT => Some(TrayEvent::Exit),
                _ => None,
            };

            if let Some(ev) = tray_event {
                if let Some(ref sender) = EVENT_SENDER {
                    let _ = sender.send(ev);
                }
                if ev == TrayEvent::Exit {
                    PostQuitMessage(0);
                }
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_popup_menu(hwnd: HWND) {
    let mut pt: POINT = std::mem::zeroed();
    GetCursorPos(&mut pt);

    let menu = CreatePopupMenu();

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    AppendMenuW(
        menu,
        MF_STRING,
        CMD_OPEN_DASHBOARD,
        to_wide("Open Dashboard").as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_START_ALL,
        to_wide("Start All Services").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_STOP_ALL,
        to_wide("Stop All Services").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_RELOAD_NGINX,
        to_wide("Reload Nginx").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_SWITCH_PROFILE,
        to_wide("Switch Profile (Standalone / Docker)").as_ptr(),
    );
    AppendMenuW(
        menu,
        MF_STRING,
        CMD_CHECK_PORTS,
        to_wide("Check Port Conflicts").as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, CMD_EXIT, to_wide("Exit Dotnet").as_ptr());

    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    DestroyMenu(menu);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_event_definitions() {
        assert_eq!(TrayEvent::OpenDashboard, TrayEvent::OpenDashboard);
        assert_ne!(TrayEvent::StartAll, TrayEvent::StopAll);
    }
}
