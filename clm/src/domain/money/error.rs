use thiserror::Error;

use crate::domain::money::Currency;

/// An error from a monetary operation.
#[derive(Error, Debug)]
pub enum MoneyError {
    /// The two amounts are in different currencies and cannot be combined.
    #[error("currency mismatch: cannot combine {} and {}", .left.code(), .right.code())]
    CurrencyMismatch { left: Currency, right: Currency },
    /// The result does not fit in the underlying `i64` (minor units).
    #[error("arithmetic overflow")]
    Overflow,
    /// The stored discriminant does not match any [`Currency`] variant — the
    /// value came from outside the domain (a hand-edited row, an older schema)
    /// and cannot be decoded.
    #[error("unknown currency discriminant: {0}")]
    UnknownCurrency(u16),
}
