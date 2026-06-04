//! [`Transaction`] — a journal entry: a dated, described set of balanced
//! [`Posting`](posting::Posting)s recorded together.

pub mod balanced_postings;
pub mod posting;
pub mod repository;

use time::Date;

use crate::{ids::TransactionId, journal::balanced_postings::BalancedPostings};

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

    /// Reconstitutes a transaction from stored fields (e.g. a database row),
    /// trusting it was valid when persisted.
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
