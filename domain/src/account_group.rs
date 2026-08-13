//! [`AccountGroup`] — a named container that organises
//! [`Account`](crate::account::Account)s in the chart of accounts (e.g. a "Cash"
//! group holding one account per currency). A group holds no money itself.

mod repository;

pub use self::repository::AccountGroupRepository;
use crate::ids::AccountGroupId;
use crate::name::Name;

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
    // region: Constructors
    /// Creates a new group from the given fields.
    ///
    /// Generate `id` with [`AccountGroupId::new`]. Pass an empty `description`
    /// (`String::new()`) if the group has none. `name` is already validated by
    /// its type — build it via [`Name::new`](crate::Name::new).
    pub fn new(id: AccountGroupId, name: Name, description: String) -> Self {
        AccountGroup {
            id,
            name,
            description,
        }
    }

    /// Reconstitutes a group from its parts.
    ///
    /// Performs no validation of its own; each part enforces its own invariant
    /// at construction. When reconstituting trusted data you can build the
    /// [`Name`] via [`Name::new_unchecked`](crate::Name::new_unchecked) to skip
    /// re-checking. Behaves identically to [`AccountGroup::new`] — the two
    /// differ only in intent: `new` for a brand-new group, `from_parts` for a
    /// storage round-trip.
    pub fn from_parts(id: AccountGroupId, name: Name, description: String) -> Self {
        AccountGroup {
            id,
            name,
            description,
        }
    }
    // endregion

    // region: Getters/Setters
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

    /// Replaces the group's name.
    pub fn set_name(&mut self, name: Name) {
        self.name = name;
    }

    /// Replaces the group's description. Pass an empty `String` to clear it.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
    // endregion
}
