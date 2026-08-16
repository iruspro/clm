//! [`Route`]: the pages the app can navigate to, and their location-bar paths.

use std::fmt::Display;

use uuid::Uuid;

use crate::domain::account::AccountKind;

/// A page the app can navigate to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Route {
    /// The main menu, where the app starts.
    Home,
    /// The help page.
    Help,
    /// The accounts screen, showing whichever [`View`] was asked for.
    Accounts(View),
    /// One account, or an account that does not exist yet.
    Account(Option<Uuid>),
    /// One account group, or a group that does not exist yet.
    Group(Option<Uuid>),
    /// The list of journal entries.
    Journal,
    /// One journal entry, or an entry that has not been recorded yet.
    Transaction {
        /// The entry, or `None` for one being recorded.
        id: Option<Uuid>,
        /// The account whose posting the cursor lands on, set when the reader
        /// arrived from that account's statement.
        from: Option<Uuid>,
    },
}

/// What the accounts screen has in its lower half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// What the whole ledger adds up to.
    Summary,
    /// The accounts of one kind, grouped.
    Kind(AccountKind),
}

impl Route {
    /// Returns the route of the main menu.
    pub fn home() -> Self {
        Self::Home
    }

    /// Returns the route of the help page.
    pub fn help() -> Self {
        Self::Help
    }

    /// Returns the route of the accounts screen, on the summary it opens at.
    pub fn accounts() -> Self {
        Self::Accounts(View::Summary)
    }

    /// Returns the route of the account `id` names.
    pub fn account(id: Uuid) -> Self {
        Self::Account(Some(id))
    }

    /// Returns the route of an account that has not been created yet.
    pub fn new_account() -> Self {
        Self::Account(None)
    }

    /// Returns the route of the group `id` names.
    pub fn group(id: Uuid) -> Self {
        Self::Group(Some(id))
    }

    /// Returns the route of a group that has not been created yet.
    pub fn new_group() -> Self {
        Self::Group(None)
    }

    /// Returns the route of the journal.
    pub fn journal() -> Self {
        Self::Journal
    }

    /// Returns the route of the entry `id` names.
    pub fn transaction(id: Uuid) -> Self {
        Self::Transaction {
            id: Some(id),
            from: None,
        }
    }

    /// Returns the route of the entry `id` names, opened on the posting that
    /// touches `account` — where a statement line leads.
    pub fn posting(id: Uuid, account: Uuid) -> Self {
        Self::Transaction {
            id: Some(id),
            from: Some(account),
        }
    }

    /// Returns the route of an entry that has not been recorded yet.
    pub fn new_transaction() -> Self {
        Self::Transaction {
            id: None,
            from: None,
        }
    }
}

/// Renders the route as the path shown in the runtime's location bar.
impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let route = match self {
            Route::Home => "home",
            Route::Help => "home/help",
            // One screen either way: the view is a parameter of it, not
            // another page.
            Route::Accounts(_) => "home/accounts",
            Route::Account(Some(_)) => "home/accounts/account",
            Route::Account(None) => "home/accounts/account/new",
            Route::Group(Some(_)) => "home/accounts/group",
            Route::Group(None) => "home/accounts/group/new",
            Route::Journal => "home/journal",
            Route::Transaction { id: Some(_), .. } => "home/journal/entry",
            Route::Transaction { id: None, .. } => "home/journal/entry/new",
        };

        write!(f, "{}", route)
    }
}
