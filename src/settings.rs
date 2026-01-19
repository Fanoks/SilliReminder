use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

fn settings_path() -> PathBuf {
    // Store settings next to the executable so autostart (different CWD) still works.
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("settings.sillisettings")))
        .unwrap_or_else(|| PathBuf::from("settings.sillisettings"))
}

pub fn load_setting() -> std::io::Result<bool> {
    let path = settings_path();

    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(path)?;
    let value = match content.trim() {
        "1" | "true" | "True" | "TRUE" => true,
        _ => false,
    };
    Ok(value)
}

pub fn save_setting(system_start: bool) -> std::io::Result<()> {
    let path = settings_path();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    let value = if system_start { b"1" } else { b"0" };
    file.write_all(value)?;
    Ok(())
}
