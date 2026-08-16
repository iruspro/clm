//! Service: strike a journal entry from the ledger.

use thiserror::Error;

use crate::domain::journal::EntryRepository;
use crate::domain::{EntryId, RepoError};

/// Input for [`Service`].
#[derive(Debug)]
pub struct Command {
    pub id: EntryId,
}

/// Removes an entry and, with it, every posting it made.
///
/// A ledger is conventionally corrected by a reversing entry rather than by
/// deletion, which keeps the history intact. This is the blunter operation, for
/// an entry that should never have been recorded at all.
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

    pub fn execute(&self, cmd: Command) -> Result<(), ServiceError> {
        self.entries.delete(cmd.id).map_err(|err| match err {
            RepoError::NotFound => ServiceError::NotFound,
            err => ServiceError::Repo(err),
        })
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
