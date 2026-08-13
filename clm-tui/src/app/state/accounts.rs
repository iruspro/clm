//! State of the Accounts screen.

use application::views::accounts;
use domain::money::Money;

/// The account list as the UI sees it, plus which row is selected.
///
/// The rows are whatever the last
/// [`AccountsLoaded`](crate::actor::InternalEvent::AccountsLoaded) event
/// delivered — this is a cache of a query result, never the source of truth.
#[derive(Debug, Default)]
pub struct AccountsState {
    /// Combined balance per currency. Balances aren't summed across currencies,
    /// so there is one entry for each currency in use.
    #[expect(
        dead_code,
        reason = "not computed or rendered until the Accounts screen draws a totals row"
    )]
    total_balance: Vec<Money>,

    accounts: Vec<accounts::ResultItem>,
    /// Index into `accounts`. Always in bounds while the list is non-empty; `0`
    /// on an empty list, where it selects nothing.
    selected: usize,
}

impl AccountsState {
    /// Replaces the list with a freshly loaded one, keeping the selection on the
    /// same account where possible.
    ///
    /// Reloading happens after every write, so selection is re-anchored **by
    /// account id** rather than by index — otherwise inserting a row above the
    /// cursor would silently move it. If the selected account is gone from the
    /// new list, the selection falls back to the next index, clamped to the end.
    pub fn set_accounts(&mut self, accounts: Vec<accounts::ResultItem>) {
        let selected_account_id = self.accounts.get(self.selected).map(|a| a.id);

        self.accounts = accounts;

        self.selected = selected_account_id
            .and_then(|id| self.accounts.iter().position(|a| a.id == id))
            .unwrap_or_else(|| self.next());
    }

    /// Moves the selection down one row, stopping at the last.
    pub fn select_next(&mut self) {
        self.selected = self.next();
    }

    /// Moves the selection up one row, stopping at the first.
    pub fn select_prev(&mut self) {
        self.selected = self.prev();
    }

    /// The next index, clamped to the last row. Selection does not wrap.
    fn next(&self) -> usize {
        usize::min(self.selected + 1, self.accounts.len().saturating_sub(1))
    }

    /// The previous index, saturating at `0`.
    fn prev(&self) -> usize {
        self.selected.saturating_sub(1)
    }
}
