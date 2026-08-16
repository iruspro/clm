use color_eyre::Result;
use rusqlite::Connection;

/// Everything a command's task is allowed to reach for.
pub struct Ctx {
    /// The open ledger. Borrow it to build a repository or run a view.
    pub db: Connection,
}

impl Ctx {
    /// Opens the database at `path`, creating it if needed, and brings its
    /// schema up to date.
    pub fn open(path: &str) -> Result<Self> {
        let db = Connection::open(path)?;
        crate::db::migrate(&db)?;

        Ok(Self { db })
    }
}
