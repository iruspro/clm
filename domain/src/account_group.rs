//! [`AccountGroup`] — a named container that organises
//! [`Account`](crate::account::Account)s in the chart of accounts (e.g. a "Cash"
//! group holding one account per currency). A group holds no money itself.

use crate::{ids::AccountGroupId, name::Name};

pub mod repository;

pub use repository::AccountGroupRepository;

// --- Entity ---
/// A named group of accounts in the chart of accounts.
///
/// Groups organise accounts (e.g. a "Cash" group with one account per currency).
/// They hold no money and cannot be posted to.
#[derive(Debug, Clone)]
pub struct AccountGroup {
    id: AccountGroupId,
    name: Name,
    description: String,
}

impl AccountGroup {
    // --- Constructors ---
    /// Creates a new group with a freshly generated id.
    ///
    /// Pass an empty `description` (`String::new()`) if the group has none.
    pub fn new(id: AccountGroupId, name: Name, description: String) -> Self {
        AccountGroup {
            id,
            name,
            description,
        }
    }

    /// Reconstitutes a group from stored fields (e.g. a database row), trusting
    /// that it was valid when persisted. Use [`AccountGroup::new`] to create one.
    pub fn from_parts(id: AccountGroupId, name: Name, description: String) -> Self {
        AccountGroup {
            id,
            name,
            description,
        }
    }

    // --- Accessors ---
    /// Returns the group's unique id.
    pub fn id(&self) -> AccountGroupId {
        self.id
    }

    /// Returns the group's display name.
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the group's description, or an empty string if none was set.
    pub fn description(&self) -> &str {
        &self.description
    }

    // --- Behavior ---
    /// Replaces the group's name.
    pub fn rename(&mut self, name: Name) {
        self.name = name;
    }

    /// Replaces the group's description. Pass an empty `String` to clear it.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}
