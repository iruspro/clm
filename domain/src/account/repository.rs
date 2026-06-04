use crate::RepoError;
use crate::account::{Account, AccountId};
use crate::money::Money;

/// Persistence operations for [`Account`] aggregates.
pub trait AccountRepository {
    /// Stores a new account.
    fn add(&self, account: &Account) -> Result<(), RepoError>;
    /// Persists changes to an existing account.
    fn update(&self, account: &Account) -> Result<(), RepoError>;
    /// Removes the account with the given id.
    fn delete(&self, id: AccountId) -> Result<(), RepoError>;

    /// Loads the account with the given id (`NotFound` if it does not exist).
    fn get(&self, id: AccountId) -> Result<Account, RepoError>;
    /// Returns the account's current balance, in its own currency.
    fn balance(&self, id: AccountId) -> Result<Money, RepoError>;

    /// Loads every account together with its current balance.
    ///
    /// The batch counterpart of [`get`](Self::get) + [`balance`](Self::balance):
    /// it returns one [`AccountWithBalance`] per account, so an account with no
    /// postings appears with a zero balance in its own currency rather than
    /// being absent.
    fn list_with_balances(&self) -> Result<Vec<AccountWithBalance>, RepoError>;
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
