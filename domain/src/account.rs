//! [`Account`] — a single-currency account in the chart of accounts: the unit
//! that postings are recorded against, together with its classification
//! ([`AccountKind`]).

mod error;
mod repository;

pub use self::error::AccountError;
pub use self::repository::AccountRepository;
use crate::ids::{AccountGroupId, AccountId};
use crate::money::Currency;
use crate::name::Name;
use crate::side::Side;

// region: Account entity
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
    // region: Constructors
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

    /// Reconstitutes an account from its parts.
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
    // endregion

    // region: Getters/Setters
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

    /// Replaces the account's name.
    pub fn set_name(&mut self, name: Name) {
        self.name = name;
    }

    /// Replaces the account's description. Pass an empty `String` to clear it.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Moves the account into the given group, or detaches it from any group
    /// when `None`.
    pub fn set_group_id(&mut self, group_id: Option<AccountGroupId>) {
        self.group_id = group_id;
    }
    // endregion
}
// endregion

// region: Classification
/// The accounting type of an account — one of the five standard categories.
///
/// This fixes the account's normal balance side (see
/// [`normal_balance`](AccountKind::normal_balance)).
///
/// The discriminants are part of the storage format (see
/// [`as_u8`](AccountKind::as_u8) / [`TryFrom<u8>`](AccountKind::try_from)), so
/// they are written out explicitly and must never be renumbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Something you own (cash, bank balance). Normal balance: debit.
    Asset = 0,
    /// Something you owe (credit-card debt, loans). Normal balance: credit.
    Liability = 1,
    /// Net worth — opening balances and accumulated result. Normal balance: credit.
    Equity = 2,
    /// A source of money that increases equity (salary, interest). Normal balance: credit.
    Income = 3,
    /// A use of money that decreases equity (groceries, rent). Normal balance: debit.
    Expense = 4,
}

impl AccountKind {
    /// All account kinds — the single source of truth for the variant list.
    pub const ALL: [AccountKind; 5] = [
        AccountKind::Asset,
        AccountKind::Liability,
        AccountKind::Equity,
        AccountKind::Income,
        AccountKind::Expense,
    ];

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

    /// Encodes the kind as its stable discriminant, for storage.
    ///
    /// The inverse is [`TryFrom<u8>`](AccountKind::try_from).
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Decodes a kind from its stored discriminant, the inverse of
/// [`as_u8`](AccountKind::as_u8).
///
/// Returns [`AccountError::UnknownKind`] for a value no variant claims.
impl TryFrom<u8> for AccountKind {
    type Error = AccountError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        AccountKind::ALL
            .into_iter()
            .find(|kind| kind.as_u8() == value)
            .ok_or(AccountError::UnknownKind(value))
    }
}
// endregion
