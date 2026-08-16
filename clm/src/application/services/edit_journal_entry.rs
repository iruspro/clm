//! Service: correct a journal entry — its date, its description, its postings.

use thiserror::Error;
use time::Date;

use crate::domain::journal::{BalancedPostings, EntryRepository};
use crate::domain::{EntryId, RepoError};

/// Input for [`Service`].
///
/// The postings replace the entry's own wholesale: a posting has no identity
/// apart from the entry it belongs to, so there is nothing to edit in place.
pub struct Command {
    pub id: EntryId,
    pub date: Date,
    pub description: String,
    pub postings: BalancedPostings,
}

/// Loads an entry, applies the command to it, and stores it again.
pub struct Service<ER> {
    entries: ER,
}

impl<ER> Service<ER>
where
    ER: EntryRepository,
{
    pub fn new(entries: ER) -> Self {
        Service { entries }
    }

    /// Read, modify, write — so an edit of an entry that is not there fails
    /// instead of recording a new one under the given id.
    pub fn execute(&self, cmd: Command) -> Result<(), ServiceError> {
        let mut entry = self.entries.read(cmd.id).map_err(|err| match err {
            RepoError::NotFound => ServiceError::NotFound,
            err => ServiceError::Repo(err),
        })?;

        entry.set_date(cmd.date);
        entry.set_description(cmd.description);
        entry.set_postings(cmd.postings);

        self.entries.update(&entry)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// No entry is stored under the id the command names.
    #[error("no entry with that id")]
    NotFound,
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
