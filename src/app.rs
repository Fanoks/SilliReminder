use std::sync::mpsc;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

use chrono::{Local, NaiveDate};
use eframe::egui::{self, RichText};
use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
use rusqlite::Connection;

use crate::{autostart, db_operations, settings, tray::TrayCommand};

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
    selected_date: NaiveDate,
    note_input: String,
    db: Option<Rc<RefCell<Connection>>>,
}

impl SilliReminder {
    pub fn new(system_start: bool, background: bool, tray_rx: mpsc::Receiver<TrayCommand>) -> Self {
        let db = match db_operations::get_db() {
            Ok(db) => Some(db),
            Err(err) => {
                eprintln!("failed to open database: {err}");
                None
            }
        };

        Self {
            system_start,
            background,
            allow_close: false,
            ignore_close_frames: 0,
            hwnd_set: false,
            tray_rx,
            selected_date: Local::now().date_naive(),
            note_input: String::new(),
            db,
        }
    }

    fn ui_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.vertical_centered(|ui| {
                    self.ui_header(ui);
                    self.ui_settings(ui);
                    self.ui_sections(ui);
                });

                // Fill the remaining space with the planned list.
                let remaining = ui.available_size();
                ui.allocate_ui(remaining, |ui| {
                    ui.vertical_centered(|ui| {
                        self.ui_planed(ui);
                    });
                });
            });
        });
    }

    fn ui_header(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(
            RichText::new("Silly Reminder")
                .size(40.0)
                .strong()
                .color(accent),
        );
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(RichText::new("Ustawienia").size(25.0).strong().color(accent));
        ui.group(|ui| {
            let response = ui.checkbox(&mut self.system_start, "Włącz podaczas włączania systemu");

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
    }

    fn ui_sections(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(RichText::new("Dodaj").size(25.0).strong().color(accent));
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            let row_h = ui.spacing().interact_size.y;
            let mut date_changed = false;
            let mut note_changed = false;
            let mut add_clicked = false;

            ui.horizontal(|ui| {
                let date_response: egui::Response = ui.add_sized(
                    egui::vec2(120.0, row_h),
                    crate::widgets::DatePickerPlButton::new(&mut self.selected_date)
                        .id_salt("reminder_date")
                        .format("%Y-%m-%d"),
                );
                date_changed = date_response.changed();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    add_clicked = ui
                        .add_sized(egui::vec2(70.0, row_h), egui::Button::new("Dodaj"))
                        .clicked();

                    let note_response: egui::Response = ui.add_sized(
                        egui::vec2(ui.available_width(), row_h),
                        egui::TextEdit::singleline(&mut self.note_input)
                            .id_salt("note_input")
                            .hint_text("Notatka..."),
                    );
                    note_changed = note_response.changed();
                });
            });

            if add_clicked {
                if let Some(db) = &self.db {
                    let note = self.note_input.trim();
                    if note.is_empty() {
                        eprintln!("note is empty; nothing inserted");
                    } else {
                        match db_operations::insert_reminder(&db.borrow(), self.selected_date, note) {
                            Ok(id) => {
                                println!("Dodano #{id}: {}, {}", self.selected_date, note);
                                self.note_input.clear();
                            }
                            Err(err) => eprintln!("failed to insert reminder: {err}"),
                        }
                    }
                } else {
                    eprintln!("database not available");
                }
            }

            if date_changed {
                println!("Selected date -> {}", self.selected_date);
            }

            if note_changed {
                println!("Note -> {}", self.note_input);
            }
        });
    }
    
    fn ui_planed(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(RichText::new("Zaplanowane").size(28.0).strong().color(accent));
        ui.group(|ui| {
            ui.set_min_size(ui.available_size());

            ui.vertical(|ui| {
                let Some(db) = &self.db else {
                    ui.label("Brak bazy danych");
                    return;
                };

                match db_operations::list_reminders(&db.borrow()) {
                    Ok(reminders) => {
                        if reminders.is_empty() {
                            ui.label("(pusto)");
                        } else {
                            let mut delete_id: Option<i64> = None;

                            egui::ScrollArea::vertical()
                                .max_height(ui.available_height())
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for r in reminders.iter() {
                                        ui.push_id(r.id, |ui| {
                                            egui::Frame::NONE
                                                .fill(ui.visuals().faint_bg_color)
                                                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                                                .corner_radius(egui::CornerRadius::same(6))
                                                .inner_margin(egui::Margin::symmetric(8, 6))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        let text_size = 18.0;
                                                        let row_h = ui
                                                            .spacing()
                                                            .interact_size
                                                            .y
                                                            .max(text_size + 10.0);

                                                        let row_text = RichText::new(format!(
                                                            "{}  -  {}",
                                                            r.date, r.note
                                                        ))
                                                        .size(text_size)
                                                        .color(ui.visuals().text_color());
                                                        ui.label(row_text);

                                                        let remaining = ui.available_width();
                                                        ui.allocate_ui_with_layout(
                                                            egui::vec2(remaining, 0.0),
                                                            egui::Layout::right_to_left(egui::Align::Center),
                                                            |ui| {
                                                                let danger = ui.visuals().error_fg_color;
                                                                let x = egui::Button::new(
                                                                    RichText::new("X")
                                                                        .size(22.0)
                                                                        .color(danger),
                                                                );
                                                                if ui.add_sized(egui::vec2(36.0, row_h), x).clicked() {
                                                                    delete_id = Some(r.id);
                                                                }
                                                            },
                                                        );
                                                    });
                                                });
                                            ui.add_space(4.0);
                                        });
                                    }
                                });

                            if let Some(id) = delete_id {
                                if let Err(err) = db_operations::delete_reminder(&db.borrow(), id) {
                                    eprintln!("failed to delete reminder {id}: {err}");
                                }
                            }
                        }
                    }
                    Err(err) => {
                        ui.label("Błąd odczytu bazy");
                        eprintln!("failed to list reminders: {err}");
                    }
                }
            });
        });
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
                TrayCommand::Exit => self.exit_app(ctx)
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

        self.ui_main(ctx);
    }
}
