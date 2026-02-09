use std::path::PathBuf;

/// Returns a writable, per-user directory for app data.
///
/// Windows: `%LOCALAPPDATA%\SilliReminder`
/// Fallback (non-Windows / missing env): directory of the executable.
pub fn app_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(base) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(base).join("SilliReminder");
        }
    }

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}
