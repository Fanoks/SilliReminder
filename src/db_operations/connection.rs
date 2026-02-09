use std::{cell::RefCell, rc::Rc};

use rusqlite::Connection;

use super::{path, schema};

pub fn get_db() -> rusqlite::Result<Rc<RefCell<Connection>>> {
    let db_path = path::db_path();
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let conn = Connection::open(db_path)?;
    schema::ensure_schema(&conn)?;
    Ok(Rc::new(RefCell::new(conn)))
}
