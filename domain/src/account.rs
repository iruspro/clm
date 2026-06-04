//! [`Account`] — a single-currency account in the chart of accounts: the unit
//! that postings are recorded against, together with its classification
//! ([`AccountKind`]).

mod repository;

pub use self::repository::{AccountRepository, AccountWithBalance};
use crate::ids::{AccountGroupId, AccountId};
use crate::money::Currency;
use crate::name::Name;
use crate::side::Side;

// --- Entity ---
/// A single account in the chart of accounts.
///
/// Holds exactly one [`Currency`] and is the unit that postings are recorded
/// against. Its [`AccountKind`] fixes how the balance behaves (see
/// [`AccountKind::normal_balance`]). May belong to an
/// [`AccountGroup`](crate::account_group::AccountGroup).
#[derive(Debug, Clone)]
pub struct Account {
    id: AccountId,
    kind: AccountKind,
    currency: Currency,

    name: Name,
    description: String,

    group_id: Option<AccountGroupId>,
}

impl Account {
    // --- Constructors ---
    /// Creates a new account from the given fields.
    ///
    /// Generate `id` with [`AccountId::new`]. Pass `None` for `group_id`
    /// if the account belongs to no group, and an empty `description`
    /// (`String::new()`) if it has none. `name` is already validated by its
    /// type — build it via [`Name::new`](crate::Name::new).
    pub fn new(
        id: AccountId,
        kind: AccountKind,
        currency: Currency,
        name: Name,
        description: String,
        group_id: Option<AccountGroupId>,
    ) -> Self {
        Account {
            id,
            kind,
            currency,
            name,
            description,
            group_id,
        }
    }

    /// Reconstitutes an account from its parts — the inverse of
    /// [`into_parts`](Account::into_parts), for rebuilding an entity loaded
    /// from storage.
    ///
    /// Performs no validation of its own; each part enforces its own invariant
    /// at construction. When reconstituting trusted data you can build those
    /// parts via their `*_unchecked` constructors (e.g.
    /// [`Name::new_unchecked`](crate::Name::new_unchecked)) to skip re-checking.
    /// Behaves identically to [`Account::new`] — the two differ only in intent:
    /// `new` for a brand-new account, `from_parts` for a storage round-trip.
    pub fn from_parts(
        id: AccountId,
        kind: AccountKind,
        currency: Currency,
        name: Name,
        description: String,
        group_id: Option<AccountGroupId>,
    ) -> Self {
        Account {
            id,
            kind,
            currency,
            name,
            description,
            group_id,
        }
    }

    /// Deconstructs the account into its owned parts, in the same order
    /// [`from_parts`](Account::from_parts) takes them.
    ///
    /// The inverse of `from_parts`: use it to move the fields out (e.g. when
    /// persisting, or building a read model) without cloning `name` and
    /// `description`.
    pub fn into_parts(
        self,
    ) -> (
        AccountId,
        AccountKind,
        Currency,
        Name,
        String,
        Option<AccountGroupId>,
    ) {
        (
            self.id,
            self.kind,
            self.currency,
            self.name,
            self.description,
            self.group_id,
        )
    }

    // --- Accessors ---
    /// Returns the account's unique id.
    pub fn id(&self) -> AccountId {
        self.id
    }

    /// Returns the account's classification (see [`AccountKind`]).
    pub fn kind(&self) -> AccountKind {
        self.kind
    }

    /// Returns the currency this account is denominated in.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Returns the account's display name.
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the account's description, or an empty string if none was set.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the id of the group this account belongs to, if any.
    pub fn group_id(&self) -> Option<AccountGroupId> {
        self.group_id
    }

    // --- Behavior ---
    /// Replaces the account's name.
    pub fn rename(&mut self, name: Name) {
        self.name = name;
    }

    /// Replaces the account's description. Pass an empty `String` to clear it.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Moves the account into the given group, or detaches it from any group
    /// when `None`.
    pub fn move_to_group(&mut self, group_id: Option<AccountGroupId>) {
        self.group_id = group_id;
    }
}

// --- Classification ---
/// The accounting type of an account — one of the five standard categories.
///
/// This fixes the account's normal balance side (see
/// [`normal_balance`](AccountKind::normal_balance)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Something you own (cash, bank balance). Normal balance: debit.
    Asset,
    /// Something you owe (credit-card debt, loans). Normal balance: credit.
    Liability,
    /// Net worth — opening balances and accumulated result. Normal balance: credit.
    Equity,
    /// A source of money that increases equity (salary, interest). Normal balance: credit.
    Income,
    /// A use of money that decreases equity (groceries, rent). Normal balance: debit.
    Expense,
}

impl AccountKind {
    /// The side on which this account's balance increases.
    ///
    /// Assets and expenses increase on the debit side; liabilities, equity and
    /// income increase on the credit side.
    pub fn normal_balance(&self) -> Side {
        match self {
            AccountKind::Asset | AccountKind::Expense => Side::Debit,
            AccountKind::Liability | AccountKind::Equity | AccountKind::Income => Side::Credit,
        }
    }
}
