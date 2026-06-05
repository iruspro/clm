//! Use case: record a new transaction (journal entry) in the ledger.

use domain::{
    TransactionId,
    journal::{BalancedPostings, Transaction, TransactionRepository},
};
use time::Date;

use crate::error::AppResult;

/// Input for [`AddTransactionUseCase`].
///
/// The `postings` arrive as [`BalancedPostings`], so balancing (≥2 postings,
/// debits equal credits per currency) has already been enforced at the boundary
/// — this use case never has to re-check it. The `date` is supplied by the
/// caller rather than read from a clock, keeping the use case deterministic; the
/// `description` may be empty.
pub struct AddTransactionCommand {
    pub date: Date,
    pub description: String,
    pub postings: BalancedPostings,
}

/// Builds a [`Transaction`] with a freshly generated id and stores it, returning
/// that id.
pub struct AddTransactionUseCase<TR> {
    transactions: TR,
}

impl<TR> AddTransactionUseCase<TR>
where
    TR: TransactionRepository,
{
    pub fn new(transactions: TR) -> Self {
        AddTransactionUseCase { transactions }
    }

    pub fn execute(&self, cmd: AddTransactionCommand) -> AppResult<TransactionId> {
        let tx = Transaction::new(
            TransactionId::generate(),
            cmd.date,
            cmd.description,
            cmd.postings,
        );

        self.transactions.add(&tx)?;

        Ok(tx.id())
    }
}
