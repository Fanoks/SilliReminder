#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod autostart;
mod db_operations;
mod settings;
mod tray;
mod widgets;

use eframe::egui;
use std::sync::mpsc;

include!(concat!(env!("OUT_DIR"), "/embedded_icon_png.rs"));

fn window_icon() -> Option<egui::IconData> {
    if ICON_PNG.is_empty() {
        return None;
    }

    eframe::icon_data::from_png_bytes(ICON_PNG).ok()
}

fn is_background_mode() -> bool {
    std::env::args().any(|arg| arg == "--background" || arg == "--autostart")
}

fn is_autostart_launch() -> bool {
    std::env::args().any(|arg| arg == "--autostart")
}

fn main() -> eframe::Result<()> {
    let system_start = settings::load_setting().unwrap_or(false);
    let background = is_background_mode();
    let autostart_launch = is_autostart_launch();

    let (tray_tx, tray_rx) = mpsc::channel();
    tray::spawn_tray(tray_tx);

    // Ensure registry matches the saved setting at startup.
    if let Err(err) = autostart::set_enabled(system_start) {
        eprintln!("failed to sync autostart on startup: {err}");
    }

    if autostart_launch && !system_start {
        // Launched from a stale autostart entry, but the setting is now off.
        // Exit without showing UI.
        return Ok(());
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([500.0, 600.0])
        // Tray-first UX (like AV apps): keep the app out of the taskbar.
        .with_taskbar(false)
        // Avoid `Visible(false)` because it can stop the repaint/update loop,
        // making tray “Open” unreliable.
        .with_visible(true);

    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Silly Reminder",
        options,
        Box::new(move |cc| {
            tray::set_repaint_context(cc.egui_ctx.clone());
            Ok(Box::new(app::SilliReminder::new(system_start, background, tray_rx)))
        }),
    )
}
