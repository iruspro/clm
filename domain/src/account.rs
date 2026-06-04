//! [`Account`] — a single-currency account in the chart of accounts: the unit
//! that postings are recorded against, together with its classification
//! ([`AccountKind`]).

pub mod repository;

use crate::{ids::AccountGroupId, ids::AccountId, money::Currency, name::Name, side::Side};

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
    /// Creates a new account with a freshly generated id and no group.
    ///
    /// Pass an empty `description` (`String::new()`) if the account has none.
    pub fn new(
        id: AccountId,
        kind: AccountKind,
        currency: Currency,
        name: Name,
        description: String,
    ) -> Self {
        Account {
            id,
            kind,
            currency,
            name,
            description,

            group_id: None,
        }
    }

    /// Reconstitutes an account from stored fields (e.g. a database row),
    /// trusting it was valid when persisted. Use [`Account::new`] to create one.
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
