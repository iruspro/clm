//! Service: rename an account, or change what it says about itself.

use thiserror::Error;

use crate::domain::account::AccountRepository;
use crate::domain::{AccountId, Name, RepoError};

/// Input for [`Service`].
///
/// Kind and currency are not here: an account's postings are recorded under
/// both, so changing either would silently restate history. Make a new account
/// instead.
#[derive(Debug)]
pub struct Command {
    pub id: AccountId,
    pub name: Name,
    pub description: String,
}

/// Loads an account, applies the command to it, and stores it again.
pub struct Service<AR> {
    accounts: AR,
}

impl<AR> Service<AR>
where
    AR: AccountRepository,
{
    pub fn new(accounts: AR) -> Self {
        Service { accounts }
    }

    /// Read, modify, write — so an edit of an account that is not there fails
    /// instead of creating one, and the fields left out keep their values.
    pub fn execute(&self, cmd: Command) -> Result<(), ServiceError> {
        let mut account = self.accounts.read(cmd.id).map_err(|err| match err {
            RepoError::NotFound => ServiceError::NotFound,
            err => ServiceError::Repo(err),
        })?;

        account.set_name(cmd.name);
        account.set_description(cmd.description);

        self.accounts.update(&account)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// No account is stored under the id the command names.
    #[error("no account with that id")]
    NotFound,
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}
