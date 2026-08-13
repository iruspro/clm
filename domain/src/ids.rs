//! Typed identifiers for domain entities — thin newtypes over a UUID (v7).

// TODO: collapse the repeated id-newtype boilerplate (struct + derives +
// `generate` + `to_uuid` + `From<Uuid>`) into a `macro_rules!` macro, so each id
// becomes a single `id_type! { /// doc \n SomeId }` invocation.

use uuid::Uuid;

/// Unique identifier for an [`AccountGroup`](crate::account_group::AccountGroup) (a time-ordered UUID v7).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountGroupId(Uuid);

impl AccountGroupId {
    /// Generates a new, unique id.
    pub fn new() -> Self {
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

impl AsRef<Uuid> for AccountGroupId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for an [`Account`](crate::account::Account) (a time-ordered UUID v7).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(Uuid);

impl AccountId {
    /// Generates a new, unique id.
    pub fn new() -> Self {
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

impl AsRef<Uuid> for AccountId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for a [`Entry`](crate::journal::Entry) (a time-ordered UUID v7).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(Uuid);

impl EntryId {
    /// Generates a new, unique id.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the underlying UUID.
    pub fn to_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for EntryId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl AsRef<Uuid> for EntryId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
