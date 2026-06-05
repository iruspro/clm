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
    fn delete(&self, id: AccountId) -> RepoResult<()>;

    /// Loads the account with the given id (`NotFound` if it does not exist).
    fn get(&self, id: AccountId) -> RepoResult<Account>;
    /// Returns the account's current balance, in its own currency.
    fn balance(&self, id: AccountId) -> RepoResult<Money>;

    /// Loads every account together with its current balance.
    ///
    /// The batch counterpart of [`get`](Self::get) + [`balance`](Self::balance):
    /// it returns one [`AccountWithBalance`] per account, so an account with no
    /// postings appears with a zero balance in its own currency rather than
    /// being absent.
    fn list_with_balances(&self) -> RepoResult<Vec<AccountWithBalance>>;
}

/// An [`Account`] paired with its current balance, as produced by
/// [`AccountRepository::list_with_balances`].
///
/// A plain read projection: it carries no invariants of its own, so its fields
/// are public.
#[derive(Debug, Clone)]
pub struct AccountWithBalance {
    pub account: Account,
    pub balance: Money,
}
