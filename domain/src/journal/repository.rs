use crate::{error::repository::RepoResult, ids::TransactionId, journal::Transaction};

/// Persistence operations for [`Transaction`] aggregates.
pub trait TransactionRepository {
    /// Stores a new transaction.
    fn add(&self, transaction: &Transaction) -> RepoResult<()>;
    /// Persists changes to an existing transaction.
    fn update(&self, transaction: &Transaction) -> RepoResult<()>;
    /// Removes the transaction with the given id.
    fn delete(&self, transaction_id: TransactionId) -> RepoResult<()>;

    /// Loads the transaction with the given id (`NotFound` if it does not exist).
    fn get(&self, transaction_id: TransactionId) -> RepoResult<Transaction>;

    fn get_all(&self) -> RepoResult<Vec<Transaction>>;
}
