use chrono::{Datelike as _, NaiveDate, Weekday};
use eframe::egui::{
    self, Align, Area, Button, Color32, ComboBox, Frame, InnerResponse, Key, Layout, Order,
    RichText, Ui, Widget,
};
use std::ops::RangeInclusive;

use crate::i18n::{self, Language};

#[derive(Default, Clone)]
struct DatePickerPlState {
    picker_visible: bool,
    year: i32,
    month: u32,
    day: u32,
    setup: bool,
}

/// A small, self-contained date picker with Polish labels.
///
/// This exists because `egui_extras::DatePickerButton` (as of egui_extras 0.33.x)
/// has hardcoded English month/day labels and no locale hook (there is a TODO for locale).
pub struct DatePickerPlButton<'a> {
    selection: &'a mut NaiveDate,
    id_salt: Option<&'a str>,
    combo_boxes: bool,
    arrows: bool,
    format: String,
    show_icon: bool,
    start_end_years: Option<RangeInclusive<i32>>,
    calendar: bool,
    calendar_week: bool,
    highlight_weekends: bool,
    language: Language,
}

impl<'a> DatePickerPlButton<'a> {
    pub fn new(selection: &'a mut NaiveDate) -> Self {
        Self {
            selection,
            id_salt: None,
            combo_boxes: true,
            arrows: true,
            format: "%Y-%m-%d".to_owned(),
            show_icon: true,
            start_end_years: None,
            calendar: true,
            calendar_week: true,
            highlight_weekends: true,
            language: i18n::language(),
        }
    }

    /// Set picker language (defaults to detected app language).
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Must be set if you have multiple date pickers in the same `Ui`.
    pub fn id_salt(mut self, id_salt: &'a str) -> Self {
        self.id_salt = Some(id_salt);
        self
    }

    /// Show combo boxes (Rok/Miesiąc/Dzień) in the popup.
    #[allow(dead_code)]
    pub fn combo_boxes(mut self, combo_boxes: bool) -> Self {
        self.combo_boxes = combo_boxes;
        self
    }

    /// Show navigation arrows (<<< << < > >> >>>).
    #[allow(dead_code)]
    pub fn arrows(mut self, arrows: bool) -> Self {
        self.arrows = arrows;
        self
    }

    /// Change the format shown on the button.
    /// See `chrono::format::strftime` for valid formats.
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Show the calendar icon on the button.
    #[allow(dead_code)]
    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    /// Show the month calendar grid (Pn..Nd) in the popup.
    #[allow(dead_code)]
    pub fn calendar(mut self, calendar: bool) -> Self {
        self.calendar = calendar;
        self
    }

    /// Show ISO week numbers column (Tydz.) next to the calendar.
    #[allow(dead_code)]
    pub fn calendar_week(mut self, calendar_week: bool) -> Self {
        self.calendar_week = calendar_week;
        self
    }

    /// Highlight weekend days in the calendar grid.
    #[allow(dead_code)]
    pub fn highlight_weekends(mut self, highlight_weekends: bool) -> Self {
        self.highlight_weekends = highlight_weekends;
        self
    }

    /// Limit the selectable years.
    #[allow(dead_code)]
    pub fn start_end_years(mut self, start_end_years: RangeInclusive<i32>) -> Self {
        self.start_end_years = Some(start_end_years);
        self
    }
}

impl Widget for DatePickerPlButton<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let id = ui.make_persistent_id(self.id_salt);

        let today = chrono::offset::Utc::now().date_naive();

        let mut state = ui
            .data_mut(|data| data.get_persisted::<DatePickerPlState>(id))
            .unwrap_or_default();

        if !state.setup {
            state.year = self.selection.year();
            state.month = self.selection.month();
            state.day = self.selection.day();
            state.setup = true;
            ui.data_mut(|data| data.insert_persisted(id, state.clone()));
        }

        let mut text = if self.show_icon {
            RichText::new(format!("{} 📅", self.selection.format(&self.format)))
        } else {
            RichText::new(format!("{}", self.selection.format(&self.format)))
        };

        if state.picker_visible {
            let visuals = ui.visuals().widgets.open;
            text = text.color(visuals.text_color());
        }

        let mut button = Button::new(text);
        if state.picker_visible {
            let visuals = ui.visuals().widgets.open;
            button = button.fill(visuals.weak_bg_fill).stroke(visuals.bg_stroke);
        }

        let mut response = ui.add(button);

        if response.clicked() {
            state.picker_visible = true;
            ui.data_mut(|data| data.insert_persisted(id, state.clone()));
        }

        if state.picker_visible {
            let popup_width = 333.0;
            let mut pos = response.rect.left_bottom();

            let width_with_padding = popup_width
                + ui.style().spacing.item_spacing.x
                + ui.style().spacing.window_margin.leftf()
                + ui.style().spacing.window_margin.rightf();

            if pos.x + width_with_padding > ui.clip_rect().right() {
                pos.x = response.rect.right() - width_with_padding;
            }

            pos.x = pos.x.max(ui.style().spacing.window_margin.leftf());

            let InnerResponse {
                inner,
                response: area_response,
            } = Area::new(ui.make_persistent_id(self.id_salt))
                .kind(egui::UiKind::Picker)
                .order(Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    Frame::popup(ui.style())
                        .show(ui, |ui| {
                            ui.set_min_width(popup_width);
                            ui.set_max_width(popup_width);

                            // Match egui_extras feel: tighter spacing, never wrap.
                            ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                            let mut close = false;
                            let mut saved = false;

                            if self.combo_boxes {
                                ui.horizontal(|ui| {
                                    ui.label(i18n::date_picker_year(self.language));
                                    ComboBox::from_id_salt("date_picker_pl_year")
                                        .selected_text(state.year.to_string())
                                        .show_ui(ui, |ui| {
                                            let (start_year, end_year) = match &self.start_end_years
                                            {
                                                Some(range) => (*range.start(), *range.end()),
                                                None => (today.year() - 100, today.year() + 10),
                                            };

                                            for year in start_year..=end_year {
                                                if ui
                                                    .selectable_value(
                                                        &mut state.year,
                                                        year,
                                                        year.to_string(),
                                                    )
                                                    .changed()
                                                {
                                                    state.day = state.day.min(last_day_of_month(
                                                        state.year,
                                                        state.month,
                                                    ));
                                                }
                                            }
                                        });

                                    ComboBox::from_id_salt("date_picker_pl_month")
                                        .selected_text(i18n::date_picker_month_name(
                                            self.language,
                                            state.month,
                                        ))
                                        .show_ui(ui, |ui| {
                                            for month in 1..=12 {
                                                if ui
                                                    .selectable_value(
                                                        &mut state.month,
                                                        month,
                                                        i18n::date_picker_month_name(
                                                            self.language,
                                                            month,
                                                        ),
                                                    )
                                                    .changed()
                                                {
                                                    state.day = state.day.min(last_day_of_month(
                                                        state.year,
                                                        state.month,
                                                    ));
                                                }
                                            }
                                        });

                                    ComboBox::from_id_salt("date_picker_pl_day")
                                        .selected_text(state.day.to_string())
                                        .show_ui(ui, |ui| {
                                            let last = last_day_of_month(state.year, state.month);
                                            for day in 1..=last {
                                                ui.selectable_value(
                                                    &mut state.day,
                                                    day,
                                                    day.to_string(),
                                                );
                                            }
                                        });
                                });
                            }

                            if self.arrows {
                                ui.horizontal(|ui| {
                                    let arrow = |ui: &mut Ui, label: &str, hover: &str| -> bool {
                                        ui.add_sized(
                                            [34.0, 22.0],
                                            Button::new(RichText::new(label).strong()),
                                        )
                                        .on_hover_text(hover)
                                        .clicked()
                                    };

                                    if arrow(
                                        ui,
                                        "<<<",
                                        i18n::date_picker_hover_year_minus(self.language),
                                    ) {
                                        state.year -= 1;
                                        state.day = state
                                            .day
                                            .min(last_day_of_month(state.year, state.month));
                                    }
                                    if arrow(
                                        ui,
                                        "<<",
                                        i18n::date_picker_hover_month_minus(self.language),
                                    ) {
                                        state.month = state.month.saturating_sub(1);
                                        if state.month == 0 {
                                            state.month = 12;
                                            state.year -= 1;
                                        }
                                        state.day = state
                                            .day
                                            .min(last_day_of_month(state.year, state.month));
                                    }
                                    if arrow(
                                        ui,
                                        "<",
                                        i18n::date_picker_hover_day_minus(self.language),
                                    ) {
                                        state.day = state.day.saturating_sub(1);
                                        if state.day == 0 {
                                            state.month = state.month.saturating_sub(1);
                                            if state.month == 0 {
                                                state.month = 12;
                                                state.year -= 1;
                                            }
                                            state.day = last_day_of_month(state.year, state.month);
                                        }
                                    }
                                    if arrow(
                                        ui,
                                        ">",
                                        i18n::date_picker_hover_day_plus(self.language),
                                    ) {
                                        state.day += 1;
                                        if state.day > last_day_of_month(state.year, state.month) {
                                            state.day = 1;
                                            state.month += 1;
                                            if state.month > 12 {
                                                state.month = 1;
                                                state.year += 1;
                                            }
                                        }
                                    }
                                    if arrow(
                                        ui,
                                        ">>",
                                        i18n::date_picker_hover_month_plus(self.language),
                                    ) {
                                        state.month += 1;
                                        if state.month > 12 {
                                            state.month = 1;
                                            state.year += 1;
                                        }
                                        state.day = state
                                            .day
                                            .min(last_day_of_month(state.year, state.month));
                                    }
                                    if arrow(
                                        ui,
                                        ">>>",
                                        i18n::date_picker_hover_year_plus(self.language),
                                    ) {
                                        state.year += 1;
                                        state.day = state
                                            .day
                                            .min(last_day_of_month(state.year, state.month));
                                    }
                                });
                            }

                            if self.calendar {
                                ui.add_space(4.0);

                                let weeks = month_weeks_monday_start(state.year, state.month);

                                ui.push_id("date_picker_pl_calendar", |ui| {
                                    let columns = if self.calendar_week { 8 } else { 7 };
                                    egui::Grid::new("grid")
                                        .num_columns(columns)
                                        .spacing([2.0, 2.0])
                                        .show(ui, |ui| {
                                            if self.calendar_week {
                                                ui.label(
                                                    RichText::new(i18n::date_picker_week(
                                                        self.language,
                                                    ))
                                                    .strong(),
                                                );
                                            }

                                            for name in i18n::date_picker_weekdays(self.language) {
                                                ui.label(RichText::new(name).strong());
                                            }
                                            ui.end_row();

                                            for week in weeks {
                                                if self.calendar_week {
                                                    let week_no = week
                                                        .first()
                                                        .map(|d| d.iso_week().week())
                                                        .unwrap_or(0);
                                                    ui.label(week_no.to_string());
                                                }

                                                for day in week {
                                                    let in_month = day.month() == state.month;
                                                    let is_weekend = day.weekday() == Weekday::Sat
                                                        || day.weekday() == Weekday::Sun;
                                                    let is_selected = day.year() == state.year
                                                        && day.month() == state.month
                                                        && day.day() == state.day;

                                                    let mut text_color =
                                                        ui.visuals().widgets.inactive.text_color();
                                                    if !in_month {
                                                        text_color =
                                                            text_color.linear_multiply(0.5);
                                                    }
                                                    if self.highlight_weekends && is_weekend {
                                                        text_color = if ui.visuals().dark_mode {
                                                            Color32::DARK_RED
                                                        } else {
                                                            Color32::LIGHT_RED
                                                        };
                                                    }

                                                    let fill_color = if is_selected {
                                                        ui.visuals().selection.bg_fill
                                                    } else {
                                                        ui.visuals().extreme_bg_color
                                                    };

                                                    let button_response = ui.add(
                                                        Button::new(
                                                            RichText::new(day.day().to_string())
                                                                .color(text_color),
                                                        )
                                                        .fill(fill_color),
                                                    );

                                                    if day == today {
                                                        // Encircle today's date (like egui_extras).
                                                        let stroke =
                                                            ui.visuals().widgets.inactive.fg_stroke;
                                                        ui.painter().circle_stroke(
                                                            button_response.rect.center(),
                                                            8.0,
                                                            stroke,
                                                        );
                                                    }

                                                    if button_response.clicked() {
                                                        state.year = day.year();
                                                        state.month = day.month();
                                                        state.day = day.day();
                                                    }
                                                }
                                                ui.end_row();
                                            }
                                        });
                                });

                                ui.add_space(4.0);
                            }

                            ui.columns(3, |cols| {
                                cols[0].allocate_space(egui::Vec2::ZERO);

                                cols[1].with_layout(Layout::top_down(Align::Center), |ui| {
                                    if ui
                                        .add_sized(
                                            [80.0, 24.0],
                                            Button::new(i18n::date_picker_cancel(self.language)),
                                        )
                                        .clicked()
                                    {
                                        close = true;
                                    }
                                });

                                cols[2].with_layout(Layout::top_down(Align::Center), |ui| {
                                    if ui
                                        .add_sized(
                                            [80.0, 24.0],
                                            Button::new(i18n::date_picker_save(self.language)),
                                        )
                                        .clicked()
                                    {
                                        *self.selection = NaiveDate::from_ymd_opt(
                                            state.year,
                                            state.month,
                                            state.day,
                                        )
                                        .expect("invalid date");
                                        saved = true;
                                        close = true;
                                    }
                                });
                            });

                            if close {
                                state.setup = false;
                                state.picker_visible = false;
                            }

                            // Persist state after UI interactions.
                            ui.data_mut(|data| data.insert_persisted(id, state.clone()));

                            saved && close
                        })
                        .inner
                });

            if inner {
                response.mark_changed();
            }

            let any_popup_open = ui.ctx().is_popup_open();
            if !response.clicked()
                && !any_popup_open
                && (ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere())
            {
                state.picker_visible = false;
                ui.data_mut(|data| data.insert_persisted(id, state));
            }
        }

        response
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let date = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid year/month");
    date.with_day(31)
        .map(|_| 31)
        .or_else(|| date.with_day(30).map(|_| 30))
        .or_else(|| date.with_day(29).map(|_| 29))
        .unwrap_or(28)
}

fn month_weeks_monday_start(year: i32, month: u32) -> Vec<Vec<NaiveDate>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid year/month");
    let last = NaiveDate::from_ymd_opt(year, month, last_day_of_month(year, month))
        .expect("invalid year/month");

    let mut start = first;
    while start.weekday() != Weekday::Mon {
        start = start.pred_opt().expect("date underflow");
    }

    let mut weeks: Vec<Vec<NaiveDate>> = Vec::new();
    loop {
        let mut week: Vec<NaiveDate> = Vec::with_capacity(7);
        for _ in 0..7 {
            week.push(start);
            start = start.succ_opt().expect("date overflow");
        }
        weeks.push(week);

        // `start` is now the next Monday.
        if start > last && start.month() != month {
            break;
        }
    }

    weeks
}
