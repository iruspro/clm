use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};

use crate::app::{Clm, Cmd};

/// Width the defined terms are padded to, so their descriptions line up.
const TERM_WIDTH: usize = 14;

/// Width the key names are padded to, so their descriptions line up.
const KEY_WIDTH: usize = 11;

/// The keys this page answers to, spelled out under it.
const HINT: &str = "j/k to scroll · H to go back · q to quit";

#[derive(Debug)]
pub struct Model {
    /// Index of the first line drawn; scrolling moves it down the text.
    scroll: u16,
}

impl Model {
    /// Opens the help at the top of the text.
    pub fn init() -> (Self, Cmd<Msg>) {
        (Model { scroll: 0 }, Cmd::none())
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::ScrollDown => {
                self.scroll = self.scroll.saturating_add(1).min(last_line());
                Cmd::none()
            }
            Msg::ScrollUp => {
                self.scroll = self.scroll.saturating_sub(1);
                Cmd::none()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // The hint keeps the last line for itself; the text takes the rest.
        let [text_area, hint_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let text = Paragraph::new(help())
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(text, text_area);
        frame.render_widget(Line::from(HINT).dim(), hint_area);
    }

    fn subscriptions(&self, publisher: Publisher) -> Cmd<Msg> {
        match publisher {
            Publisher::Terminal(TerminalEvent::Key(event)) => match event.code {
                // Window navigation
                KeyCode::Char('q') => Cmd::quit(),
                KeyCode::Char('H') => navigation::back(),
                KeyCode::Char('L') => navigation::forward(),
                // Page
                KeyCode::Char('j') => Cmd::msg(Msg::ScrollDown),
                KeyCode::Char('k') => Cmd::msg(Msg::ScrollUp),
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    ScrollDown,
    ScrollUp,
}

/// The help text: what the app is, the vocabulary it uses, and every key.
fn help() -> Text<'static> {
    let mut lines = vec![
        Line::from("Command Line Money".bold()),
        Line::from("A double-entry ledger for the terminal.").dim(),
        Line::from(""),
        Line::from(
            "Money does not come from nothing, and it does not disappear. It \
             moves. Every time it moves, you write down which account it left \
             and which account it reached. The amount that leaves is always the \
             same as the amount that arrives. When the two sides are equal, your \
             books are correct.",
        ),
        Line::from(""),
        heading("WORDS TO KNOW"),
    ];

    lines.extend([
        term("Ledger", "The book that holds all your records."),
        term("Journal", "Every entry in the ledger, newest first."),
        term(
            "Double-entry",
            "Every record has two sides, and both must be equal.",
        ),
        term(
            "Account",
            "A place where money sits, comes from, or goes to.",
        ),
        term(
            "Group",
            "A set of accounts kept together, so they are easy to find.",
        ),
        term(
            "Entry",
            "One move of money: a date, a text, and two or more postings.",
        ),
        term(
            "Posting",
            "One line of an entry: an account, a side, and an amount.",
        ),
        term("Balance", "How much money an account holds now."),
        term("Period", "The stretch of time a balance is added up over."),
        term(
            "Statement",
            "The list of every entry that touched one account.",
        ),
    ]);

    lines.push(Line::from(""));
    lines.push(heading("ACCOUNT KINDS"));
    lines.push(Line::from("Every account is one of five kinds."));

    lines.extend([
        term("Assets", "What you own — cash, money in a bank."),
        term("Liabilities", "What you owe — a card debt, a loan."),
        term(
            "Equity",
            "Your own money: what you started with, plus what you gained.",
        ),
        term(
            "Income",
            "Money coming in. It makes equity bigger — salary, interest.",
        ),
        term(
            "Expenses",
            "Money going out. It makes equity smaller — food, rent.",
        ),
    ]);

    lines.push(Line::from(""));
    lines.push(heading("DEBIT AND CREDIT"));
    lines.push(Line::from(
        "Every posting is on one of two sides. Debit is the left side. Credit \
         is the right side.",
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Debit does not mean plus, and credit does not mean minus. What a side \
         does depends on the kind of the account: a debit makes an asset bigger, \
         but it makes a liability smaller.",
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Each kind has one side that makes it bigger. That side is called the \
         normal balance of the account. Assets and expenses get bigger on the \
         debit side. Liabilities, equity and income get bigger on the credit \
         side.",
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Inside one entry, the debits and the credits must come to the same \
         amount. The app does not save an entry until they do.",
    ));

    lines.push(Line::from(""));
    lines.push(heading("THE SUMMARY"));
    lines.push(Line::from(
        "The accounts screen opens on the summary, which shows two totals. Net \
         worth is what you own minus what you owe. Result is what came in \
         minus what went out.",
    ));
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Each currency is counted on its own. For each one the summary also \
         says whether its debits and its credits are still equal. If they are \
         not, it shows how far off they are, and something was written down \
         wrong.",
    ));

    lines.push(Line::from(""));
    lines.push(heading("KEYS"));
    lines.push(Line::from(
        "Upper case and lower case do different things: J is not j.",
    ));
    lines.extend([
        Line::from("  Anywhere").bold(),
        key("q", "quit the app"),
        key("H / L", "go back / forward through the pages you opened"),
        Line::from("  Home").bold(),
        key("j / k", "move down / up the menu"),
        key("Enter", "open the page you chose"),
        Line::from("  Accounts").bold(),
        key("J / K", "move between the parameters and the lower part"),
        key("j / k", "move up and down inside the part you are in"),
        key("h / l", "change the value of the chosen parameter"),
        key("Enter", "open what you chose, in either part"),
        key("g", "open the group of the account you chose"),
        Line::from("  Account").bold(),
        key("j / k", "move between the fields"),
        key("h / l", "change the kind, the currency or the group"),
        key("Enter", "type in a field, or open the entry you chose"),
        key("s", "save the account"),
        key("J / K", "move between the fields and the statement"),
        Line::from("  Group").bold(),
        key("j / k", "move between the fields"),
        key("Enter", "type in the name or the description"),
        key("s", "save the group"),
        key("J / K", "move between the fields and the accounts"),
        key("m", "move the account you chose to another group"),
        key("r", "take the account you chose out of the group"),
        key("Esc", "undo what you typed, or cancel the move"),
        Line::from("  Journal").bold(),
        key("j / k", "move through the entries, newest first"),
        key("Enter", "open the entry you chose"),
        key("a", "make a new entry"),
        Line::from("  Entry").bold(),
        key("J / K", "move between the header and the postings"),
        key("j / k", "move up and down inside the part you are in"),
        key("Enter", "type in the date, description or amount"),
        key("a", "add a posting"),
        key("x", "remove the posting you chose"),
        key("h / l", "change the account of the posting"),
        key("t", "put the posting on the other side: debit or credit"),
        key("s", "save the entry"),
        key("D", "delete the entry"),
        Line::from("  Help").bold(),
        key("j / k", "scroll up and down"),
    ]);

    Text::from(lines)
}

/// A section heading, set off from the text above it.
fn heading(title: &'static str) -> Line<'static> {
    Line::from(title.bold())
}

/// A defined term and its description, in two columns.
fn term(name: &'static str, description: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw(format!("  {name:TERM_WIDTH$}")).bold(),
        Span::raw(description),
    ])
}

/// A key and what it does, in the same two columns as [`term`].
fn key(keys: &'static str, description: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw(format!("    {keys:KEY_WIDTH$}")).bold(),
        Span::raw(description).dim(),
    ])
}

/// The last line the text can be scrolled to, so it cannot leave the screen.
fn last_line() -> u16 {
    help().lines.len().saturating_sub(1) as u16
}
