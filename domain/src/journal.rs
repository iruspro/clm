//! [`Transaction`] — a journal entry: a dated, described set of balanced
//! [`Posting`]s recorded together.

mod balanced_postings;
mod posting;
mod repository;

use time::Date;

pub use self::balanced_postings::{BalancedPostings, BalancedPostingsError};
pub use self::posting::{Magnitude, MagnitudeError, Posting};
pub use self::repository::TransactionRepository;
use crate::ids::TransactionId;

// --- Entity ---
/// A journal entry: a dated set of balanced postings recorded as one event.
///
/// The postings are held as [`BalancedPostings`], so a `Transaction` always
/// carries a balanced set (debits equal credits within each currency). Its
/// identity is a [`TransactionId`].
#[derive(Debug, Clone)]
pub struct Transaction {
    id: TransactionId,
    date: Date,
    description: String,
    postings: BalancedPostings,
}

impl Transaction {
    // --- Constructors ---
    /// Creates a transaction from its parts.
    ///
    /// `postings` is already guaranteed balanced by [`BalancedPostings`], and
    /// `description` may be empty.
    pub fn new(
        id: TransactionId,
        date: Date,
        description: String,
        postings: BalancedPostings,
    ) -> Self {
        Transaction {
            id,
            date,
            description,
            postings,
        }
    }

    /// Reconstitutes a transaction from its parts — the inverse of
    /// [`into_parts`](Transaction::into_parts), for rebuilding an entity loaded
    /// from storage.
    ///
    /// Performs no validation of its own: `postings` already encodes the
    /// balance invariant (and each [`Posting`]'s positivity) in its type. When
    /// reconstituting trusted data you can build it via
    /// [`BalancedPostings::new_unchecked`] to skip re-checking. Behaves
    /// identically to [`Transaction::new`] — the two differ only in intent:
    /// `new` for a brand-new transaction, `from_parts` for a storage round-trip.
    pub fn from_parts(
        id: TransactionId,
        date: Date,
        description: String,
        postings: BalancedPostings,
    ) -> Self {
        Transaction {
            id,
            date,
            description,
            postings,
        }
    }

    /// Deconstructs the transaction into its owned parts, in the same order
    /// [`from_parts`](Transaction::from_parts) takes them — its inverse.
    ///
    /// Everything moves out without reallocating: [`BalancedPostings`] is handed
    /// straight back, so no [`Posting`] is cloned or rebuilt.
    pub fn into_parts(self) -> (TransactionId, Date, String, BalancedPostings) {
        (self.id, self.date, self.description, self.postings)
    }

    // --- Accessors ---
    /// Returns the transaction's unique id.
    pub fn id(&self) -> TransactionId {
        self.id
    }

    /// Returns the date the transaction occurred.
    pub fn date(&self) -> Date {
        self.date
    }

    /// Returns the transaction's description (may be empty).
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the transaction's balanced postings.
    pub fn postings(&self) -> &BalancedPostings {
        &self.postings
    }

    // --- Behavior ---
    /// Sets the date the transaction occurred.
    pub fn set_date(&mut self, date: Date) {
        self.date = date;
    }

    /// Replaces the description. Pass an empty `String` to clear it.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Replaces the postings (already validated as balanced).
    pub fn set_postings(&mut self, postings: BalancedPostings) {
        self.postings = postings;
    }
}
