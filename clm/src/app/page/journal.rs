use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};
use uuid::Uuid;

use crate::app::page::{pane, show, totals};
use crate::app::router::Route;
use crate::app::{Clm, Cmd};
use crate::application::views::journal;
use crate::domain::Side;
use crate::domain::money::{Currency, Money};

/// How many entries the list asks for. The journal only grows, so the page
/// takes the most recent slice of it rather than all of it.
const LIMIT: u64 = 200;

/// Height of the postings split: six rows, plus its border.
const POSTINGS_HEIGHT: u16 = 8;

/// Width the dates are drawn in: `YYYY-MM-DD`.
const DATE_WIDTH: u16 = 10;

/// Stands in for an entry that says nothing about itself.
const NO_DESCRIPTION: &str = "—";

/// The keys this page answers to, spelled out under it.
const HINT: &str = "j/k to move · H to go back · q to quit";

#[derive(Debug)]
pub struct Model {
    /// The most recent entries, newest first.
    entries: Vec<Entry>,
    /// The postings of the entry the cursor is on.
    postings: Vec<Posting>,
    /// Cursor in [`entries`](Self::entries).
    selected: usize,
    /// What went wrong reading the journal, if anything did.
    error: Option<String>,
}

impl Model {
    /// Opens the journal on its most recent entry.
    pub fn init() -> (Self, Cmd<Msg>) {
        let model = Model {
            entries: Vec::new(),
            postings: Vec::new(),
            selected: 0,
            error: None,
        };

        (model, load())
    }

    /// The entry the cursor is on, if the journal has any.
    fn current(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    /// Asks for the postings of whatever the cursor is now on.
    fn open_current(&self) -> Cmd<Msg> {
        match self.current() {
            Some(entry) => load_postings(entry.id),
            None => Cmd::none(),
        }
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::LoadedEntries(Ok(entries)) => {
                self.entries = entries;
                self.error = None;
                // A shorter journal may have left the cursor past the end.
                self.selected = self.selected.min(self.entries.len().saturating_sub(1));

                self.open_current()
            }
            Msg::LoadedPostings(Ok(postings)) => {
                self.postings = postings;
                Cmd::none()
            }
            Msg::LoadedEntries(Err(error)) | Msg::LoadedPostings(Err(error)) => {
                self.error = Some(error);
                Cmd::none()
            }
            Msg::SelectNext => {
                self.selected = wrapping_next(self.selected, self.entries.len());
                self.open_current()
            }
            Msg::SelectPrev => {
                self.selected = wrapping_prev(self.selected, self.entries.len());
                self.open_current()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let [entries_area, postings_area, hint_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(POSTINGS_HEIGHT),
            Constraint::Length(1),
        ])
        .areas(area);

        self.view_entries(frame, entries_area);
        self.view_postings(frame, postings_area);
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
                KeyCode::Char('j') => Cmd::msg(Msg::SelectNext),
                KeyCode::Char('k') => Cmd::msg(Msg::SelectPrev),
                KeyCode::Enter => match self.current() {
                    Some(entry) => Cmd::request_route(Route::transaction(entry.id)),
                    None => Cmd::none(),
                },
                KeyCode::Char('a') => Cmd::request_route(Route::new_transaction()),
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
}

// region: View
impl Model {
    /// The upper split: one row per entry, newest first.
    fn view_entries(&self, frame: &mut Frame, area: Rect) {
        let block = pane(String::from(" Journal "), true);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(error) = &self.error {
            frame.render_widget(Line::from(error.clone()).red(), inner);
            return;
        }

        if self.entries.is_empty() {
            frame.render_widget(Line::from("No transactions").dim(), inner);
            return;
        }

        let rows = self.entries.iter().enumerate().map(|(index, entry)| {
            let description = if entry.description.is_empty() {
                Cell::from(NO_DESCRIPTION).dim()
            } else {
                Cell::from(entry.description.clone())
            };

            Row::new(vec![
                Cell::from(format!("{} {}", marker(index == self.selected), entry.date)).dim(),
                description,
                Cell::from(Line::from(count_label(entry.posting_counts)).right_aligned()).dim(),
            ])
        });

        let table = Table::new(
            rows,
            [
                // The marker and its space ride in the date column.
                Constraint::Length(DATE_WIDTH + 2),
                Constraint::Fill(1),
                Constraint::Length(12),
            ],
        );

        let mut state = TableState::new().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, inner, &mut state);
    }

    /// The lower split: what the selected entry moves, laid out as a ledger
    /// does it — debits on the left, credits on the right.
    ///
    /// The title carries the total of one side, which for a balanced entry is
    /// also the total of the other.
    fn view_postings(&self, frame: &mut Frame, area: Rect) {
        let (debits, credits): (Vec<&Posting>, Vec<&Posting>) = self
            .postings
            .iter()
            .partition(|posting| posting.side == Side::Debit);

        let total = totals(debits.iter().map(|posting| posting.amount));

        let mut block = pane(String::from(" Postings "), false);
        if !total.is_empty() {
            block = block.title(Line::from(format!(" {} ", show(&total))).right_aligned());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.postings.is_empty() {
            frame.render_widget(Line::from("Nothing posted").dim(), inner);
            return;
        }

        let [debit_area, credit_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(inner);

        // The rule down the middle is the stem of the T the two sides hang off.
        let divider = Block::new()
            .borders(Borders::LEFT)
            .border_style(Style::new().dark_gray());
        let credit_inner = divider.inner(credit_area);
        frame.render_widget(divider, credit_area);

        frame.render_widget(side_table(&debits, "Debit"), debit_area);
        frame.render_widget(side_table(&credits, "Credit"), credit_inner);
    }
}

/// One side of an entry: its accounts, and what each of them moved.
fn side_table<'a>(postings: &[&Posting], header: &'a str) -> Table<'a> {
    let width = postings
        .iter()
        .map(|posting| posting.amount.to_string().chars().count())
        .max()
        .unwrap_or(0);

    let rows: Vec<Row<'a>> = postings
        .iter()
        .map(|posting| {
            Row::new(vec![
                Cell::from(format!("  {}", posting.account)),
                Cell::from(Line::from(posting.amount.to_string()).right_aligned()),
            ])
        })
        .collect();

    Table::new(
        rows,
        [Constraint::Fill(1), Constraint::Length(width as u16 + 1)],
    )
    .header(Row::new(vec![Cell::from(format!("  {header}")), Cell::from("")]).dim())
}

/// The cursor, or the space it would take.
fn marker(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}

/// How the size of an entry is spelled on screen.
fn count_label(postings: u32) -> String {
    match postings {
        1 => String::from("1 posting"),
        other => format!("{other} postings"),
    }
}

// endregion

// region: Loading
/// Asks the worker for the most recent entries.
fn load() -> Cmd<Msg> {
    Cmd::task(|ctx| {
        let entries = journal::view(&ctx.db, LIMIT)
            .map(|items| items.into_iter().map(Entry::from).collect())
            .map_err(|error| error.to_string());

        Msg::LoadedEntries(entries)
    })
}

/// Asks the worker for the postings of one entry.
fn load_postings(entry_id: Uuid) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let postings = journal::postings(&ctx.db, entry_id)
            .map(|items| items.into_iter().filter_map(Posting::new).collect())
            .map_err(|error| error.to_string());

        Msg::LoadedPostings(postings)
    })
}
// endregion

/// The row after `current` in a list of `len` rows, wrapping round the end.
fn wrapping_next(current: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (current + 1) % len }
}

/// The row before `current` in a list of `len` rows, wrapping round the start.
fn wrapping_prev(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (current + len - 1) % len
    }
}

/// One entry in the list.
#[derive(Debug)]
pub struct Entry {
    id: Uuid,
    date: String,
    description: String,
    posting_counts: u32,
}

impl From<journal::EntryItem> for Entry {
    fn from(item: journal::EntryItem) -> Self {
        Entry {
            id: item.id,
            date: item.date,
            description: item.description,
            posting_counts: item.posting_counts,
        }
    }
}

/// One posting of the selected entry.
#[derive(Debug)]
pub struct Posting {
    account: String,
    side: Side,
    /// The amount as a magnitude: which way it goes is [`side`](Self::side).
    amount: Money,
}

impl Posting {
    /// Splits a stored posting into a side and an amount, dropping any posting
    /// whose currency this build does not know.
    ///
    /// The sign says which side the posting is on, so the amount itself is
    /// shown without it — a credit of `-$50` reads as `$50 credit`.
    fn new(item: journal::PostingItem) -> Option<Self> {
        let currency = Currency::try_from(item.currency).ok()?;

        let side = if item.amount < 0 {
            Side::Credit
        } else {
            Side::Debit
        };

        Some(Posting {
            account: item.account,
            side,
            amount: Money::new(item.amount.abs(), currency),
        })
    }
}

#[derive(Debug)]
pub enum Msg {
    LoadedEntries(Result<Vec<Entry>, String>),
    LoadedPostings(Result<Vec<Posting>, String>),
    SelectNext,
    SelectPrev,
}
