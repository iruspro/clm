//! [`Posting`] — one line of a [`Transaction`](crate::journal::Transaction): an
//! amount applied to an account on the debit or credit side.

pub mod magnitude;

use crate::{account::AccountId, journal::posting::magnitude::Magnitude, money::Money, side::Side};

/// A single line of a transaction: an amount applied to an account on one side.
///
/// The account is referenced by [`AccountId`] — postings live inside the
/// transaction aggregate and reference accounts rather than embedding them. The
/// [`amount`](Magnitude) is always positive; its direction is given by [`Side`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posting {
    account_id: AccountId,
    side: Side,
    amount: Magnitude,
}

impl Posting {
    // --- Constructors ---
    /// Creates a posting applying `amount` to `account_id` on the given `side`.
    pub fn new(account_id: AccountId, side: Side, amount: Magnitude) -> Self {
        Posting {
            account_id,
            side,
            amount,
        }
    }

    // --- Accessors ---
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
        let amount = self.amount.into_money();
        match self.side {
            Side::Debit => amount,
            Side::Credit => -amount,
        }
    }
}
