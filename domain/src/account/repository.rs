use crate::{
    account::{Account, AccountId},
    error::repository::RepoResult,
    money::Money,
};

/// Persistence operations for [`Account`] aggregates.
pub trait AccountRepository {
    /// Stores a new account.
    fn add(&self, account: &Account) -> RepoResult<()>;
    /// Persists changes to an existing account.
    fn update(&self, account: &Account) -> RepoResult<()>;
    /// Removes the account with the given id.
    fn delete(&self, account_id: AccountId) -> RepoResult<()>;

    /// Loads the account with the given id (`NotFound` if it does not exist).
    fn get(&self, account_id: AccountId) -> RepoResult<Account>;
    /// Returns the account's current balance, in its own currency.
    fn balance(&self, account_id: AccountId) -> RepoResult<Money>;
}
