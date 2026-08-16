//! Throwaway databases carrying the real schema, for tests.

use rusqlite::Connection;

/// An empty in-memory database with every migration applied.
///
/// # Panics
/// If the database cannot be opened or the schema fails to apply.
pub fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory database");
    crate::db::migrate(&conn).expect("apply migrations");

    conn
}
