use chrono::{NaiveDate};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: i64,
    pub date: NaiveDate,
    pub note: String
}

pub(in crate::db_operations) fn parse_db_date(date_str: &str) -> rusqlite::Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
