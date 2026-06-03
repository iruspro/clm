use crate::{
    account_group::{AccountGroup, AccountGroupId},
    error::repository::RepoResult,
};

/// Persistence operations for [`AccountGroup`] aggregates.
pub trait AccountGroupRepository {
    /// Stores a new group.
    fn add(&self, group: &AccountGroup) -> RepoResult<()>;
    /// Persists changes to an existing group.
    fn update(&self, group: &AccountGroup) -> RepoResult<()>;
    /// Removes the group with the given id.
    fn delete(&self, group_id: AccountGroupId) -> RepoResult<()>;
}
