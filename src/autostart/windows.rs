use std::error::Error;

pub(super) fn add_to_autostart() -> Result<(), Box<dyn Error>> {
    /// Enable autostart for the current user.
    ///
    /// Implementation:
    /// - Writes to: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
    /// - Value name: `SilliReminder`
    /// - Value data: `"<path-to-exe>" --autostart`
    ///
    /// Notes:
    /// - The exe path is quoted to handle spaces.
    /// - We include `--autostart` so the app can start minimized/background.
    use std::env;
    use winreg::RegKey;
    use winreg::enums::*;

    let exe_path = env::current_exe()?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    )?;

    // Use --autostart so we can detect stale entries and keep UI hidden.
    let command = format!("\"{}\" --autostart", exe_path.display());
    run_key.set_value("SilliReminder", &command)?;

    Ok(())
}

pub(super) fn remove_from_autostart() -> Result<(), Box<dyn Error>> {
    use winreg::RegKey;
    /// Disable autostart for the current user.
    ///
    /// Deleting a missing value is treated as success (idempotent operation).
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    )?;

    match run_key.delete_value("SilliReminder") {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(Box::new(err)),
    }

    Ok(())
}
