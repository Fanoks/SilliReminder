# SilliReminder — Instructions (English)

SilliReminder is a small Windows reminders app. It stores your reminders locally on your PC (SQLite database) and shows Windows notifications when a reminder becomes due.

## Quick start (3 steps)

1. **Open the app**
   - If you see a window with sections like **Settings**, **Add**, **Planned**: it’s running.
   - If you don’t see the window: look for the **SilliReminder icon in the system tray** (near the clock). Right‑click it → **Open**.
   - If you still don’t see the icon: click the **`^` (Show hidden icons)** arrow near the clock and look there.

2. **Add a reminder (the “Add” section)**
   - Click the date field on the left (e.g. `2026-02-12` with a small calendar icon) and choose the date.
   - Click the **“Note…”** field and type your reminder.
   - Click **“Add”** on the right.

3. **Wait for the notification**
   - When the reminder becomes due, Windows will show a notification.
   - If you don’t see it, check the Troubleshooting section below.

## How the app behaves (important)

### The app may run “in the tray”
- Closing the window may **hide it to the tray** instead of exiting.
- To open the window again: right‑click the tray icon → **Open**.
- To exit completely: right‑click the tray icon → **Exit**.

### Notifications
- When a reminder becomes due, the app triggers an OS-level notification.
- The app is designed to avoid spamming the same reminder repeatedly across restarts (it remembers what was already announced).

### Start with Windows (the “Settings” section)
- Enable the checkbox **“Start with system”** if you want SilliReminder to start with Windows.
- This uses per-user autostart (no admin required).

## Managing reminders

### See planned reminders (the “Planned” section)
- Reminders appear under **“Planned”**.
- The list format looks like: `YYYY-MM-DD - Your note`.

### Delete
- Each reminder row has a **red “X” button** on the right.
- Click **X** to delete that reminder.

## Where your data is stored

The app stores data per user (no admin rights needed):

- Database (your reminders): `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`
- Settings (preferences): `%LOCALAPPDATA%\SilliReminder\settings.sillisettings`

## Uninstall (clean removal)

During uninstall, you can choose:

- **Remove database (reminders)** — deletes your reminders.
- **Remove settings (preferences)** — resets app preferences.

Uninstall also removes the autostart registry entry (if it exists):
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\SilliReminder`

## Troubleshooting (common problems)

### “I installed it but I can’t see the window”
1. Check the system tray (near the clock).
2. Right‑click the SilliReminder tray icon → **Open**.
3. If there is no tray icon: click the **`^` (Show hidden icons)** arrow near the clock.
4. If you still can’t find it: start the app again from Start Menu.

### “No notifications appear”
1. Open **Windows Settings → System → Notifications**.
2. Make sure notifications are enabled globally.
3. Find **SilliReminder** in the app list and enable notifications.
4. Focus Assist / Do Not Disturb can suppress notifications — disable it temporarily.

### “Uninstall can’t remove the database/settings”
- The app may still be running in the tray and holding files open.
- Right‑click tray icon → **Exit**, then run uninstall again.

### “Windows SmartScreen warns about the app”
- This can happen for unsigned apps or new releases.
- Prefer downloading from the official link you trust.

## Tips for reliable reminders
- Keep your PC time correct (Windows time settings).
- Don’t put your PC into deep sleep if you need exact timing.

## Examples for accountants

Use the note field to make reminders unambiguous. Good examples:

- `VAT return — submit` (set date a few days before)
- `Payroll — send bank transfers`
- `Month end — close books`
- `Invoice deadline — client XYZ`

Tip: write the action first (Submit/Pay/Send/Close), then the subject.

## Backup (highly recommended)

Your reminders are stored only on this PC (local database). If you reinstall Windows or change computers, reminders will not automatically move.

To back up reminders, copy this file somewhere safe:

- `%LOCALAPPDATA%\SilliReminder\data\silli_reminder.db`

---
