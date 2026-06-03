//! [`AccountGroup`] — a named container that organises
//! [`Account`](crate::account::Account)s in the chart of accounts (e.g. a "Cash"
//! group holding one account per currency). A group holds no money itself.

use uuid::Uuid;

pub mod repository;

// --- Identity ---
/// Unique identifier for an [`AccountGroup`] (a time-ordered UUID v7).
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct AccountGroupId(Uuid);

impl AccountGroupId {
    /// Generates a new, unique id.
    pub fn generate() -> Self {
        AccountGroupId(Uuid::now_v7())
    }
}

impl From<Uuid> for AccountGroupId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        AccountGroupId(u)
    }
}

// --- Entity ---
/// A named group of accounts in the chart of accounts.
///
/// Groups organise accounts (e.g. a "Cash" group with one account per currency).
/// They hold no money and cannot be posted to.
#[derive(Debug)]
pub struct AccountGroup {
    id: AccountGroupId,
    name: String,
    description: String,
}

impl AccountGroup {
    /// Creates a new group with a freshly generated id.
    ///
    /// `description` may be omitted (`None`); it is then stored as an empty string.
    pub fn new(name: impl Into<String>, description: Option<&str>) -> Self {
        AccountGroup {
            id: AccountGroupId::generate(),
            name: name.into(),
            description: description.map(|d| d.to_string()).unwrap_or_default(),
        }
    }

    /// Reconstitutes a group from stored fields (e.g. a database row), trusting
    /// that it was valid when persisted. Use [`AccountGroup::new`] to create one.
    pub fn from_parts(id: AccountGroupId, name: String, description: String) -> Self {
        AccountGroup {
            id,
            name,
            description,
        }
    }

    /// Returns the group's unique id.
    pub fn id(&self) -> AccountGroupId {
        self.id
    }

    /// Returns the group's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the group's description, or an empty string if none was set.
    pub fn description(&self) -> &str {
        &self.description
    }
}
