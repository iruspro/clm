//! Pages of the application, and the few things they all draw.

pub mod account;
pub mod accounts;
pub mod group;
pub mod help;
pub mod home;
pub mod journal;
pub mod summary;
pub mod transaction;

use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Block;

use crate::domain::money::Money;

/// Separates the per-currency totals of a group, or of a whole kind.
const TOTAL_SEPARATOR: &str = " · ";

/// A split's frame. The focused one is highlighted, so it is clear which split
/// the keys act on.
pub fn pane(title: String, focused: bool) -> Block<'static> {
    let border = if focused {
        Style::new().yellow()
    } else {
        Style::new().dark_gray()
    };

    Block::bordered()
        .title(Line::from(title).bold())
        .border_style(border)
}

/// Sums `amounts` per currency, in a stable order.
///
/// A list of accounts can hold several currencies, and those totals cannot be
/// added together — so each currency keeps its own.
pub fn totals(amounts: impl Iterator<Item = Money>) -> Vec<Money> {
    let mut totals: Vec<Money> = Vec::new();

    for amount in amounts {
        let running = totals
            .iter_mut()
            .find(|total| total.currency() == amount.currency());

        match running {
            // A total passing `i64` is not worth losing the line over; keep
            // the sum so far.
            Some(total) => *total = total.checked_add(amount).unwrap_or(*total),
            None => totals.push(amount),
        }
    }

    totals.sort_by_key(|total| total.currency().as_u16());
    totals
}

/// Totals as one column: `€1,234.00 · $500.00`.
pub fn show(totals: &[Money]) -> String {
    totals
        .iter()
        .map(Money::to_string)
        .collect::<Vec<_>>()
        .join(TOTAL_SEPARATOR)
}
