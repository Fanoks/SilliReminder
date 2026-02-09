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

use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;

use std::collections::VecDeque;

use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use super::TrayCommand;
use crate::i18n;
use crate::tray::TrayNotificationKind;

static TRAY_SENDER: OnceLock<Sender<TrayCommand>> = OnceLock::new();
static REQUEST_REPAINT: OnceLock<fn()> = OnceLock::new();
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
static NOTIFY_QUEUE: OnceLock<Mutex<VecDeque<QueuedNotification>>> = OnceLock::new();

const WM_TRAYICON: u32 = WM_APP + 1;
const WM_TRAY_NOTIFY: u32 = WM_APP + 2;
const ID_MENU_OPEN: usize = 1;
const ID_MENU_EXIT: usize = 2;
const RESTORE_DELAY_MS: u64 = 200;

#[derive(Debug, Clone)]
struct QueuedNotification {
    title: String,
    body: String,
    kind: TrayNotificationKind,
}

pub(super) fn set_main_window_hwnd(hwnd: isize) {
    MAIN_HWND.store(hwnd, Ordering::Relaxed);
}

/// Starts the tray thread and creates the hidden message window.
///
/// Safe to call from the UI thread; the Win32 message loop runs on the spawned thread.
pub(super) fn spawn_tray(sender: Sender<TrayCommand>, request_repaint: fn()) {
    let _ = TRAY_SENDER.set(sender);
    let _ = REQUEST_REPAINT.set(request_repaint);
    let _ = NOTIFY_QUEUE.set(Mutex::new(VecDeque::new()));

    std::thread::spawn(move || run_tray_loop());
}

pub(super) fn enqueue_notification(title: &str, body: &str, kind: TrayNotificationKind) {
    let Some(queue) = NOTIFY_QUEUE.get() else {
        return;
    };

    {
        let mut q = queue.lock().unwrap_or_else(|p| p.into_inner());
        q.push_back(QueuedNotification {
            title: title.to_owned(),
            body: body.to_owned(),
            kind,
        });
    }

    let raw = TRAY_HWND.load(Ordering::Relaxed);
    if raw == 0 {
        return;
    }
    let hwnd = HWND(raw as *mut core::ffi::c_void);
    if hwnd.0.is_null() {
        return;
    }

    unsafe {
        // SAFETY: Win32 FFI call. `hwnd` is an OS handle; we don't dereference it.
        let _ = PostMessageW(Some(hwnd), WM_TRAY_NOTIFY, WPARAM(0), LPARAM(0));
    }
}

fn copy_wide_trunc(dst: &mut [u16], s: &str) {
    let mut it = s.encode_utf16();
    if dst.is_empty() {
        return;
    }

    let mut i = 0usize;
    while i + 1 < dst.len() {
        match it.next() {
            Some(ch) => {
                dst[i] = ch;
                i += 1;
            }
            None => break,
        }
    }
    dst[i] = 0;
}

fn show_balloon(hwnd: HWND, n: &QueuedNotification) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_INFO;

    copy_wide_trunc(&mut nid.szInfoTitle, &n.title);
    copy_wide_trunc(&mut nid.szInfo, &n.body);

    nid.dwInfoFlags = match n.kind {
        TrayNotificationKind::Info => NIIF_INFO,
        TrayNotificationKind::Warning => NIIF_WARNING,
        TrayNotificationKind::Error => NIIF_ERROR,
    };

    unsafe {
        // SAFETY: Best-effort Win32 notification update.
        let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}

fn spawn_message_box(n: QueuedNotification) {
    // Spawn a dedicated thread so we don't block the tray message pump.
    std::thread::spawn(move || {
        let mut title_w: Vec<u16> = n.title.encode_utf16().collect();
        title_w.push(0);

        let mut body_w: Vec<u16> = n.body.encode_utf16().collect();
        body_w.push(0);

        let icon = match n.kind {
            TrayNotificationKind::Info => MB_ICONINFORMATION,
            TrayNotificationKind::Warning => MB_ICONWARNING,
            TrayNotificationKind::Error => MB_ICONERROR,
        };

        let flags = MB_OK | icon | MB_TOPMOST | MB_SETFOREGROUND;

        unsafe {
            // SAFETY: Win32 modal dialog. Strings are null-terminated and live for the call.
            let _ = MessageBoxW(
                None,
                PCWSTR(body_w.as_ptr()),
                PCWSTR(title_w.as_ptr()),
                flags,
            );
        }
    });
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

    TRAY_HWND.store(hwnd.0 as isize, Ordering::Relaxed);

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

    let tip = i18n::tray_tooltip(i18n::language());
    copy_wide_trunc(&mut nid.szTip, tip);

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
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
        WM_TRAY_NOTIFY => {
            if let Some(queue) = NOTIFY_QUEUE.get() {
                let n = {
                    let mut q = queue.lock().unwrap_or_else(|p| p.into_inner());
                    q.pop_front()
                };
                if let Some(n) = n {
                    show_balloon(hwnd, &n);
                    spawn_message_box(n);
                }
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

    fn wide_null(s: &str) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0);
        v
    }

    let lang = i18n::language();
    let open_w = wide_null(i18n::tray_open(lang));
    let exit_w = wide_null(i18n::tray_exit(lang));

    unsafe {
        // SAFETY: Win32 FFI calls to populate the menu.
        let _ = AppendMenuW(hmenu, MF_STRING, ID_MENU_OPEN, PCWSTR(open_w.as_ptr()));
        let _ = AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT, PCWSTR(exit_w.as_ptr()));
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
