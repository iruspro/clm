//! **Schema and bootstrap.**
//!
//! Owns the SQL side of the app in one place: the table/column names as
//! [`sea_query`] identifiers ([`idens`]), the migration scripts, and the two
//! functions that apply them.
//!
//! Both functions take an open [`Connection`] rather than a path, so the caller
//! decides where the database lives — a file for the binary, `:memory:` for
//! tests.

pub mod idens;
#[cfg(feature = "testing")]
pub mod testing;

use rusqlite::Connection;

/// Migrations in application order — the single source of truth for the schema,
/// shared by the runtime bootstrap and by tests.
pub const MIGRATIONS: &[&str] = &[include_str!("sql/migrations/01-create-schema.sql")];

/// Teardown script: drops every table, children before parents.
const TEARDOWN: &str = include_str!("sql/reset.sql");

/// Applies every migration to `conn`, preserving existing data.
///
/// # Errors
/// Returns the underlying [`rusqlite::Error`] if any script fails to execute.
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    for migration in MIGRATIONS {
        conn.execute_batch(migration)?;
    }

    Ok(())
}

/// Recreates the schema from scratch: drops every table, then migrates.
///
/// **Destructive** — this discards all existing data, so it is for development
/// only.
///
/// # Errors
/// Same as [`migrate`], plus any failure of the teardown script.
pub fn reset(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(TEARDOWN)?;

    migrate(conn)
}
