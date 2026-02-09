use std::fs::create_dir_all;
use std::path::PathBuf;

pub(super) fn db_path() -> PathBuf {
    // Keep it simple and portable: store next to the executable.
    // This avoids needing extra deps like `dirs` and works well for a small tray app.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let dir = exe_dir.join("data");
    if let Err(e) = create_dir_all(&dir) {
        eprintln!("Couldn't create DB directory: {e}");
    }

    dir.join("silli_reminder.db")
}
