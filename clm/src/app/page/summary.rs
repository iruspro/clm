//! The summary half of the accounts screen: what the whole ledger adds up to.
//!
//! Not a page of its own — the accounts screen owns the state and asks this
//! module to read it and to draw it.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row, Table, TableState};
use rusqlite::Connection;

use crate::application::views::summary;
use crate::domain::Side;
use crate::domain::account::AccountKind;
use crate::domain::money::{Currency, Money};

/// Width the row labels are padded to, so the amounts line up.
const LABEL_WIDTH: usize = 16;

/// Said of a currency whose debits and credits agree.
const BALANCED: &str = "debits = credits";

/// Said of one whose do not — which means something went in crooked.
const UNBALANCED: &str = "off by";

/// Reads what every kind holds, gathered into one block per currency.
pub fn read(db: &Connection) -> Result<Vec<Book>, String> {
    let items = summary::view(db).map_err(|error| error.to_string())?;

    Ok(books(items))
}

/// How many lines the blocks come to, for whoever has to scroll them.
pub fn height(books: &[Book]) -> usize {
    books.iter().map(|book| book.rows().len()).sum()
}

/// Draws the blocks into `area`, or says the ledger is still empty.
///
/// `line` is the one the reader has scrolled to, and all `area` is asked to
/// do is keep it on screen.
pub fn view(frame: &mut Frame, area: Rect, books: &[Book], line: usize) {
    if books.is_empty() {
        frame.render_widget(Line::from("Nothing in the ledger yet").dim(), area);
        return;
    }

    let rows: Vec<Row> = books.iter().flat_map(Book::rows).collect();
    // The right-hand column carries both the amounts and each currency's
    // balance check, so it has to be wide enough for the longer of them.
    let width = books
        .iter()
        .flat_map(|book| {
            let amounts = book.amounts().map(|amount| amount.to_string()).to_vec();
            amounts.into_iter().chain([book.check()])
        })
        .map(|text| text.chars().count())
        .max()
        .unwrap_or(0);

    let table = Table::new(
        rows,
        [Constraint::Fill(1), Constraint::Length(width as u16 + 2)],
    );

    // Selecting the line rather than setting the offset outright leaves the
    // widget to work out how far to scroll, which is how it never scrolls
    // past the end. Nothing is drawn on the selected line — no highlight
    // style is set — so the summary keeps its plain look.
    let mut state = TableState::new().with_selected(Some(line));
    frame.render_stateful_widget(table, area, &mut state);
}

/// What one currency holds, and what it adds up to.
///
/// Equity has no field of its own: it is what the other four already say, so
/// the block leaves it out. Its postings still reach
/// [`drift`](Self::drift) — a check that skipped a kind would not be one.
#[derive(Debug)]
pub struct Book {
    currency: Currency,
    assets: Money,
    liabilities: Money,
    income: Money,
    expenses: Money,
    /// What the raw postings sum to. Double entry says zero; anything else
    /// means the books do not add up.
    drift: Money,
}

impl Book {
    /// What is left after what is owed — the number the ledger exists to give.
    fn net_worth(&self) -> Money {
        self.assets
            .checked_sub(self.liabilities)
            .unwrap_or(self.assets)
    }

    /// What came in, less what went out.
    fn result(&self) -> Money {
        self.income
            .checked_sub(self.expenses)
            .unwrap_or(self.income)
    }

    /// Every amount the block draws, for sizing the column.
    fn amounts(&self) -> [Money; 6] {
        [
            self.assets,
            self.liabilities,
            self.net_worth(),
            self.income,
            self.expenses,
            self.result(),
        ]
    }

    /// Whether this currency's postings come to zero, in words.
    fn check(&self) -> String {
        if self.drift.is_zero() {
            String::from(BALANCED)
        } else {
            format!("{} {}", UNBALANCED, self.drift)
        }
    }

    /// The block as drawn: the currency, its four kinds, and its two totals.
    fn rows(&self) -> Vec<Row<'static>> {
        let check = Line::from(self.check());
        let check = if self.drift.is_zero() {
            check.dim()
        } else {
            check.red()
        };

        vec![
            // Only the code takes the colour: the check beside it says dim or
            // red for its own reasons, and those outrank a heading.
            Row::new(vec![
                Cell::from(self.currency.code()).bold().magenta(),
                Cell::from(check.right_aligned()),
            ]),
            row("Assets", self.assets),
            row("Liabilities", self.liabilities),
            total("Net worth", self.net_worth()),
            row("Income", self.income),
            row("Expenses", self.expenses),
            total("Result", self.result()),
            Row::new(vec![Cell::from(""), Cell::from("")]),
        ]
    }
}

/// One line of a block: a kind, and what it holds.
fn row(label: &str, amount: Money) -> Row<'static> {
    Row::new(vec![
        Cell::from(format!("  {label:<LABEL_WIDTH$}")),
        Cell::from(Line::from(amount.to_string()).right_aligned()),
    ])
}

/// A line that adds up the ones above it, picked out from them by colour.
fn total(label: &str, amount: Money) -> Row<'static> {
    row(label, amount).bold().magenta()
}

/// Gathers the read model's rows into one block per currency.
///
/// Each kind is turned into the terms it is read in — a liability of `-$500`
/// raw is `$500` owed — while `drift` keeps the raw signed total, because that
/// is the one that has to come to zero.
fn books(items: Vec<summary::ResultItem>) -> Vec<Book> {
    let mut books: Vec<Book> = Vec::new();

    for item in items {
        let (Ok(currency), Ok(kind)) = (
            Currency::try_from(item.currency),
            AccountKind::try_from(item.kind),
        ) else {
            continue;
        };

        let raw = Money::new(item.total, currency);
        let shown = match kind.normal_balance() {
            Side::Debit => raw,
            Side::Credit => -raw,
        };

        let book = match books.iter_mut().find(|book| book.currency == currency) {
            Some(book) => book,
            None => {
                books.push(Book {
                    currency,
                    assets: Money::zero(currency),
                    liabilities: Money::zero(currency),
                    income: Money::zero(currency),
                    expenses: Money::zero(currency),
                    drift: Money::zero(currency),
                });
                books.last_mut().expect("the book just pushed")
            }
        };

        // Equity has no field: it is left out of the block, though never out
        // of the check below.
        if let Some(field) = match kind {
            AccountKind::Asset => Some(&mut book.assets),
            AccountKind::Liability => Some(&mut book.liabilities),
            AccountKind::Income => Some(&mut book.income),
            AccountKind::Expense => Some(&mut book.expenses),
            AccountKind::Equity => None,
        } {
            *field = field.checked_add(shown).unwrap_or(*field);
        }

        book.drift = book.drift.checked_add(raw).unwrap_or(book.drift);
    }

    books.sort_by_key(|book| book.currency.as_u16());
    books
}
