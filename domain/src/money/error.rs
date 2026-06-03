use std::{error::Error, fmt};

use crate::money::Currency;

/// Shorthand for a result whose error is a [`MoneyError`].
pub type MoneyResult<T> = Result<T, MoneyError>;

/// An error from a monetary operation.
#[derive(Debug)]
pub enum MoneyError {
    /// The two amounts are in different currencies and cannot be combined.
    CurrencyMismatch { left: Currency, right: Currency },
    /// The result does not fit in the underlying `i64` (minor units).
    Overflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::CurrencyMismatch { left, right } => write!(
                f,
                "currency mismatch: cannot combine {} and {}",
                left.code(),
                right.code()
            ),
            MoneyError::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl Error for MoneyError {}
