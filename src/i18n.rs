use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Pl,
    En,
}

static LANG: OnceLock<Language> = OnceLock::new();

pub fn init() {
    let _ = LANG.set(detect_language());
}

pub fn language() -> Language {
    *LANG.get_or_init(detect_language)
}

fn detect_language() -> Language {
    let locale = sys_locale::get_locale().unwrap_or_default();
    let locale = locale.to_ascii_lowercase();

    if locale.starts_with("pl") {
        Language::Pl
    } else {
        Language::En
    }
}

pub fn app_title(_lang: Language) -> &'static str {
    "SilliReminder"
}

pub fn app_header(_lang: Language) -> &'static str {
    "SilliReminder"
}

pub fn ui_settings(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Ustawienia",
        Language::En => "Settings",
    }
}

pub fn ui_start_with_system(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Włącz podczas włączania systemu",
        Language::En => "Start with system",
    }
}

pub fn ui_add(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Dodaj",
        Language::En => "Add",
    }
}

pub fn ui_add_button(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Dodaj",
        Language::En => "Add",
    }
}

pub fn ui_note_hint(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Notatka...",
        Language::En => "Note...",
    }
}

pub fn ui_planned(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Zaplanowane",
        Language::En => "Planned",
    }
}

pub fn ui_no_db(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Brak bazy danych",
        Language::En => "Database unavailable",
    }
}

pub fn ui_empty(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "(pusto)",
        Language::En => "(empty)",
    }
}

pub fn ui_db_read_error(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Błąd odczytu bazy",
        Language::En => "Failed to read database",
    }
}

pub fn tray_tooltip(_lang: Language) -> &'static str {
    "SilliReminder"
}

pub fn tray_open(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Otwórz",
        Language::En => "Open",
    }
}

pub fn tray_exit(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Zamknij",
        Language::En => "Exit",
    }
}

pub fn notif_prefix(lang: Language, level: u8) -> &'static str {
    match (lang, level) {
        (Language::Pl, 1) => "≤ 7 dni",
        (Language::Pl, 2) => "≤ 3 dni",
        (Language::Pl, _) => "≤ 1 dzień",
        (Language::En, 1) => "≤ 7 days",
        (Language::En, 2) => "≤ 3 days",
        (Language::En, _) => "≤ 1 day",
    }
}

pub fn notif_title(lang: Language, level: u8) -> String {
    match lang {
        Language::Pl => format!("Przypomnienie ({})", notif_prefix(lang, level)),
        Language::En => format!("Reminder ({})", notif_prefix(lang, level)),
    }
}

pub fn notif_date_label(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Data",
        Language::En => "Date",
    }
}

pub fn date_picker_year(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Rok:",
        Language::En => "Year:",
    }
}

pub fn date_picker_week(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Tydz.",
        Language::En => "Wk",
    }
}

pub fn date_picker_weekdays(lang: Language) -> [&'static str; 7] {
    match lang {
        Language::Pl => ["Pn", "Wt", "Śr", "Cz", "Pt", "So", "Nd"],
        Language::En => ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    }
}

pub fn date_picker_cancel(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Anuluj",
        Language::En => "Cancel",
    }
}

pub fn date_picker_save(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "Zapisz",
        Language::En => "Save",
    }
}

pub fn date_picker_month_name(lang: Language, month: u32) -> &'static str {
    match lang {
        Language::Pl => match month {
            1 => "Styczeń",
            2 => "Luty",
            3 => "Marzec",
            4 => "Kwiecień",
            5 => "Maj",
            6 => "Czerwiec",
            7 => "Lipiec",
            8 => "Sierpień",
            9 => "Wrzesień",
            10 => "Październik",
            11 => "Listopad",
            12 => "Grudzień",
            _ => "?",
        },
        Language::En => match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "?",
        },
    }
}

pub fn date_picker_hover_year_minus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "odejmij 1 rok",
        Language::En => "subtract 1 year",
    }
}

pub fn date_picker_hover_month_minus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "odejmij 1 miesiąc",
        Language::En => "subtract 1 month",
    }
}

pub fn date_picker_hover_day_minus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "odejmij 1 dzień",
        Language::En => "subtract 1 day",
    }
}

pub fn date_picker_hover_day_plus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "dodaj 1 dzień",
        Language::En => "add 1 day",
    }
}

pub fn date_picker_hover_month_plus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "dodaj 1 miesiąc",
        Language::En => "add 1 month",
    }
}

pub fn date_picker_hover_year_plus(lang: Language) -> &'static str {
    match lang {
        Language::Pl => "dodaj 1 rok",
        Language::En => "add 1 year",
    }
}
