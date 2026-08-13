//! Static configuration: the window title, the database location and every key
//! binding.
//!
//! [`Config::parse`] is currently a hard-coded set of defaults. It is shaped like
//! a parse step — and the fields are `&'static str` — so that reading a real
//! config file or CLI flags later is a change to that one function.

use ratatui::crossterm::event::KeyCode;

/// Key bindings for TUI actions.
///
/// Every reducer branch compares against a field of this struct rather than a
/// literal [`KeyCode`], so all of the app's key handling is remappable from one
/// place, and the help sheet renders the bindings actually in effect.
#[derive(Debug, Clone, Copy)]
pub struct KeyBindings {
    // Global.
    /// Open the quit confirmation.
    pub quit: KeyCode,
    /// Open the help sheet.
    pub help: KeyCode,
    /// Next screen; inside a form, the next field.
    pub focus_next: KeyCode,
    /// Previous screen; inside a form, the previous field.
    pub focus_prev: KeyCode,
    /// Next row in a list, or the next option of a focused choice field.
    pub next_selection: KeyCode,
    /// Previous row in a list, or the previous option of a focused choice field.
    pub prev_selection: KeyCode,

    // Modal answers.
    /// Confirm the quit prompt.
    pub yes: KeyCode,
    /// Dismiss the quit prompt.
    pub no: KeyCode,
    /// Submit a form, or acknowledge an error.
    pub confirm: KeyCode,
    /// Close a form without submitting.
    pub cancel: KeyCode,

    // Actions.
    /// Edit the selected item.
    pub edit: KeyCode,
    /// Open the "new account" form.
    pub new_account: KeyCode,
    /// Open the "new account group" form.
    pub new_group: KeyCode,
    /// Open the "new transaction" form.
    pub new_transaction: KeyCode,
}

/// Everything the app needs to start: what to call itself, where its data lives
/// and how it is driven.
///
/// [`Copy`] and `&'static str`-based, so it can be handed to the threads that
/// need it without borrowing or cloning.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Title shown in the header and on the help sheet.
    pub title: &'static str,
    /// Path to the SQLite database file, relative to the working directory.
    pub database_url: &'static str,

    /// The active key bindings.
    pub keys: KeyBindings,
}

impl Config {
    /// Builds the configuration.
    ///
    /// The defaults are vi-flavoured: `j`/`k` move a selection, `Tab`/`BackTab`
    /// move between screens and form fields, and the create actions are the
    /// first letter of what they create.
    pub fn parse() -> Self {
        let keys = KeyBindings {
            quit: KeyCode::Char('q'),
            help: KeyCode::F(1),
            focus_next: KeyCode::Tab,
            focus_prev: KeyCode::BackTab,
            next_selection: KeyCode::Char('j'), // vi down key
            prev_selection: KeyCode::Char('k'), // vi up key
            yes: KeyCode::Char('y'),
            no: KeyCode::Char('n'),
            confirm: KeyCode::Enter,
            cancel: KeyCode::Esc,
            edit: KeyCode::Char('e'),
            new_account: KeyCode::Char('a'),
            new_group: KeyCode::Char('g'),
            new_transaction: KeyCode::Char('t'),
        };

        Self {
            title: "Command Line Money",
            database_url: "clm.db",
            keys,
        }
    }
}
