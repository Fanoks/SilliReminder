# SilliReminder installation (Windows)

This project ships a simple per-user installer script that downloads the EXE from GitHub and installs it under:

- `%LOCALAPPDATA%\\Programs\\SilliReminder\\SilliReminder.exe`

App data (DB + settings) is stored separately under:

- `%LOCALAPPDATA%\\SilliReminder\\...`

## Install

1. Build and publish a release EXE on GitHub.
2. Copy the direct download URL to the release asset (the `.exe`).
3. Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\\install.ps1 -Url "<YOUR_URL_HERE>"
```

## Update / overwrite

```powershell
powershell -ExecutionPolicy Bypass -File scripts\\install.ps1 -Url "<YOUR_URL_HERE>" -Force
```

## Uninstall

```powershell
powershell -ExecutionPolicy Bypass -File scripts\\uninstall.ps1
```
