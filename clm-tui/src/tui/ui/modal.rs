//! Rendering for the overlay [`Modal`]s.
//!
//! [`render`] clears the frame and dispatches to one function per [`Modal`]
//! variant. The shared [`popup_block`] and [`centered_rect`] helpers keep the
//! popups visually consistent.

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::app::State;
use crate::app::config::KeyBindings;
use crate::app::state::{ConcreteForm, Modal};
use crate::tui::form::Form;
use crate::tui::form::fields::FieldView;

/// Draws the active [`Modal`] over a cleared frame.
pub fn render(frame: &mut Frame, state: &State, modal: &Modal) {
    // Clears the entire screen and anything already drawn.
    frame.render_widget(Clear, frame.area());

    match modal {
        Modal::Quit => quit(frame, state),
        Modal::Help => help(frame, state),
        Modal::Error(err) => error(frame, err.msg()),
        Modal::Form(inner) => form(frame, state.keys(), inner),
    }
}

/// The "Exit the app?" confirmation.
fn quit(frame: &mut Frame, state: &State) {
    let keys = state.keys();

    let block = popup_block("Exit");
    let message = Paragraph::new(format!("Exit the app? ({}/{})", keys.yes, keys.no))
        .centered()
        .block(block);

    let area = centered_rect(40, 3, frame.area());
    frame.render_widget(message, area);
}

/// A cheatsheet of the global key bindings.
fn help(frame: &mut Frame, state: &State) {
    let keys = state.keys();

    let sections = [
        (
            "Navigation",
            vec![
                (keys.focus_next, "Next screen"),
                (keys.focus_prev, "Previous screen"),
                (keys.quit, "Quit"),
                (keys.help, "Toggle help"),
            ],
        ),
        (
            "Actions",
            vec![
                (keys.new_account, "New account"),
                (keys.new_group, "New account group"),
                (keys.new_transaction, "New transaction"),
            ],
        ),
    ];

    // Width of the widest `<key>` token, so the action column lines up.
    let key_width = sections
        .iter()
        .flat_map(|(_, items)| items)
        .map(|(key, _)| key_token(*key).chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = vec![Line::from(state.title()).bold().centered(), Line::raw("")];
    for (heading, items) in &sections {
        lines.push(Line::from(*heading).bold());
        for (key, action) in items {
            lines.push(shortcut(*key, action, key_width));
        }
        lines.push(Line::raw(""));
    }

    let block = popup_block("Help").title_bottom(
        Line::from(" Press any key to close ")
            .centered()
            .white()
            .not_bold(),
    );
    let help = Paragraph::new(Text::from(lines)).block(block);

    let area = centered_rect(44, 16, frame.area());
    frame.render_widget(help, area);
}

/// A single error message.
fn error(frame: &mut Frame, message: &str) {
    let block = error_block("Error");
    let paragraph = Paragraph::new(message)
        .left_aligned()
        .wrap(Wrap { trim: true })
        .block(block);

    let area = centered_rect(50, 7, frame.area());
    frame.render_widget(paragraph, area);
}

/// Draws the active form. Each concrete form describes its own title and fields
/// (see [`Form::fields`]), so this one path renders any of them; a variant with
/// no form yet shows a placeholder.
fn form(frame: &mut Frame, keys: KeyBindings, form: &ConcreteForm) {
    match form.as_form() {
        Some(form) => render_form(frame, keys, form),
        None => coming_soon(frame),
    }
}

/// Renders any [`Form`]: one row per field, the focused field marked with a
/// caret and any validation error shown in red beneath it, with a shared key
/// hint along the bottom border.
fn render_form(frame: &mut Frame, keys: KeyBindings, form: &dyn Form) {
    let mut lines = Vec::new();
    for field in form.fields() {
        push_field(&mut lines, &field);
    }

    let hint = format!(
        " {}: next · {}/{}: change · {}: save · {}: cancel ",
        keys.focus_next, keys.prev_selection, keys.next_selection, keys.confirm, keys.cancel,
    );
    let block =
        popup_block(form.title()).title_bottom(Line::from(hint).centered().white().not_bold());

    // One row per field, plus each field's error row, plus the two borders.
    let height = lines.len() as u16 + 2;
    let paragraph = Paragraph::new(Text::from(lines)).block(block);
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(paragraph, area);
}

/// Placeholder for a form variant that isn't implemented yet.
fn coming_soon(frame: &mut Frame) {
    let block = popup_block("Coming soon");
    let paragraph = Paragraph::new("Not implemented yet.")
        .centered()
        .block(block);
    let area = centered_rect(40, 3, frame.area());
    frame.render_widget(paragraph, area);
}

/// Pushes a `Label: value` row (the focused field gets a caret and highlight),
/// and an indented red row below it when the field has an error.
fn push_field(lines: &mut Vec<Line<'static>>, field: &FieldView) {
    let marker = if field.focused { "> " } else { "  " };
    let mut label_span = Span::from(format!("{marker}{}: ", field.label));
    if field.focused {
        label_span = label_span.light_magenta().bold();
    }
    lines.push(Line::from(vec![
        label_span,
        Span::from(field.value.to_string()),
    ]));

    if let Some(message) = field.error {
        lines.push(Line::from(format!("      {message}")).red());
    }
}

/// A `<key>` token, e.g. `<q>` or `<Tab>`, as shown in the help sheet.
fn key_token(key: KeyCode) -> String {
    format!("<{key}>")
}

/// A `<key>  Action` line, the key part highlighted and left-padded to
/// `key_width` so actions align into a column.
fn shortcut(key: KeyCode, action: &str, key_width: usize) -> Line<'static> {
    let token = format!("{:<key_width$}", key_token(key));
    Line::from(vec![
        Span::from(format!("  {token}")).light_magenta().bold(),
        Span::from(format!("  {action}")),
    ])
}

/// A [`Rect`] of the given `width` and `height`, centered within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);

    area
}

/// A bordered popup box with a styled `title`.
fn popup_block<'a>(title: &'a str) -> Block<'a> {
    Block::bordered()
        .style(Style::new().white().on_black())
        .title(title)
        .title_style(Style::new().light_magenta().bold())
}

/// A [`popup_block`] with red border and title, plus a "close" hint, for the
/// error popups.
fn error_block(title: &str) -> Block<'_> {
    popup_block(title)
        .border_style(Style::new().red())
        .title_style(Style::new().red().bold())
        .title_bottom(
            Line::from(" Press any key to close ")
                .centered()
                .white()
                .not_bold(),
        )
}
