use std::error::Error;

use rusqlite::Connection;

use super::{parse_db_date, Reminder};

pub fn list_reminders(conn: &Connection) -> Result<Vec<Reminder>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT `id`, `date`, `note`, `notified_level`
         FROM `Reminder`
         ORDER BY `date` ASC, `id` ASC;",
    )?;

    let iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let date_str: String = row.get(1)?;
        let note: String = row.get(2)?;
        let notified_level: i64 = row.get(3)?;

        Ok(Reminder {
            id,
            date: parse_db_date(&date_str)?,
            note,
            notified_level: notified_level.clamp(0, 3) as u8,
        })
    })?;

    Ok(iter.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[allow(dead_code)]
pub fn get_reminder(conn: &Connection, id: i64) -> Result<Option<Reminder>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT `id`, `date`, `note`, `notified_level`
         FROM `Reminder`
         WHERE `id` = ?1;",
    )?;

    let mut rows = stmt.query([id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };

    let id: i64 = row.get(0)?;
    let date_str: String = row.get(1)?;
    let note: String = row.get(2)?;
    let notified_level: i64 = row.get(3)?;

    Ok(Some(Reminder {
        id,
        date: parse_db_date(&date_str)?,
        note,
        notified_level: notified_level.clamp(0, 3) as u8,
    }))
}
