//! Service: create a new group in the chart of accounts.

use thiserror::Error;

use crate::domain::account_group::{AccountGroup, AccountGroupRepository};
use crate::domain::{AccountGroupId, Name, RepoError};

/// Input for [`Service`].
#[derive(Debug)]
pub struct Command {
    pub name: Name,
    pub description: String,
}

/// Builds an [`AccountGroup`] with a freshly generated id and stores it,
/// returning that id.
pub struct Service<GR> {
    groups: GR,
}

impl<GR> Service<GR>
where
    GR: AccountGroupRepository,
{
    pub fn new(groups: GR) -> Self {
        Service { groups }
    }

    pub fn execute(&self, cmd: Command) -> Result<AccountGroupId, ServiceError> {
        let id = AccountGroupId::new();

        let group = AccountGroup::new(id, cmd.name, cmd.description);
        self.groups.create(&group)?;

        Ok(id)
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
