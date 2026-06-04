use crate::RepoError;
use crate::ids::TransactionId;
use crate::journal::Transaction;

/// Persistence operations for [`Transaction`] aggregates.
pub trait TransactionRepository {
    /// Stores a new transaction.
    fn add(&self, transaction: &Transaction) -> Result<(), RepoError>;
    /// Persists changes to an existing transaction.
    fn update(&self, transaction: &Transaction) -> Result<(), RepoError>;
    /// Removes the transaction with the given id.
    fn delete(&self, transaction_id: TransactionId) -> Result<(), RepoError>;

    /// Loads the transaction with the given id (`NotFound` if it does not exist).
    fn get(&self, transaction_id: TransactionId) -> Result<Transaction, RepoError>;

    fn get_all(&self) -> Result<Vec<Transaction>, RepoError>;
}
