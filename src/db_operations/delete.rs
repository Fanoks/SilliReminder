use std::error::Error;

use rusqlite::Connection;

pub fn delete_reminder(conn: &Connection, id: i64) -> Result<(), Box<dyn Error>> {
    conn.execute("DELETE FROM `Reminder` WHERE `id` = ?1;", (&id,))?;
    Ok(())
}
