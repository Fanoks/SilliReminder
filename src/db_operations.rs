//! Database access layer.
//!
//! Split overview:
//! - `types.rs`: DB-facing domain types (`Reminder`) + parsing helpers
//! - `schema.rs`: schema creation/migrations
//! - `queries.rs`: read/query helpers
//! - `insert.rs`, `update.rs`, `delete.rs`: write helpers
//! - `path.rs`: DB file location
//! - `connection.rs`: open connection + ensure schema

mod connection;
mod delete;
mod insert;
mod path;
mod queries;
mod schema;
mod types;

pub use connection::get_db;
pub use delete::delete_reminder;
pub use insert::insert_reminder;
pub use queries::list_reminders;
pub use types::Reminder;
// More helpers exist in submodules (delete/update/get) when needed.

// Internal-only items shared across db submodules.
pub(in crate::db_operations) use types::{parse_db_date};
