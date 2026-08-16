use crate::domain::account_group::AccountGroup;
use crate::domain::{AccountGroupId, RepoError};

/// Persistence operations for [`AccountGroup`] aggregates.
pub trait AccountGroupRepository {
    /// Stores a new group.
    fn create(&self, group: &AccountGroup) -> Result<(), RepoError>;
    /// Loads the group with the given id (`NotFound` if it does not exist).
    fn read(&self, id: AccountGroupId) -> Result<AccountGroup, RepoError>;
    /// Persists changes to an existing group, matched by its id — including a
    /// renamed [`Name`](crate::domain::Name) (`NotFound` if no such group is stored).
    fn update(&self, group: &AccountGroup) -> Result<(), RepoError>;
    /// Removes the group with the given id (`NotFound` if it does not exist).
    fn delete(&self, id: AccountGroupId) -> Result<(), RepoError>;
}
