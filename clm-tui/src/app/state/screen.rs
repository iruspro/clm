//! Which top-level screen is showing.

/// The top-level screens, in tab-bar order.
///
/// The explicit discriminants line the variants up with [`ALL`](Screen::ALL):
/// `state.screen() as usize` is the index the tab widget highlights, so the two
/// stay in sync as long as they are edited together.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Screen {
    /// Accounts and their balances. The screen the app opens on.
    #[default]
    Accounts = 0,
    /// Income and expense categories.
    Categories = 1,
    /// The transaction ledger.
    Transactions = 2,
}

impl Screen {
    /// Tab labels, indexed by the variants' discriminants.
    pub const ALL: [&str; 3] = ["Accounts", "Categories", "Transactions"];

    /// The next screen to the right, wrapping around.
    pub(super) fn toggle_next(self) -> Self {
        match self {
            Screen::Accounts => Screen::Categories,
            Screen::Categories => Screen::Transactions,
            Screen::Transactions => Screen::Accounts,
        }
    }

    /// The next screen to the left, wrapping around.
    pub(super) fn toggle_prev(self) -> Self {
        match self {
            Screen::Accounts => Screen::Transactions,
            Screen::Categories => Screen::Accounts,
            Screen::Transactions => Screen::Categories,
        }
    }
}
