//! Service: record a new journal entry in the ledger.

use thiserror::Error;
use time::Date;

use crate::domain::journal::{BalancedPostings, Entry, EntryRepository};
use crate::domain::{EntryId, RepoError};

/// Input for [`AddJournalEntryService`].
pub struct AddJournalEntryCommand {
    pub date: Date,
    pub description: String,
    pub postings: BalancedPostings,
}

/// Builds an [`Entry`] with a freshly generated id and stores it, returning
/// that id.
pub struct AddJournalEntryService<ER> {
    entries: ER,
}

impl<ER> AddJournalEntryService<ER>
where
    ER: EntryRepository,
{
    pub fn new(entries: ER) -> Self {
        AddJournalEntryService { entries }
    }

    pub fn execute(&self, cmd: AddJournalEntryCommand) -> Result<EntryId, ServiceError> {
        let id = EntryId::new();

        let entry = Entry::new(id, cmd.date, cmd.description, cmd.postings);
        self.entries.create(&entry)?;

        Ok(id)
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
