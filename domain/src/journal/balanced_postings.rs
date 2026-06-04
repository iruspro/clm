//! [`BalancedPostings`] — the postings of a transaction, validated to balance
//! per currency (which implies at least two postings).

mod error;

use std::collections::HashMap;

pub use error::BalancedPostingsError;

use crate::journal::posting::Posting;
use crate::money::Money;

/// A set of postings guaranteed to balance per currency.
///
/// "Balanced" means that **within each currency** the debits and credits sum to
/// zero (using [`Posting::signed`](crate::journal::posting::Posting::signed)).
/// Currencies balance independently, so a cross-currency transaction is valid
/// only if each currency nets to zero on its own. Because magnitudes are
/// positive, balancing implies at least two postings.
#[derive(Debug, Clone)]
pub struct BalancedPostings(Vec<Posting>);

impl BalancedPostings {
    /// Validates and wraps a set of postings.
    ///
    /// Returns [`BalancedPostingsError::TooFew`] if there are fewer than two
    /// postings, [`BalancedPostingsError::Unbalanced`] if any currency's debits
    /// and credits do not net to zero, or [`BalancedPostingsError::Money`] if
    /// summing the amounts overflows.
    pub fn new(raw_postings: Vec<Posting>) -> Result<Self, BalancedPostingsError> {
        if raw_postings.len() < 2 {
            return Err(BalancedPostingsError::TooFew);
        }

        if !BalancedPostings::is_balanced(&raw_postings)? {
            return Err(BalancedPostingsError::Unbalanced);
        }

        Ok(BalancedPostings(raw_postings))
    }

    /// Consumes the value and returns the inner postings.
    pub fn into_vec(self) -> Vec<Posting> {
        self.0
    }

    /// Wraps postings without validating them.
    ///
    /// For trusted input only (e.g. a value already validated and then loaded
    /// from storage). Passing too few or unbalanced postings yields a value that
    /// breaks the invariant.
    pub fn new_unchecked(raw_postings: Vec<Posting>) -> Self {
        BalancedPostings(raw_postings)
    }

    /// Returns `true` if, within every currency, the signed amounts sum to zero.
    ///
    /// Returns [`BalancedPostingsError::Money`] if summing overflows.
    fn is_balanced(postings: &[Posting]) -> Result<bool, BalancedPostingsError> {
        let mut totals: HashMap<_, Money> = HashMap::new();

        for posting in postings {
            let m = posting.signed();
            let entry = totals
                .entry(m.currency())
                .or_insert_with(|| Money::zero(m.currency()));
            *entry = entry.checked_add(m)?;
        }

        Ok(totals.values().all(|&sum| sum.is_zero()))
    }
}

/// Fallible conversion, equivalent to [`BalancedPostings::new`].
impl TryFrom<Vec<Posting>> for BalancedPostings {
    type Error = BalancedPostingsError;

    fn try_from(value: Vec<Posting>) -> Result<Self, Self::Error> {
        BalancedPostings::new(value)
    }
}

impl AsRef<Vec<Posting>> for BalancedPostings {
    fn as_ref(&self) -> &Vec<Posting> {
        &self.0
    }
}
