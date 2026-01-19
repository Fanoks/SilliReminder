use std::error::Error;

pub fn set_enabled(enabled: bool) -> Result<(), Box<dyn Error>> {
    if enabled {
        add_to_autostart()
    } else {
        remove_from_autostart()
    }
}

#[cfg(target_os = "windows")]
fn add_to_autostart() -> Result<(), Box<dyn Error>> {
    windows::add_to_autostart()
}

#[cfg(target_os = "windows")]
fn remove_from_autostart() -> Result<(), Box<dyn Error>> {
    windows::remove_from_autostart()
}

#[cfg(not(target_os = "windows"))]
fn add_to_autostart() -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn remove_from_autostart() -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(target_os = "windows")]
#[path = "autostart/windows.rs"]
mod windows;
