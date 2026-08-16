use crate::domain::RepoError;
use crate::domain::ids::EntryId;
use crate::domain::journal::Entry;

/// Persistence operations for [`Entry`] aggregates.
pub trait EntryRepository {
    /// Stores a new entry.
    fn create(&self, entry: &Entry) -> Result<(), RepoError>;
    /// Loads the entry with the given id (`NotFound` if it does not exist).
    fn read(&self, entry_id: EntryId) -> Result<Entry, RepoError>;
    /// Persists changes to an existing entry.
    fn update(&self, entry: &Entry) -> Result<(), RepoError>;
    /// Removes the entry with the given id.
    fn delete(&self, entry_id: EntryId) -> Result<(), RepoError>;
}
