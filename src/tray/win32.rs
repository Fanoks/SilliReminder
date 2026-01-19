//! Windows tray backend (Win32).
//!
//! This module intentionally contains the *only* platform-specific and `unsafe` code needed
//! for the system tray icon.
//!
//! Why `unsafe` exists here:
//! - Win32 APIs are FFI calls and are marked `unsafe` by the `windows` crate.
//! - The tray icon callback is implemented as a Win32 window procedure, which must be an
//!   `unsafe extern "system" fn`.
//!
//! Safety model (invariants we rely on):
//! - `MAIN_HWND` is set from eframe's `raw-window-handle` Win32 handle and is only used for
//!   basic message/show/focus APIs. We do not dereference it; we pass it back to Win32.
//! - The tray window created in `run_tray_loop` lives on the tray thread and the message loop
//!   runs until `PostQuitMessage` is called.
//! - We treat all Win32 return values as best-effort; failures are non-fatal.

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use super::TrayCommand;

static TRAY_SENDER: OnceLock<Sender<TrayCommand>> = OnceLock::new();
static REQUEST_REPAINT: OnceLock<fn()> = OnceLock::new();
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

const WM_TRAYICON: u32 = WM_APP + 1;
const ID_MENU_OPEN: usize = 1;
const ID_MENU_EXIT: usize = 2;
const RESTORE_DELAY_MS: u64 = 200;

pub(super) fn set_main_window_hwnd(hwnd: isize) {
    MAIN_HWND.store(hwnd, Ordering::Relaxed);
}

/// Starts the tray thread and creates the hidden message window.
///
/// Safe to call from the UI thread; the Win32 message loop runs on the spawned thread.
pub(super) fn spawn_tray(sender: Sender<TrayCommand>, request_repaint: fn()) {
    let _ = TRAY_SENDER.set(sender);
    let _ = REQUEST_REPAINT.set(request_repaint);

    std::thread::spawn(move || run_tray_loop());
}

fn request_repaint() {
    if let Some(cb) = REQUEST_REPAINT.get() {
        cb();
    }
}

/// "Nudges" the UI window's message loop.
///
/// When the app is minimized, some systems can be slow to wake the UI thread.
/// Posting `WM_NULL` is a harmless way to prompt message processing.
fn wake_main_window() {
    let raw = MAIN_HWND.load(Ordering::Relaxed);
    if raw == 0 {
        return;
    }

    let hwnd = HWND(raw as *mut core::ffi::c_void);
    if hwnd.0.is_null() {
        return;
    }

    unsafe {
        // SAFETY: Win32 FFI call. `hwnd` is an OS handle; we don't dereference it.
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    }
}

/// Restores the main window after a small delay.
///
/// We delay the restore to reduce the "black panel" flash: it gives egui a moment
/// to produce a frame before the OS shows the restored surface.
fn restore_main_window_delayed(delay_ms: u64) {
    let raw = MAIN_HWND.load(Ordering::Relaxed);
    if raw == 0 {
        return;
    }

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        let raw = MAIN_HWND.load(Ordering::Relaxed);
        if raw == 0 {
            return;
        }

        let hwnd = HWND(raw as *mut core::ffi::c_void);
        if hwnd.0.is_null() {
            return;
        }

        unsafe {
            // SAFETY: Win32 FFI calls. Using an HWND is safe as an opaque handle.
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
    });
}

/// Creates the tray icon and runs a standard Win32 message loop.
fn run_tray_loop() {
    let hmodule = unsafe { GetModuleHandleW(PCWSTR::null()) }.unwrap_or_default();

    let class_name = w!("SilliReminderTrayWindow");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: hmodule.into(),
        lpszClassName: class_name,
        ..Default::default()
    };

    unsafe {
        // SAFETY: Win32 registration of a window class for our hidden tray window.
        RegisterClassW(&wc);
    }

    let hwnd = unsafe {
        // SAFETY: Win32 creates a message-only-ish hidden window; we don't show it.
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!(""),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hmodule.into()),
            None,
        )
    }
    .unwrap_or_default();

    if hwnd.0.is_null() {
        return;
    }

    // Prefer the icon embedded into the EXE resources (icon id 1).
    let mut hicon = unsafe { LoadIconW(Some(hmodule.into()), PCWSTR(1usize as *const u16)) }
        .unwrap_or_default();
    if hicon.0.is_null() {
        hicon = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default();
    }

    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = hicon;

    let tip = "SilliReminder";
    let mut wide: Vec<u16> = tip.encode_utf16().collect();
    wide.push(0);
    for (dst, src) in nid.szTip.iter_mut().zip(wide.iter()) {
        *dst = *src;
    }

    unsafe {
        // SAFETY: Adds the tray icon. `nid` lives for the duration of the message loop.
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        unsafe {
            // SAFETY: Standard Win32 message pump.
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        // SAFETY: Best-effort cleanup of the tray icon and window.
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        let _ = DestroyWindow(hwnd);
    }
}

/// Win32 window procedure for the hidden tray window.
///
/// Required by Win32. Keep it small and side-effect-light.
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam.0 as u32;
            match event {
                WM_LBUTTONUP => {
                    if let Some(sender) = TRAY_SENDER.get() {
                        let _ = sender.send(TrayCommand::Open);
                        request_repaint();
                        wake_main_window();
                        restore_main_window_delayed(RESTORE_DELAY_MS);
                    }
                }
                WM_RBUTTONUP => {
                    show_menu(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as usize;
            if let Some(sender) = TRAY_SENDER.get() {
                match id {
                    ID_MENU_OPEN => {
                        let _ = sender.send(TrayCommand::Open);
                        request_repaint();
                        wake_main_window();
                        restore_main_window_delayed(RESTORE_DELAY_MS);
                    }
                    ID_MENU_EXIT => {
                        let _ = sender.send(TrayCommand::Exit);
                        request_repaint();
                        unsafe {
                            // SAFETY: Win32 FFI call; quits the tray thread message loop.
                            PostQuitMessage(0);
                        }
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                // SAFETY: Win32 FFI call; quits the tray thread message loop.
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Builds and shows the right-click popup menu (Open/Exit).
fn show_menu(hwnd: HWND) {
    let hmenu = unsafe { CreatePopupMenu() }.unwrap_or_default();
    if hmenu.0.is_null() {
        return;
    }

    unsafe {
        // SAFETY: Win32 FFI calls to populate the menu.
        let _ = AppendMenuW(hmenu, MF_STRING, ID_MENU_OPEN, w!("Open"));
        let _ = AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT, w!("Exit"));
    }

    let mut point = POINT::default();
    unsafe {
        // SAFETY: Win32 FFI calls to show the menu and then destroy it.
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            hmenu,
            TPM_RIGHTBUTTON,
            point.x,
            point.y,
            Some(0),
            hwnd,
            None,
        );
        let _ = DestroyMenu(hmenu);
    }
}
