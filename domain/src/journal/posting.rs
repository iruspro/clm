//! [`Posting`] — one line of a [`Entry`](crate::journal::Entry): an
//! amount applied to an account on the debit or credit side.

mod magnitude;

pub use magnitude::{Magnitude, MagnitudeError};

use crate::ids::AccountId;
use crate::money::Money;
use crate::side::Side;

/// A single line of an entry: an amount applied to an account on one side.
///
/// The account is referenced by [`AccountId`] — postings live inside the
/// journal entry aggregate and reference accounts rather than embedding them. The
/// [`amount`](Magnitude) is always positive; its direction is given by [`Side`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posting {
    account_id: AccountId,
    side: Side,
    amount: Magnitude,
}

impl Posting {
    /// Creates a posting applying `amount` to `account_id` on the given `side`.
    pub fn new(account_id: AccountId, side: Side, amount: Magnitude) -> Self {
        Posting {
            account_id,
            side,
            amount,
        }
    }

    /// Returns the id of the account this posting applies to.
    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    /// Returns the posting's side (debit or credit).
    pub fn side(&self) -> Side {
        self.side
    }

    /// Returns the posting's amount (always positive; direction is the [`side`](Self::side)).
    pub fn amount(&self) -> Magnitude {
        self.amount
    }

    /// Returns the amount as a signed [`Money`]: positive for a debit, negative
    /// for a credit.
    ///
    /// Uses the fixed debit-positive convention and does not depend on the
    /// account's type. Summing `signed()` over an account's postings yields its
    /// raw (debit-positive) balance.
    pub fn signed(&self) -> Money {
        let amount = self.amount.to_money();
        match self.side {
            Side::Debit => amount,
            Side::Credit => -amount,
        }
    }
}
