use rusqlite::Connection;

pub(super) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("PRAGMA foreign_keys = ON;", ())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Reminder`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `date` TEXT NOT NULL,
            `note` TEXT NOT NULL,
            `notified_level` INTEGER NOT NULL DEFAULT 0
        );",
        (),
    )?;

    // Migration for older DBs.
    let mut stmt = conn.prepare("PRAGMA table_info(`Reminder`);")?;
    let cols = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let mut has_notified_level = false;
    for c in cols {
        if c? == "notified_level" {
            has_notified_level = true;
            break;
        }
    }
    if !has_notified_level {
        conn.execute(
            "ALTER TABLE `Reminder` ADD COLUMN `notified_level` INTEGER NOT NULL DEFAULT 0;",
            (),
        )?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS `idx_reminder_date` ON `Reminder`(`date`);",
        (),
    )?;

    Ok(())
}
