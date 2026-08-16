//! Service: rename a group, or change what it says about itself.

use thiserror::Error;

use crate::domain::account_group::AccountGroupRepository;
use crate::domain::{AccountGroupId, Name, RepoError};

/// Input for [`Service`].
#[derive(Debug)]
pub struct Command {
    pub id: AccountGroupId,
    pub name: Name,
    pub description: String,
}

/// Loads a group, applies the command to it, and stores it again.
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

    /// Read, modify, write — rather than writing the command straight out, so
    /// an edit of a group that is not there fails instead of creating one.
    pub fn execute(&self, cmd: Command) -> Result<(), ServiceError> {
        let mut group = self.groups.read(cmd.id).map_err(|err| match err {
            RepoError::NotFound => ServiceError::NotFound,
            err => ServiceError::Repo(err),
        })?;

        group.set_name(cmd.name);
        group.set_description(cmd.description);

        self.groups.update(&group)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// No group is stored under the id the command names.
    #[error("no group with that id")]
    NotFound,
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
