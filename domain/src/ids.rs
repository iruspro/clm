//! Typed identifiers for domain entities — thin newtypes over a UUID (v7).

// TODO: collapse the repeated id-newtype boilerplate (struct + derives +
// `generate` + `to_uuid` + `From<Uuid>`) into a `macro_rules!` macro, so each id
// becomes a single `id_type! { /// doc \n SomeId }` invocation. Deferred until
// comfortable with macros.

use uuid::Uuid;

/// Unique identifier for an [`AccountGroup`](crate::account_group::AccountGroup) (a time-ordered UUID v7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountGroupId(Uuid);

impl AccountGroupId {
    /// Generates a new, unique id.
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the underlying UUID.
    pub fn to_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for AccountGroupId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

/// Unique identifier for an [`Account`](crate::account::Account) (a time-ordered UUID v7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(Uuid);

impl AccountId {
    /// Generates a new, unique id.
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the underlying UUID.
    pub fn to_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for AccountId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

/// Unique identifier for a [`Transaction`](crate::journal::Transaction) (a time-ordered UUID v7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(Uuid);

impl TransactionId {
    /// Generates a new, unique id.
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the underlying UUID.
    pub fn to_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TransactionId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}
