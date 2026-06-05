//! Query: list every transaction (journal entry) in the ledger.

use domain::TransactionId;
use domain::journal::{Posting, TransactionRepository};
use time::Date;

use crate::error::AppError;

/// A read model describing one transaction.
///
/// Named distinctly from the [`Transaction`](domain::journal::Transaction)
/// entity (and kept separate from it) so the UI depends on this flat view rather
/// than on the domain aggregate's getter API. `postings` is exposed as a plain
/// `Vec<Posting>`: the
/// [`BalancedPostings`](domain::journal::BalancedPostings) invariant (≥2 postings
/// that balance per currency) is a write-side guarantee the UI doesn't need, so
/// the read side just hands over the list to render.
pub struct TransactionSummary {
    pub id: TransactionId,
    pub date: Date,
    pub description: String,
    pub postings: Vec<Posting>,
}

/// Lists every transaction as [`TransactionSummary`] rows.
pub struct GetTransactionsQuery<TR> {
    transactions: TR,
}

impl<TR> GetTransactionsQuery<TR>
where
    TR: TransactionRepository,
{
    pub fn new(transactions: TR) -> Self {
        GetTransactionsQuery { transactions }
    }

    /// Returns one [`TransactionSummary`] per transaction, empty when there are
    /// none.
    pub fn execute(&self) -> Result<Vec<TransactionSummary>, AppError> {
        let transactions = self.transactions.get_all()?;

        Ok(transactions
            .into_iter()
            .map(|t| {
                // `into_parts` moves the fields out instead of cloning.
                let (id, date, description, postings) = t.into_parts();
                TransactionSummary {
                    id,
                    date,
                    description,
                    postings: postings.into_vec(),
                }
            })
            .collect())
    }
}
