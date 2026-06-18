use thiserror::Error;

use crate::money::Currency;

/// An error from a monetary operation.
#[derive(Error, Debug)]
pub enum MoneyError {
    /// The two amounts are in different currencies and cannot be combined.
    #[error("currency mismatch: cannot combine {} and {}", .left.code(), .right.code())]
    CurrencyMismatch { left: Currency, right: Currency },
    /// The result does not fit in the underlying `i64` (minor units).
    #[error("arithmetic overflow")]
    Overflow,
}
