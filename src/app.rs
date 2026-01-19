use std::sync::mpsc;
use std::time::Duration;

use eframe::egui::{self, Color32, RichText};
use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

use crate::{autostart, settings, tray::TrayCommand};

/// Main application state and UI.
///
/// High-level flow:
/// - A Win32 tray thread sends [`TrayCommand`] values over an `mpsc` channel.
/// - `update()` drains the channel each frame and reacts:
///   - `Open` -> `show_window()` (un-minimize + focus)
///   - `Exit` -> `exit_app()` (close viewport)
/// - Clicking the window close button (X) does **not** exit: we cancel the close
///   request and minimize to tray instead.
///
/// Notes:
/// - We avoid `ViewportCommand::Visible(false)` because making the window invisible can
///   prevent repaints/updates on some systems, breaking tray “Open”.
/// - After restoring from tray, Windows can report a stale `close_requested()` for a few
///   frames; `ignore_close_frames` suppresses immediately hiding again.
pub struct SilliReminder {
    system_start: bool,
    background: bool,
    allow_close: bool,
    ignore_close_frames: u8,
    hwnd_set: bool,
    tray_rx: mpsc::Receiver<TrayCommand>,
}

impl SilliReminder {
    pub fn new(system_start: bool, background: bool, tray_rx: mpsc::Receiver<TrayCommand>) -> Self {
        Self {
            system_start,
            background,
            allow_close: false,
            ignore_close_frames: 0,
            hwnd_set: false,
            tray_rx,
        }
    }

    fn hide_to_tray(&mut self, ctx: &egui::Context) {
        self.background = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    fn show_window(&mut self, ctx: &egui::Context) {
        self.allow_close = false;
        self.background = false;
        // After re-opening from tray, Windows/egui can still report a stale
        // close request for a short moment. Ignore it for a couple frames.
        self.ignore_close_frames = 30;
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn exit_app(&mut self, ctx: &egui::Context) {
        self.allow_close = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        ctx.request_repaint();
    }
}

impl eframe::App for SilliReminder {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Match egui visuals to avoid a black flash when the OS restores the window
        // before egui has painted a frame.
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Capture the native window handle once so the tray thread can restore the window
        // even if egui isn't getting repaints while minimized.
        if !self.hwnd_set {
            if let Ok(handle) = frame.window_handle() {
                if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                    crate::tray::set_main_window_hwnd(win32.hwnd.get() as isize);
                    self.hwnd_set = true;
                }
            }
        }

        let commands: Vec<TrayCommand> = self.tray_rx.try_iter().collect();
        for cmd in commands {
            match cmd {
                TrayCommand::Open => self.show_window(ctx),
                TrayCommand::Exit => self.exit_app(ctx),
            }
        }

        // Close button (X): keep running and hide to tray.
        if self.ignore_close_frames > 0 {
            self.ignore_close_frames = self.ignore_close_frames.saturating_sub(1);
        }

        if !self.allow_close && !self.background && ctx.input(|i| i.viewport().close_requested()) {
            // Always cancel the close request; it may linger for several frames on Windows.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

            // Only treat it as an actual "user pressed X" when we're not in the
            // post-open suppression window.
            if self.ignore_close_frames == 0 {
                self.hide_to_tray(ctx);
            }
        }

        if self.background {
            // Background work goes here.
            // Keep the window minimized instead of invisible so the event loop stays alive.
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            ctx.request_repaint_after(Duration::from_secs(1));
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Silly Reminder")
                        .size(40.0)
                        .strong()
                        .color(Color32::KHAKI),
                );

                ui.label(RichText::new("Ustawienia").size(25.0).strong().color(Color32::KHAKI));
                ui.group(|ui| {
                    let response = ui.checkbox(
                        &mut self.system_start,
                        "Włącz podaczas włączania systemu",
                    );

                    if response.changed() {
                        println!("system_start toggled -> {}", self.system_start);

                        if let Err(err) = autostart::set_enabled(self.system_start) {
                            eprintln!("failed to update autostart: {err}");
                        }

                        if let Err(err) = settings::save_setting(self.system_start) {
                            eprintln!("failed to save setting: {err}");
                        }
                    }
                });

                ui.label(RichText::new("Dodaj").size(25.0).strong().color(Color32::KHAKI));
                ui.label(RichText::new("Zaplanowane").size(25.0).strong().color(Color32::KHAKI));
            });
        });
    }
}
