use thiserror::Error;

use crate::money::MoneyError;

/// An error from constructing [`BalancedPostings`](super::BalancedPostings).
#[derive(Error, Debug)]
pub enum BalancedPostingsError {
    /// Fewer than two postings were provided; a transaction needs at least two.
    #[error("a transaction needs at least two postings")]
    TooFew,
    /// Debits and credits do not net to zero within some currency.
    #[error("postings do not balance: debits and credits must net to zero per currency")]
    Unbalanced,
    /// A monetary operation failed while summing the postings (e.g. overflow).
    #[error("error while summing postings: {0}")]
    Money(#[from] MoneyError),
}
