use std::error::Error;

use rusqlite::{Connection, params};

pub fn set_reminder_notified_level(
    conn: &Connection,
    id: i64,
    notified_level: u8,
) -> Result<(), Box<dyn Error>> {
    let notified_level: i64 = (notified_level.min(3)) as i64;
    conn.execute(
        "UPDATE `Reminder` SET `notified_level` = ?1 WHERE `id` = ?2;",
        params![notified_level, id],
    )?;
    Ok(())
}
