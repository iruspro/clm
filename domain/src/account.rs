//! [`Account`] — a single-currency account in the chart of accounts: the unit
//! that postings are recorded against, together with its classification
//! ([`AccountKind`]).

pub mod repository;

use crate::{account_group::AccountGroupId, journal::Side, money::Currency};
use uuid::Uuid;

// --- Identity ---
/// Unique identifier for an [`Account`] (a time-ordered UUID v7).
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct AccountId(Uuid);

impl AccountId {
    /// Generates a new, unique id.
    pub fn generate() -> Self {
        AccountId(Uuid::now_v7())
    }
}

impl From<Uuid> for AccountId {
    /// Wraps an existing UUID — used when reconstituting from storage.
    fn from(u: Uuid) -> Self {
        AccountId(u)
    }
}

// --- Entity ---
/// A single account in the chart of accounts.
///
/// Holds exactly one [`Currency`] and is the unit that postings are recorded
/// against. Its [`AccountKind`] fixes how the balance behaves (see
/// [`AccountKind::normal_balance`]). May belong to an
/// [`AccountGroup`](crate::account_group::AccountGroup).
#[derive(Debug)]
pub struct Account {
    id: AccountId,
    kind: AccountKind,
    currency: Currency,

    name: String,
    description: String,

    group_id: Option<AccountGroupId>,
}

impl Account {
    /// Creates a new account with a freshly generated id and no group.
    ///
    /// `description` may be omitted (`None`); it is then stored as an empty string.
    pub fn new(
        kind: AccountKind,
        currency: Currency,
        name: impl Into<String>,
        description: Option<&str>,
    ) -> Self {
        Account {
            id: AccountId::generate(),
            kind,
            currency,
            name: name.into(),
            description: description.map(|d| d.to_string()).unwrap_or_default(),

            group_id: None,
        }
    }

    /// Reconstitutes an account from stored fields (e.g. a database row),
    /// trusting it was valid when persisted. Use [`Account::new`] to create one.
    pub fn from_parts(
        id: AccountId,
        kind: AccountKind,
        currency: Currency,
        name: String,
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
    pub fn name(&self) -> &str {
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
}

// --- Classification ---
/// The accounting type of an account — one of the five standard categories.
///
/// This fixes the account's normal balance side (see
/// [`normal_balance`](AccountKind::normal_balance)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Something you own (cash, bank balance). Normal balance: debit.
    Asset(AssetKind),
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
            AccountKind::Asset(_) | AccountKind::Expense => Side::Debit,
            AccountKind::Liability | AccountKind::Equity | AccountKind::Income => Side::Credit,
        }
    }
}

/// The kind of asset an [`AccountKind::Asset`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    /// A bank account.
    Bank,
    /// Physical cash.
    Cash,
}
