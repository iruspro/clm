use crate::RepoError;
use crate::account_group::{AccountGroup, AccountGroupId};

/// Persistence operations for [`AccountGroup`] aggregates.
pub trait AccountGroupRepository {
    /// Stores a new group.
    fn add(&self, group: &AccountGroup) -> Result<(), RepoError>;
    /// Persists changes to an existing group.
    fn update(&self, group: &AccountGroup) -> Result<(), RepoError>;
    /// Removes the group with the given id.
    fn delete(&self, group_id: AccountGroupId) -> Result<(), RepoError>;
}
