//! [`Entry`] — a journal entry: a dated, described set of balanced
//! [`Posting`]s recorded together.

mod balanced_postings;
mod posting;
mod repository;

use time::Date;

pub use self::balanced_postings::{BalancedPostings, BalancedPostingsError};
pub use self::posting::{Magnitude, MagnitudeError, Posting};
pub use self::repository::EntryRepository;
use crate::ids::EntryId;

// region: Entity
/// A journal entry: a dated set of balanced postings recorded as one event.
///
/// The postings are held as [`BalancedPostings`], so a `Entry` always
/// carries a balanced set (debits equal credits within each currency). Its
/// identity is a [`EntryId`].
#[derive(Debug, Clone)]
pub struct Entry {
    id: EntryId,
    date: Date,
    description: String,
    postings: BalancedPostings,
}

impl Entry {
    // region: Constructors
    /// Creates an entry from its parts.
    ///
    /// `postings` is already guaranteed balanced by [`BalancedPostings`], and
    /// `description` may be empty.
    pub fn new(id: EntryId, date: Date, description: String, postings: BalancedPostings) -> Self {
        Entry {
            id,
            date,
            description,
            postings,
        }
    }

    /// Reconstitutes an entry from its parts, for rebuilding an entity loaded
    /// from storage.
    ///
    /// Performs no validation of its own: `postings` already encodes the
    /// balance invariant (and each [`Posting`]'s positivity) in its type. When
    /// reconstituting trusted data you can build it via
    /// [`BalancedPostings::new_unchecked`] to skip re-checking. Behaves
    /// identically to [`Entry::new`] — the two differ only in intent:
    /// `new` for a brand-new entry, `from_parts` for a storage round-trip.
    pub fn from_parts(
        id: EntryId,
        date: Date,
        description: String,
        postings: BalancedPostings,
    ) -> Self {
        Entry {
            id,
            date,
            description,
            postings,
        }
    }
    // endregion

    // region: Getters/Setters
    /// Returns the entry's unique id.
    pub fn id(&self) -> EntryId {
        self.id
    }

    /// Returns the date the entry occurred.
    pub fn date(&self) -> Date {
        self.date
    }

    /// Returns the entry's description (may be empty).
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the entry's balanced postings.
    pub fn postings(&self) -> &BalancedPostings {
        &self.postings
    }

    /// Sets the date the entry occurred.
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
    // endregion
}
// endregion
