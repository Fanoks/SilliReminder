use std::error::Error;

use chrono::NaiveDate;
use rusqlite::{Connection, params};

pub fn insert_reminder(
    conn: &Connection,
    date: NaiveDate,
    note: &str,
) -> Result<i64, Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `Reminder` (`date`, `note`) VALUES (?1, ?2);",
        params![date.format("%Y-%m-%d").to_string(), note],
    )?;
    Ok(conn.last_insert_rowid())
}
