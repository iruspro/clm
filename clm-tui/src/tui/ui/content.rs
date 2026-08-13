//! Rendering for the main content area — the body of the active screen.
//!
//! [`render`] wraps the area in a border and dispatches to one function per
//! [`Screen`], each drawing into the block's inner area.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;

use crate::app::state::Screen;
use crate::app::{AccountsState, CategoriesState, State, TransactionsState};

/// Draws the active [`Screen`]'s body into `area`, framed by a border.
pub fn render(frame: &mut Frame, state: &State, area: Rect) {
    let block = Block::bordered();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match state.screen() {
        Screen::Accounts => accounts(frame, state.accounts(), inner),
        Screen::Categories => categories(frame, state.categories(), inner),
        Screen::Transactions => transactions(frame, state.transactions(), inner),
    }
}

/// Renders the Accounts screen into `area`.
fn accounts(_frame: &mut Frame, _state: &AccountsState, _area: Rect) {
    // render
}

/// Renders the Categories screen into `area`.
fn categories(_frame: &mut Frame, _state: &CategoriesState, _area: Rect) {
    // render
}

/// Renders the Transactions screen into `area`.
fn transactions(_frame: &mut Frame, _state: &TransactionsState, _area: Rect) {
    // render
}
