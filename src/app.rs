use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc};

use chrono::{Local, NaiveDate};
use eframe::egui::{self, RichText};
use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
use rusqlite::Connection;

use crate::{debug_err, debug_log};
use crate::{autostart, db_operations, settings, tray::TrayCommand};
use crate::tray::{TrayNotificationKind};
use crate::i18n::{self, Language};

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
    lang: Language,
    system_start: bool,
    background: bool,
    allow_close: bool,
    ignore_close_frames: u8,
    hwnd_set: bool,
    tray_rx: mpsc::Receiver<TrayCommand>,
    selected_date: NaiveDate,
    note_input: String,
    db: Option<Rc<RefCell<Connection>>>,

    notifications: VecDeque<BoundaryNotification>,
    next_boundary_check: Instant,
}

#[derive(Debug, Clone)]
struct BoundaryNotification {
    date: NaiveDate,
    note: String,
    level: u8,
}

impl SilliReminder {
    pub fn new(system_start: bool, background: bool, tray_rx: mpsc::Receiver<TrayCommand>) -> Self {
        let db = match db_operations::get_db() {
            Ok(db) => Some(db),
            Err(err) => {
                debug_err!("failed to open database: {err}");
                None
            }
        };

        Self {
            lang: i18n::language(),
            system_start,
            background,
            allow_close: false,
            ignore_close_frames: 0,
            hwnd_set: false,
            tray_rx,
            selected_date: Local::now().date_naive(),
            note_input: String::new(),
            db,

            notifications: VecDeque::new(),
            next_boundary_check: Instant::now(),
        }
    }

    fn urgency_level(today: NaiveDate, date: NaiveDate) -> u8 {
        let days_until = (date - today).num_days();
        if days_until <= 1 {
            3
        } else if days_until <= 3 {
            2
        } else if days_until <= 7 {
            1
        } else {
            0
        }
    }

    fn maybe_check_boundary_notifications(&mut self) {
        let now = Instant::now();
        if now < self.next_boundary_check {
            return;
        }

        self.next_boundary_check = now
            + if self.background {
                Duration::from_secs(60)
            } else {
                Duration::from_secs(10)
            };

        let Some(db) = &self.db else {
            return;
        };

        let today = Local::now().date_naive();
        let reminders = match db_operations::list_reminders(&db.borrow()) {
            Ok(r) => r,
            Err(err) => {
                debug_err!("failed to list reminders for notifications: {err}");
                return;
            }
        };

        for r in reminders {
            let current_level = Self::urgency_level(today, r.date);
            let previous_level = r.notified_level.min(3);

            if current_level <= previous_level {
                continue;
            }

            // Queue *each* boundary crossed so we don't skip 7->3->1 transitions
            // even if the app was closed for a while.
            for level in (previous_level + 1)..=current_level {
                if level == 0 {
                    continue;
                }
                self.notifications.push_back(BoundaryNotification {
                    date: r.date,
                    note: r.note.clone(),
                    level,
                });
            }

            if let Err(err) = db_operations::set_reminder_notified_level(&db.borrow(), r.id, current_level) {
                debug_err!("failed to persist notified_level for {}: {err}", r.id);
            }
        }
    }

    fn dispatch_notifications_to_tray(&mut self) {
        while let Some(n) = self.notifications.pop_front() {
            let kind = match n.level {
                1 => TrayNotificationKind::Info,
                2 => TrayNotificationKind::Warning,
                _ => TrayNotificationKind::Error,
            };

            let title = i18n::notif_title(self.lang, n.level);
            let body = format!(
                "{}\n{}: {}",
                n.note,
                i18n::notif_date_label(self.lang),
                n.date
            );
            crate::tray::notify(&title, &body, kind);
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
            RichText::new(i18n::app_header(self.lang))
                .size(40.0)
                .strong()
                .color(accent),
        );
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(
            RichText::new(i18n::ui_settings(self.lang))
                .size(25.0)
                .strong()
                .color(accent),
        );
        ui.group(|ui| {
            let response = ui.checkbox(&mut self.system_start, i18n::ui_start_with_system(self.lang));

            if response.changed() {
                debug_log!("system_start toggled -> {}", self.system_start);

                if let Err(err) = autostart::set_enabled(self.system_start) {
                    debug_err!("failed to update autostart: {err}");
                }

                if let Err(err) = settings::save_setting(self.system_start) {
                    debug_err!("failed to save setting: {err}");
                }
            }
        });
    }

    fn ui_sections(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(
            RichText::new(i18n::ui_add(self.lang))
                .size(25.0)
                .strong()
                .color(accent),
        );
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
                        .format("%Y-%m-%d")
                        .language(self.lang),
                );
                date_changed = date_response.changed();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    add_clicked = ui
                        .add_sized(
                            egui::vec2(70.0, row_h),
                            egui::Button::new(i18n::ui_add_button(self.lang)),
                        )
                        .clicked();

                    let note_response: egui::Response = ui.add_sized(
                        egui::vec2(ui.available_width(), row_h),
                        egui::TextEdit::singleline(&mut self.note_input)
                            .id_salt("note_input")
                            .hint_text(i18n::ui_note_hint(self.lang)),
                    );
                    note_changed = note_response.changed();
                });
            });

            if add_clicked {
                if let Some(db) = &self.db {
                    let note = self.note_input.trim();
                    if note.is_empty() {
                            debug_err!("note is empty; nothing inserted");
                    } else {
                        match db_operations::insert_reminder(&db.borrow(), self.selected_date, note) {
                            Ok(id) => {
                                    debug_log!("Dodano #{id}: {}, {}", self.selected_date, note);
                                self.note_input.clear();
                            }
                                Err(err) => debug_err!("failed to insert reminder: {err}"),
                        }
                    }
                } else {
                        debug_err!("database not available");
                }
            }

            if date_changed {
                debug_log!("Selected date -> {}", self.selected_date);
            }

            if note_changed {
                debug_log!("Note -> {}", self.note_input);
            }
        });
    }
    
    fn ui_planed(&mut self, ui: &mut egui::Ui) {
        let accent = ui.visuals().hyperlink_color;
        ui.label(
            RichText::new(i18n::ui_planned(self.lang))
                .size(28.0)
                .strong()
                .color(accent),
        );
        ui.group(|ui| {
            ui.set_min_size(ui.available_size());

            ui.vertical(|ui| {
                let Some(db) = &self.db else {
                    ui.label(i18n::ui_no_db(self.lang));
                    return;
                };

                match db_operations::list_reminders(&db.borrow()) {
                    Ok(reminders) => {
                        if reminders.is_empty() {
                            ui.label(i18n::ui_empty(self.lang));
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

                                                        let today = Local::now().date_naive();
                                                        let days_until = (r.date - today).num_days();
                                                        let reminder_color = if days_until > 7 {
                                                            ui.visuals().text_color()
                                                        } else if days_until > 3 {
                                                            ui.visuals().hyperlink_color
                                                        } else if days_until > 1 {
                                                            ui.visuals().warn_fg_color
                                                        } else {
                                                            ui.visuals().error_fg_color
                                                        };

                                                        let row_text = RichText::new(format!(
                                                            "{}  -  {}",
                                                            r.date, r.note
                                                        ))
                                                        .size(text_size)
                                                        .color(reminder_color);
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
                                    debug_err!("failed to delete reminder {id}: {err}");
                                }
                            }
                        }
                    }
                    Err(err) => {
                        ui.label(i18n::ui_db_read_error(self.lang));
                        debug_err!("failed to list reminders: {err}");
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

        // Boundary notifications (DB-backed): check periodically even in background.
        self.maybe_check_boundary_notifications();
        self.dispatch_notifications_to_tray();

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
