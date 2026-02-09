use rusqlite::Connection;

pub(super) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("PRAGMA foreign_keys = ON;", ())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Reminder`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `date` TEXT NOT NULL,
            `note` TEXT NOT NULL
        );",
        (),
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS `idx_reminder_date` ON `Reminder`(`date`);",
        (),
    )?;

    Ok(())
}
