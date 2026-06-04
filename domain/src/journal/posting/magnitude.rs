//! [`Magnitude`] — a strictly positive [`Money`] amount: the size of a posting,
//! with the debit/credit direction carried separately by
//! [`Side`](crate::side::Side).

mod error;

pub use error::MagnitudeError;

use crate::money::Money;

/// A strictly positive monetary amount — the magnitude of a posting.
///
/// A posting's direction lives in [`Side`](crate::side::Side), so its size is
/// always positive: zero and negative amounts are rejected. Holding a
/// `Magnitude` guarantees its inner [`Money`] is greater than zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magnitude(Money);

impl Magnitude {
    /// Wraps a [`Money`] amount, requiring it to be strictly positive.
    ///
    /// Returns [`MagnitudeError::NonPositive`] if `raw_amount` is zero or negative.
    pub fn new(raw_amount: Money) -> Result<Self, MagnitudeError> {
        if !raw_amount.is_positive() {
            return Err(MagnitudeError::NonPositive);
        }
        Ok(Magnitude(raw_amount))
    }

    /// Consumes the value and returns the inner [`Money`].
    pub fn to_money(self) -> Money {
        self.0
    }

    /// Wraps a [`Money`] amount without validating it.
    ///
    /// For trusted input only (e.g. a value already validated and then loaded
    /// from storage). Passing a non-positive amount yields a `Magnitude` that
    /// breaks the positivity invariant.
    pub fn new_unchecked(raw_amount: Money) -> Self {
        Magnitude(raw_amount)
    }
}

/// Fallible conversion, equivalent to [`Magnitude::new`].
impl TryFrom<Money> for Magnitude {
    type Error = MagnitudeError;

    fn try_from(value: Money) -> Result<Self, Self::Error> {
        Magnitude::new(value)
    }
}

impl AsRef<Money> for Magnitude {
    fn as_ref(&self) -> &Money {
        &self.0
    }
}
