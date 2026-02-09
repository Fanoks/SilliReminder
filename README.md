# SilliReminder

Windows tray-first reminders app for simple, reliable due-date notifications.

- Stores data locally (SQLite)
- Runs in the system tray (near the clock)
- Sends Windows notifications when reminders become due
- Optional “Start with system” autostart
- UI languages: Polish / English

## For users

### Install
This repo contains two installer approaches:

- Standalone installer (recommended): see [installer/innosetup/README.md](installer/innosetup/README.md)
- Script installer (PowerShell): see [INSTALL.md](INSTALL.md)

### Instructions
- English: [docs/INSTRUCTIONS_EN.md](docs/INSTRUCTIONS_EN.md)
- Polish: [docs/INSTRUCTIONS_PL.md](docs/INSTRUCTIONS_PL.md)

### Where data is stored
Per-user (no admin required):

- DB (reminders): `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`
- Settings: `%LOCALAPPDATA%\SilliReminder\settings.sillisettings`

## For developers

### Build
```powershell
cargo build
```

### Run
```powershell
cargo run
```

### Release build
```powershell
cargo build --release
```

Output EXE:
- `target\release\SilliReminder.exe`

### Notes
- This project targets Windows (tray integration + notifications).

## License

MIT — see [LICENSE](LICENSE).
