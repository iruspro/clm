use crate::domain::RepoError;
use crate::domain::account::{Account, AccountId};

/// Persistence operations for [`Account`] aggregates.
pub trait AccountRepository {
    /// Stores a new account.
    fn create(&self, account: &Account) -> Result<(), RepoError>;
    /// Loads the account with the given id (`NotFound` if it does not exist).
    fn read(&self, id: AccountId) -> Result<Account, RepoError>;
    /// Persists changes to an existing account, matched by its id
    /// (`NotFound` if no such account is stored).
    fn update(&self, account: &Account) -> Result<(), RepoError>;
}
