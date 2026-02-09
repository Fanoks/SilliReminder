use std::sync::OnceLock;
use std::sync::mpsc::Sender;

use eframe::egui;

static REPAINT_CTX: OnceLock<egui::Context> = OnceLock::new();

pub fn set_repaint_context(ctx: egui::Context) {
    let _ = REPAINT_CTX.set(ctx);
}

fn request_repaint() {
    if let Some(ctx) = REPAINT_CTX.get() {
        ctx.request_repaint();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrayCommand {
    Open,
    Exit,
}

pub fn set_main_window_hwnd(hwnd: isize) {
    #[cfg(target_os = "windows")]
    {
        win32::set_main_window_hwnd(hwnd);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
    }
}

#[cfg(target_os = "windows")]
#[path = "tray/win32.rs"]
mod win32;

pub fn spawn_tray(sender: Sender<TrayCommand>) {
    #[cfg(target_os = "windows")]
    {
        win32::spawn_tray(sender, request_repaint);
        return;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = sender;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrayNotificationKind {
    Info,
    Warning,
    Error,
}

pub fn notify(title: &str, body: &str, kind: TrayNotificationKind) {
    #[cfg(target_os = "windows")]
    {
        win32::enqueue_notification(title, body, kind);
        return;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (title, body, kind);
    }
}
