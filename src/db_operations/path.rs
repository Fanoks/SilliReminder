use std::path::PathBuf;

pub(super) fn db_path() -> PathBuf {
    crate::paths::app_data_dir()
        .join("data")
        .join("silli_reminder.db")
}
