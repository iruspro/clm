use std::{error::Error, fmt};

use crate::money::MoneyError;

/// Shorthand for a result whose error is a [`BalancedPostingsError`].
pub type BalancedPostingsResult<T> = Result<T, BalancedPostingsError>;

/// An error from constructing [`BalancedPostings`](super::BalancedPostings).
#[derive(Debug)]
pub enum BalancedPostingsError {
    /// Fewer than two postings were provided; a transaction needs at least two.
    TooFew,
    /// Debits and credits do not net to zero within some currency.
    Unbalanced,
    /// A monetary operation failed while summing the postings (e.g. overflow).
    Money(MoneyError),
}

impl fmt::Display for BalancedPostingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BalancedPostingsError::TooFew => {
                write!(f, "a transaction needs at least two postings")
            }
            BalancedPostingsError::Unbalanced => write!(
                f,
                "postings do not balance: debits and credits must net to zero per currency"
            ),
            BalancedPostingsError::Money(err) => {
                write!(f, "error while summing postings: {err}")
            }
        }
    }
}

impl Error for BalancedPostingsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BalancedPostingsError::Money(err) => Some(err),
            _ => None,
        }
    }
}

impl From<MoneyError> for BalancedPostingsError {
    fn from(value: MoneyError) -> Self {
        BalancedPostingsError::Money(value)
    }
}
