//! Rendering: turning application [`State`] into terminal widgets.
//!
//! [`render`] draws the whole frame — the tab bar plus the active screen — and
//! delegates the screen body to the [`content`] submodule and any overlay to
//! the [`modal`] submodule.

pub mod content;
pub mod modal;

use ratatui::layout::{Constraint, Layout, Offset, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::{Frame, symbols};

use crate::app::state::{Screen, State};

/// Draws one frame: the tab bar across the top and the active screen below it.
pub fn render(frame: &mut Frame, state: &State) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, main] = frame.area().layout(&layout);

    let keys = state.keys();
    let hints = format!(
        "<{}>/<{}> switch   <{}> quit   <{}> help",
        keys.focus_next, keys.focus_prev, keys.quit, keys.help,
    );
    let title = Line::from_iter([
        Span::from(state.title()).bold(),
        Span::raw("    "),
        Span::raw(hints),
    ]);
    frame.render_widget(title.centered(), top);

    if let Some(modal) = state.modal() {
        modal::render(frame, state, modal);
    } else {
        content::render(frame, state, main);
        render_tabs(frame, state, main + Offset::new(1, 0));
    }
}

/// Render the tabs.
fn render_tabs(frame: &mut Frame, state: &State, area: Rect) {
    let tabs = Tabs::new(Screen::ALL)
        .style(Color::White)
        .highlight_style(Style::default().light_magenta().on_black().bold())
        .select(state.screen() as usize)
        .divider(symbols::DOT);

    frame.render_widget(tabs, area);
}
