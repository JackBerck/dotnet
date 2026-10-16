//! Native Win32 Desktop Window implementation for Dotnet (No Web browser, pure Windows PC GUI).

use crate::error::{Result, WindowsError};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, InvalidateRect, HBRUSH, HFONT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics, LoadCursorW, LoadIconW,
    MoveWindow, PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow, SetWindowTextW,
    ShowWindow, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY,
    HCURSOR, HICON, IDC_ARROW, IDI_APPLICATION, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_RESTORE,
    SW_SHOW, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_SIZE, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

pub const CMD_WIN_START_ALL: usize = 201;
pub const CMD_WIN_STOP_ALL: usize = 202;
pub const CMD_WIN_RELOAD_NGINX: usize = 203;
pub const CMD_WIN_SCAN_ADOPT: usize = 204;
pub const CMD_WIN_SWITCH_PROFILE: usize = 205;
pub const CMD_WIN_REFRESH: usize = 206;

pub const CMD_TAB_SERVICES: usize = 301;
pub const CMD_TAB_TOOLS: usize = 302;
pub const CMD_TAB_PORTS: usize = 303;
pub const CMD_TAB_LOGS: usize = 304;

pub const CMD_SVC_ACTION_BASE: usize = 400; // 400 + i (toggle service i)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeWindowEvent {
    StartAll,
    StopAll,
    ReloadNginx,
    ScanAndAdoptTools,
    SwitchProfile,
    RefreshStatus,
    SwitchTab(usize),
    ToggleService(usize),
    WindowHiddenToTray,
    WindowClosed,
}

static mut EVENT_TX: Option<Sender<NativeWindowEvent>> = None;
static mut EDIT_HWND: HWND = std::ptr::null_mut();
static ACTIVE_TAB: AtomicUsize = AtomicUsize::new(0); // 0: Services, 1: Tools, 2: Ports, 3: Logs

pub struct NativeMainWindow {
    hwnd: HWND,
    edit_display: HWND,
    hfont: HFONT,
    hfont_mono: HFONT,
    bg_brush: HBRUSH,
    is_open: Arc<AtomicBool>,
}

unsafe impl Send for NativeMainWindow {}
unsafe impl Sync for NativeMainWindow {}

impl NativeMainWindow {
    /// Create and show the native Windows desktop application window.
    pub fn create(
        title: &str,
        width: i32,
        height: i32,
    ) -> Result<(Self, Receiver<NativeWindowEvent>)> {
        let (tx, rx) = channel();
        unsafe {
            EVENT_TX = Some(tx);
        }

        let class_name: Vec<u16> = "DotnetNativeMainWindowClass\0".encode_utf16().collect();
        let window_title: Vec<u16> = format!("{title}\0").encode_utf16().collect();

        unsafe {
            let hicon: HICON = LoadIconW(std::ptr::null_mut(), IDI_APPLICATION);
            let hcursor: HCURSOR = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
            let bg_brush = CreateSolidBrush(0x001e140f); // Dark Slate BGR: #0f141e

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(native_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: std::ptr::null_mut(),
                hIcon: hicon,
                hCursor: hcursor,
                hbrBackground: bg_brush,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = ((screen_w - width) / 2).max(0);
            let y = ((screen_h - height) / 2).max(0);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
                x,
                y,
                width,
                height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if hwnd.is_null() {
                return Err(WindowsError::SystemError(
                    "Failed to create native desktop window".to_string(),
                ));
            }

            // Create Segoe UI font for controls
            let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
            let hfont = CreateFontW(17, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, font_name.as_ptr());

            // Create Consolas monospace font for tabular output
            let mono_name: Vec<u16> = "Consolas\0".encode_utf16().collect();
            let hfont_mono =
                CreateFontW(17, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, mono_name.as_ptr());

            // Create Toolbar Buttons
            create_button(
                hwnd,
                "▶ Start All",
                20,
                20,
                110,
                36,
                CMD_WIN_START_ALL,
                hfont,
            );
            create_button(
                hwnd,
                "⏹ Stop All",
                140,
                20,
                110,
                36,
                CMD_WIN_STOP_ALL,
                hfont,
            );
            create_button(
                hwnd,
                "🔄 Reload Nginx",
                260,
                20,
                125,
                36,
                CMD_WIN_RELOAD_NGINX,
                hfont,
            );
            create_button(
                hwnd,
                "🔍 Scan & Adopt",
                395,
                20,
                130,
                36,
                CMD_WIN_SCAN_ADOPT,
                hfont,
            );
            create_button(
                hwnd,
                "⚡ Switch Profile",
                535,
                20,
                130,
                36,
                CMD_WIN_SWITCH_PROFILE,
                hfont,
            );
            create_button(hwnd, "⟳ Refresh", 675, 20, 100, 36, CMD_WIN_REFRESH, hfont);

            // Create Tab Navigation Buttons
            create_button(
                hwnd,
                "📋 Layanan (Services)",
                20,
                72,
                170,
                32,
                CMD_TAB_SERVICES,
                hfont,
            );
            create_button(
                hwnd,
                "🛠 Tools Terpasang",
                200,
                72,
                170,
                32,
                CMD_TAB_TOOLS,
                hfont,
            );
            create_button(
                hwnd,
                "🔌 Port Aktif (TCP)",
                380,
                72,
                170,
                32,
                CMD_TAB_PORTS,
                hfont,
            );
            create_button(
                hwnd,
                "📄 Log Peristiwa",
                560,
                72,
                150,
                32,
                CMD_TAB_LOGS,
                hfont,
            );

            // Create Multi-line Read-Only Rich Display Area
            let edit_class: Vec<u16> = "EDIT\0".encode_utf16().collect();
            let edit_display = CreateWindowExW(
                0,
                edit_class.as_ptr(),
                std::ptr::null(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | (ES_MULTILINE as u32)
                    | (ES_READONLY as u32)
                    | (ES_AUTOVSCROLL as u32),
                20,
                115,
                width - 60,
                height - 180,
                hwnd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            EDIT_HWND = edit_display;
            SendMessageW(edit_display, 0x0030, hfont_mono as usize, 1); // WM_SETFONT Consolas

            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);

            Ok((
                Self {
                    hwnd,
                    edit_display,
                    hfont,
                    hfont_mono,
                    bg_brush,
                    is_open: Arc::new(AtomicBool::new(true)),
                },
                rx,
            ))
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Update text displayed inside the native window content panel.
    pub fn update_content(&self, text: &str) {
        let wide: Vec<u16> = format!("{text}\0").encode_utf16().collect();
        unsafe {
            SetWindowTextW(self.edit_display, wide.as_ptr());
            InvalidateRect(self.hwnd, std::ptr::null(), 1);
        }
    }

    /// Show the window on screen and bring to front.
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_RESTORE);
            SetForegroundWindow(self.hwnd);
        }
    }

    /// Hide the window into system tray.
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Check if the window is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    /// Get currently selected active tab index.
    pub fn get_active_tab() -> usize {
        ACTIVE_TAB.load(Ordering::SeqCst)
    }
}

impl Drop for NativeMainWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hfont.is_null() {
                DeleteObject(self.hfont);
            }
            if !self.hfont_mono.is_null() {
                DeleteObject(self.hfont_mono);
            }
            if !self.bg_brush.is_null() {
                DeleteObject(self.bg_brush);
            }
            if !self.hwnd.is_null() {
                DestroyWindow(self.hwnd);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_button(
    parent: HWND,
    label: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cmd_id: usize,
    font: HFONT,
) -> HWND {
    let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();
    let btn_text: Vec<u16> = format!("{label}\0").encode_utf16().collect();

    let btn = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        btn_text.as_ptr(),
        WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32),
        x,
        y,
        w,
        h,
        parent,
        cmd_id as *mut std::ffi::c_void,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );

    if !font.is_null() {
        SendMessageW(btn, 0x0030, font as usize, 1); // WM_SETFONT
    }
    btn
}

unsafe extern "system" fn native_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            if let Some(ref sender) = EVENT_TX {
                match id {
                    CMD_WIN_START_ALL => {
                        let _ = sender.send(NativeWindowEvent::StartAll);
                    }
                    CMD_WIN_STOP_ALL => {
                        let _ = sender.send(NativeWindowEvent::StopAll);
                    }
                    CMD_WIN_RELOAD_NGINX => {
                        let _ = sender.send(NativeWindowEvent::ReloadNginx);
                    }
                    CMD_WIN_SCAN_ADOPT => {
                        let _ = sender.send(NativeWindowEvent::ScanAndAdoptTools);
                    }
                    CMD_WIN_SWITCH_PROFILE => {
                        let _ = sender.send(NativeWindowEvent::SwitchProfile);
                    }
                    CMD_WIN_REFRESH => {
                        let _ = sender.send(NativeWindowEvent::RefreshStatus);
                    }
                    CMD_TAB_SERVICES => {
                        ACTIVE_TAB.store(0, Ordering::SeqCst);
                        let _ = sender.send(NativeWindowEvent::SwitchTab(0));
                    }
                    CMD_TAB_TOOLS => {
                        ACTIVE_TAB.store(1, Ordering::SeqCst);
                        let _ = sender.send(NativeWindowEvent::SwitchTab(1));
                    }
                    CMD_TAB_PORTS => {
                        ACTIVE_TAB.store(2, Ordering::SeqCst);
                        let _ = sender.send(NativeWindowEvent::SwitchTab(2));
                    }
                    CMD_TAB_LOGS => {
                        ACTIVE_TAB.store(3, Ordering::SeqCst);
                        let _ = sender.send(NativeWindowEvent::SwitchTab(3));
                    }
                    _ => {}
                }
            }
            0
        }
        WM_SIZE => {
            let width = (lparam & 0xFFFF) as i32;
            let height = ((lparam >> 16) & 0xFFFF) as i32;
            if !EDIT_HWND.is_null() {
                MoveWindow(
                    EDIT_HWND,
                    20,
                    115,
                    (width - 55).max(200),
                    (height - 150).max(100),
                    1,
                );
            }
            0
        }
        WM_CLOSE => {
            // Minimize to system tray instead of destroying process
            ShowWindow(hwnd, SW_HIDE);
            if let Some(ref sender) = EVENT_TX {
                let _ = sender.send(NativeWindowEvent::WindowHiddenToTray);
            }
            0
        }
        WM_DESTROY => {
            if let Some(ref sender) = EVENT_TX {
                let _ = sender.send(NativeWindowEvent::WindowClosed);
            }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
