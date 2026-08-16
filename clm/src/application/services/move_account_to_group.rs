//! Service: put an account in a group, move it to another, or take it out.

use thiserror::Error;

use crate::domain::account::AccountRepository;
use crate::domain::account_group::AccountGroupRepository;
use crate::domain::{AccountGroupId, AccountId, RepoError};

/// Input for [`Service`].
///
/// `group_id` of `None` takes the account out of whatever group it is in —
/// the same operation, with no destination.
#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub group_id: Option<AccountGroupId>,
}

/// Repoints an account at another group, after checking that both ends exist.
pub struct Service<AR, GR> {
    accounts: AR,
    groups: GR,
}

impl<AR, GR> Service<AR, GR>
where
    AR: AccountRepository,
    GR: AccountGroupRepository,
{
    pub fn new(accounts: AR, groups: GR) -> Self {
        Service { accounts, groups }
    }

    /// The destination is read before the account is touched: the schema sets
    /// `account_group_id` to NULL when a group goes away, and SQLite only
    /// enforces the foreign key when `PRAGMA foreign_keys` is on — so a move to
    /// a group that no longer exists could otherwise be stored as "ungrouped"
    /// and look like it worked.
    pub fn execute(&self, cmd: Command) -> Result<(), ServiceError> {
        if let Some(group_id) = cmd.group_id {
            self.groups.read(group_id).map_err(|err| match err {
                RepoError::NotFound => ServiceError::GroupNotFound,
                err => ServiceError::Repo(err),
            })?;
        }

        let mut account = self
            .accounts
            .read(cmd.account_id)
            .map_err(|err| match err {
                RepoError::NotFound => ServiceError::AccountNotFound,
                err => ServiceError::Repo(err),
            })?;

        account.set_group_id(cmd.group_id);
        self.accounts.update(&account)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// No account is stored under the id the command names.
    #[error("no account with that id")]
    AccountNotFound,
    /// No group is stored under the id the command names.
    #[error("no group with that id")]
    GroupNotFound,
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
