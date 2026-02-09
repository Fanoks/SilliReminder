# Standalone installer (setup.exe) via Inno Setup

This produces a single `SilliReminder-Setup.exe` that **downloads** your app EXE from GitHub Releases during install.

## Prerequisites

1. Install **Inno Setup 6**.
2. Install the **Inno Download Plugin (IDP)**.

## Configure

Edit:

- [installer/innosetup/SilliReminder.iss](installer/innosetup/SilliReminder.iss)

Set:

- `#define MyAppUrl "..."`

And set:

- `#define MyAppSha256 "..."`

Compute SHA-256 for your release EXE (recommended: compute from the exact file you upload to GitHub Releases):

```powershell
(Get-FileHash .\SilliReminder.exe -Algorithm SHA256).Hash
```

Use the direct browser-download URL to your GitHub release asset `SilliReminder.exe`.

## Build

### 1) Install tools

- Install **Inno Setup 6**.
- Install **Inno Download Plugin (IDP)**.

IDP must be installed so that `#include <idp.iss>` works when compiling.

If IDP doesn’t have an installer on your machine (zip-only), install it manually:

1. Find your Inno Setup install directory, typically:
	- `C:\Program Files (x86)\Inno Setup 6\`
2. Copy `idp.iss` into:
	- `C:\Program Files (x86)\Inno Setup 6\ISPP\Include\`
3. Copy the plugin DLL(s) (commonly `idp.dll`) into:
	- `C:\Program Files (x86)\Inno Setup 6\Plugins\`

After this, compiling a script containing `#include <idp.iss>` should work.

### 2) Compile (GUI)

1. Open Inno Setup.
2. Open `installer/innosetup/SilliReminder.iss`.
3. Click **Build → Compile**.

### 3) Compile (CLI)

Or from CLI (if `ISCC.exe` is in PATH):

```powershell
ISCC installer\innosetup\SilliReminder.iss
```

## Install location

Per-user install (no admin):

- `{localappdata}\Programs\SilliReminder\SilliReminder.exe`

User data (DB + settings) remains here (handled by the app itself):

- `%LOCALAPPDATA%\SilliReminder\...`

## Install wizard options

The installer shows these checkboxes:

- Create a Desktop shortcut
- Download extra instructions (option only for now)
- Open SilliReminder after closing the wizard

## Uninstall behavior

The uninstall wizard shows 2 optional checkboxes:

- Remove database (reminders)
- Remove settings (preferences)

Autostart registry cleanup is always performed on uninstall:

- Deletes `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\SilliReminder` (if present)

Current data file locations used by the app:

- DB: `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`
- Settings: `%LOCALAPPDATA%\SilliReminder\settings.sillisettings`
