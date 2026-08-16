use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Row, Table, TableState};
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};
use time::Date;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use uuid::Uuid;

use crate::app::page::{pane, show, totals};
use crate::app::router::Route;
use crate::app::{Clm, Cmd};
use crate::application::infrastructure::repository::entry::SQLiteEntryRepository;
use crate::application::services::{add_journal_entry, delete_journal_entry, edit_journal_entry};
use crate::application::views::Order;
use crate::application::views::accounts::{self, By, Filters};
use crate::domain::journal::{BalancedPostings, EntryRepository, Magnitude, Posting};
use crate::domain::money::{Currency, Money};
use crate::domain::{AccountId, EntryId, Side};

/// How a date is typed and shown: ISO-8601, as the ledger stores it.
const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

/// Height of the header split: one line per field, plus its border.
const HEADER_HEIGHT: u16 = 4;

/// Width the field names are padded to, so their values line up.
const LABEL_WIDTH: usize = 13;

/// Marks where typing lands, so an empty field still shows the caret.
const CARET: &str = "▌";

/// Stands in for a field that has nothing in it.
const EMPTY: &str = "—";

/// What a date looks like, for when what was typed does not.
const DATE_SHAPE: &str = "Date must look like 2026-03-15.";

#[derive(Debug)]
pub struct Model {
    /// The entry's id, or `None` until it has been recorded.
    id: Option<Uuid>,
    /// The date as typed; it becomes a real date only when the entry is saved.
    date: String,
    description: String,
    postings: Vec<PostingLine>,
    /// Every account, in the order the pickers step through them.
    accounts: Vec<Choice>,
    focus: Focus,
    mode: Mode,
    /// Cursor in the header: `0` is the date, `1` the description.
    field: usize,
    /// Cursor in [`postings`](Self::postings).
    selected: usize,
    /// The account whose posting the cursor should land on, spent the first
    /// time the entry is read — after that the cursor is the reader's.
    from: Option<Uuid>,
    status: Status,
}

/// Which split the keys act on.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Header,
    Postings,
}

/// What the keys currently do.
#[derive(Debug)]
enum Mode {
    /// Moving about the entry.
    Browsing,
    /// Typing into whatever the cursor is on, with the text to put back on Esc.
    Editing { backup: String },
}

/// The last thing that happened, and whether it worked.
#[derive(Debug)]
enum Status {
    Idle,
    Done(String),
    Failed(String),
}

/// One posting being written: an account, a side, and an amount as typed.
#[derive(Debug)]
pub struct PostingLine {
    /// Which of [`Model::accounts`] this posting lands in.
    account: usize,
    side: Side,
    /// The amount as typed — `1234.56`, in the account's own currency.
    amount: String,
}

/// An account a posting can name.
#[derive(Debug)]
pub struct Choice {
    id: Uuid,
    name: String,
    currency: Currency,
}

impl Model {
    /// Opens the entry `id` names, or a blank one to record.
    ///
    /// `from` is the account the reader came from, if they came from one's
    /// statement; the cursor lands on that account's posting.
    pub fn init(id: Option<Uuid>, from: Option<Uuid>) -> (Self, Cmd<Msg>) {
        let model = Model {
            id,
            date: String::new(),
            description: String::new(),
            postings: Vec::new(),
            accounts: Vec::new(),
            focus: Focus::Header,
            mode: Mode::Browsing,
            field: 0,
            selected: 0,
            from,
            status: Status::Idle,
        };

        (model, load(id))
    }

    /// Where the posting that lands in `account` sits, if the entry has one.
    fn posting_of(&self, account: Uuid) -> Option<usize> {
        self.postings.iter().position(|line| {
            self.accounts
                .get(line.account)
                .is_some_and(|choice| choice.id == account)
        })
    }

    /// The text the cursor is on, if it is on any.
    fn buffer(&mut self) -> Option<&mut String> {
        match self.focus {
            Focus::Header if self.field == 0 => Some(&mut self.date),
            Focus::Header => Some(&mut self.description),
            Focus::Postings => self
                .postings
                .get_mut(self.selected)
                .map(|line| &mut line.amount),
        }
    }

    /// The currency of the account a posting names.
    fn currency_of(&self, line: &PostingLine) -> Option<Currency> {
        self.accounts
            .get(line.account)
            .map(|choice| choice.currency)
    }

    /// The postings as amounts, dropping the ones that are not a number yet —
    /// so the totals still add up while one of them is being typed.
    fn amounts(&self, side: Side) -> Vec<Money> {
        let amounts = self
            .postings
            .iter()
            .filter(|line| line.side == side)
            .filter_map(|line| {
                let currency = self.currency_of(line)?;
                Some(Money::new(parse_amount(&line.amount, currency)?, currency))
            });

        totals(amounts)
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::Loaded(Ok(loaded)) => {
                let loaded = *loaded;
                self.accounts = loaded.accounts;
                if let Some(entry) = loaded.entry {
                    self.date = entry.date;
                    self.description = entry.description;
                    self.postings = entry.postings;
                }
                self.selected = self.selected.min(self.postings.len().saturating_sub(1));

                // Coming from an account's statement lands on that account's
                // posting, with the keys already down there so the cursor
                // shows. Spent once: a later reload leaves it where it is.
                if let Some(account) = self.from.take()
                    && let Some(index) = self.posting_of(account)
                {
                    self.selected = index;
                    self.focus = Focus::Postings;
                }

                Cmd::none()
            }
            Msg::Loaded(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            }

            // region: Moving about
            Msg::FocusHeader => {
                self.focus = Focus::Header;
                Cmd::none()
            }
            Msg::FocusPostings => {
                self.focus = Focus::Postings;
                Cmd::none()
            }
            Msg::SelectNext => {
                match self.focus {
                    Focus::Header => self.field = wrapping_next(self.field, 2),
                    Focus::Postings => {
                        self.selected = wrapping_next(self.selected, self.postings.len());
                    }
                }
                Cmd::none()
            }
            Msg::SelectPrev => {
                match self.focus {
                    Focus::Header => self.field = wrapping_prev(self.field, 2),
                    Focus::Postings => {
                        self.selected = wrapping_prev(self.selected, self.postings.len());
                    }
                }
                Cmd::none()
            }
            // endregion

            // region: The postings
            Msg::AddPosting => {
                if self.accounts.is_empty() {
                    self.status = Status::Failed("There are no accounts to post to.".to_string());
                    return Cmd::none();
                }

                // A new posting starts on the side that is currently short, so
                // the usual two-line entry needs no toggling.
                let side = if self.amounts(Side::Debit).is_empty() {
                    Side::Debit
                } else {
                    Side::Credit
                };

                self.postings.push(PostingLine {
                    account: 0,
                    side,
                    amount: String::new(),
                });
                self.focus = Focus::Postings;
                self.selected = self.postings.len() - 1;
                Cmd::none()
            }
            Msg::RemovePosting => {
                if self.selected < self.postings.len() {
                    self.postings.remove(self.selected);
                    self.selected = self.selected.min(self.postings.len().saturating_sub(1));
                }
                Cmd::none()
            }
            Msg::ToggleSide => {
                if let Some(line) = self.postings.get_mut(self.selected) {
                    line.side = match line.side {
                        Side::Debit => Side::Credit,
                        Side::Credit => Side::Debit,
                    };
                }
                Cmd::none()
            }
            Msg::CycleAccount(forward) => {
                let len = self.accounts.len();
                if let Some(line) = self.postings.get_mut(self.selected) {
                    line.account = if forward {
                        wrapping_next(line.account, len)
                    } else {
                        wrapping_prev(line.account, len)
                    };
                }
                Cmd::none()
            }
            // endregion

            // region: Typing
            Msg::Edit => {
                if let Some(backup) = self.buffer().map(|text| text.clone()) {
                    self.mode = Mode::Editing { backup };
                }
                Cmd::none()
            }
            Msg::Typed(character) => {
                if let Some(text) = self.buffer() {
                    text.push(character);
                }
                Cmd::none()
            }
            Msg::Backspaced => {
                if let Some(text) = self.buffer() {
                    text.pop();
                }
                Cmd::none()
            }
            Msg::Committed => {
                self.mode = Mode::Browsing;
                Cmd::none()
            }
            Msg::Cancelled => {
                if let Mode::Editing { backup } = std::mem::replace(&mut self.mode, Mode::Browsing)
                    && let Some(text) = self.buffer()
                {
                    *text = backup;
                }
                Cmd::none()
            }
            // endregion

            // region: Saving and striking out
            Msg::Save => match self.entry() {
                Ok(entry) => save(self.id, entry),
                Err(error) => {
                    self.status = Status::Failed(error);
                    Cmd::none()
                }
            },
            Msg::Saved(Ok(id)) => {
                // A newly recorded entry has a screen of its own; go there.
                if self.id.is_none() {
                    return Cmd::request_route(Route::transaction(id));
                }

                self.status = Status::Done("Saved.".to_string());
                load(self.id)
            }
            Msg::Saved(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            }
            Msg::Delete => match self.id {
                Some(id) => delete(id),
                None => {
                    self.status = Status::Failed("Nothing to delete yet.".to_string());
                    Cmd::none()
                }
            },
            Msg::Struck(Ok(())) => navigation::back(),
            Msg::Struck(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            } // endregion
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let [header_area, postings_area, status_area, hint_area] = Layout::vertical([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.view_header(frame, header_area);
        self.view_postings(frame, postings_area);
        frame.render_widget(self.status_line(), status_area);
        frame.render_widget(Line::from(self.hint()).dim(), hint_area);
    }

    fn subscriptions(&self, publisher: Publisher) -> Cmd<Msg> {
        let Publisher::Terminal(TerminalEvent::Key(event)) = publisher else {
            return Cmd::none();
        };

        // While a field is open every printable key is text, so the ordinary
        // bindings — `q` included — have to stand aside for it.
        if matches!(self.mode, Mode::Editing { .. }) {
            return match event.code {
                KeyCode::Char(character) => Cmd::msg(Msg::Typed(character)),
                KeyCode::Backspace => Cmd::msg(Msg::Backspaced),
                KeyCode::Enter => Cmd::msg(Msg::Committed),
                KeyCode::Esc => Cmd::msg(Msg::Cancelled),
                _ => Cmd::none(),
            };
        }

        match event.code {
            // Window navigation
            KeyCode::Char('q') => Cmd::quit(),
            KeyCode::Char('H') => navigation::back(),
            KeyCode::Char('L') => navigation::forward(),
            // Split navigation
            KeyCode::Char('J') => Cmd::msg(Msg::FocusPostings),
            KeyCode::Char('K') => Cmd::msg(Msg::FocusHeader),
            // Inside the focused split
            KeyCode::Char('j') => Cmd::msg(Msg::SelectNext),
            KeyCode::Char('k') => Cmd::msg(Msg::SelectPrev),
            KeyCode::Char('l') => Cmd::msg(Msg::CycleAccount(true)),
            KeyCode::Char('h') => Cmd::msg(Msg::CycleAccount(false)),
            KeyCode::Enter => Cmd::msg(Msg::Edit),
            // The entry
            KeyCode::Char('a') => Cmd::msg(Msg::AddPosting),
            KeyCode::Char('x') => Cmd::msg(Msg::RemovePosting),
            KeyCode::Char('t') => Cmd::msg(Msg::ToggleSide),
            KeyCode::Char('s') => Cmd::msg(Msg::Save),
            KeyCode::Char('D') => Cmd::msg(Msg::Delete),
            _ => Cmd::none(),
        }
    }
}

// region: View
impl Model {
    /// The upper split: when the entry happened, and what it was.
    fn view_header(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::Header;
        let title = match self.id {
            Some(_) => String::from(" Entry "),
            None => String::from(" New entry "),
        };

        let block = pane(title, focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let typing = matches!(self.mode, Mode::Editing { .. });
        let on = |field: usize| focused && self.field == field;

        let lines = vec![
            field_line("Date", &self.date, on(0), typing),
            field_line("Description", &self.description, on(1), typing),
        ];

        frame.render_widget(Text::from(lines), inner);
    }

    /// The lower split: the postings, with each side's running total in the
    /// title — an entry can only be saved when the two agree.
    fn view_postings(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == Focus::Postings;
        let debits = self.amounts(Side::Debit);
        let credits = self.amounts(Side::Credit);

        let mut block = pane(String::from(" Postings "), focused);
        if !self.postings.is_empty() {
            let balanced = debits == credits;
            let sign = if balanced { "=" } else { "≠" };
            let tally = Line::from(format!(
                " {} {sign} {} ",
                or_nothing(show(&debits)),
                or_nothing(show(&credits))
            ))
            .right_aligned();

            block = block.title(if balanced { tally.green() } else { tally.red() });
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.postings.is_empty() {
            frame.render_widget(Line::from("No postings yet — press a").dim(), inner);
            return;
        }

        let typing = matches!(self.mode, Mode::Editing { .. });
        let rows = self.postings.iter().enumerate().map(|(index, line)| {
            let selected = focused && index == self.selected;
            let account = match self.accounts.get(line.account) {
                Some(choice) => choice.name.clone(),
                None => String::from(EMPTY),
            };
            let currency = self
                .currency_of(line)
                .map(Currency::code)
                .unwrap_or_default();

            let amount = if selected && typing {
                Line::from(vec![
                    Span::raw(line.amount.clone()).bold(),
                    Span::raw(CARET).bold(),
                ])
            } else if line.amount.is_empty() {
                Line::from(EMPTY).dim()
            } else {
                Line::from(line.amount.clone())
            };

            Row::new(vec![
                Cell::from(format!("{} {}", marker(selected), account)),
                Cell::from(side_label(line.side)).dim(),
                Cell::from(currency).dim(),
                Cell::from(amount.right_aligned()),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(5),
                Constraint::Length(16),
            ],
        );

        let mut state = TableState::new().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, inner, &mut state);
    }

    /// The outcome of the last action: green when it worked, red when it did not.
    fn status_line(&self) -> Line<'static> {
        match &self.status {
            Status::Idle => Line::from(""),
            Status::Done(message) => Line::from(message.clone()).green(),
            Status::Failed(message) => Line::from(message.clone()).red(),
        }
    }

    /// The keys that do something in the current mode.
    fn hint(&self) -> &'static str {
        match (&self.mode, self.focus) {
            (Mode::Editing { .. }, _) => "type · Enter to keep it · Esc to put it back",
            (Mode::Browsing, Focus::Header) => {
                "j/k field · Enter type · a posting · s save · D delete · J postings · H back"
            }
            (Mode::Browsing, Focus::Postings) => {
                "j/k · h/l account · t side · Enter amount · a add · x drop · s save · K header"
            }
        }
    }

    /// The entry as the services want it, or what is wrong with it.
    ///
    /// Everything is checked here, in one place, so a save either goes through
    /// or says exactly what stopped it.
    fn entry(&self) -> Result<Draft, String> {
        let date =
            Date::parse(self.date.trim(), &DATE_FORMAT).map_err(|_| DATE_SHAPE.to_string())?;

        let mut postings = Vec::with_capacity(self.postings.len());
        for line in &self.postings {
            let choice = self
                .accounts
                .get(line.account)
                .ok_or_else(|| String::from("A posting has no account."))?;

            let minor = parse_amount(&line.amount, choice.currency).ok_or_else(|| {
                format!(
                    "{} is not an amount in {}.",
                    or_nothing(line.amount.clone()),
                    choice.currency.code()
                )
            })?;

            let amount = Magnitude::new(Money::new(minor, choice.currency))
                .map_err(|_| String::from("Every amount has to be more than zero."))?;

            postings.push(Posting::new(AccountId::from(choice.id), line.side, amount));
        }

        let postings = BalancedPostings::new(postings).map_err(|err| match err {
            crate::domain::journal::BalancedPostingsError::TooFew => {
                String::from("An entry needs at least two postings.")
            }
            crate::domain::journal::BalancedPostingsError::Unbalanced => {
                String::from("Debits and credits have to agree, currency by currency.")
            }
            other => other.to_string(),
        })?;

        Ok(Draft {
            date,
            description: self.description.clone(),
            postings,
        })
    }
}

/// One header field: its name, and its text or the caret being typed at.
fn field_line(label: &str, value: &str, selected: bool, typing: bool) -> Line<'static> {
    let name = Span::raw(format!("{label:<LABEL_WIDTH$}"));

    if selected && typing {
        return Line::from(vec![
            Span::raw("> "),
            name,
            Span::raw(value.to_string()).bold(),
            Span::raw(CARET).bold(),
        ]);
    }

    let value = if value.is_empty() {
        Span::raw(EMPTY).dim()
    } else {
        Span::raw(value.to_string())
    };

    Line::from(vec![
        Span::raw(if selected { "> " } else { "  " }),
        name,
        value,
    ])
}

/// The cursor, or the space it would take.
fn marker(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}

/// How a posting's side is spelled on screen.
fn side_label(side: Side) -> &'static str {
    match side {
        Side::Debit => "debit",
        Side::Credit => "credit",
    }
}

/// A placeholder for an empty string, so a message never trails off.
fn or_nothing(text: String) -> String {
    if text.is_empty() {
        String::from(EMPTY)
    } else {
        text
    }
}

/// Reads a typed amount into minor units of `currency`.
///
/// Accepts `1234.5`, `1,234.50` and `1234`, and refuses anything with more
/// decimals than the currency has — `0.001` of a euro is not a euro amount,
/// and silently rounding it would be worse than saying so.
fn parse_amount(text: &str, currency: Currency) -> Option<i64> {
    let cleaned: String = text
        .chars()
        .filter(|character| !character.is_whitespace() && *character != ',')
        .collect();

    let (whole, fraction) = match cleaned.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (cleaned.as_str(), ""),
    };

    let decimals = usize::from(currency.decimals());
    if fraction.len() > decimals || !fraction.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let whole: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let fraction: i64 = if decimals == 0 {
        0
    } else {
        format!("{fraction:0<decimals$}").parse().ok()?
    };

    whole
        .checked_mul(10i64.pow(currency.decimals().into()))?
        .checked_add(fraction)
}
// endregion

// region: Loading
/// Asks the worker for every account, and for the entry being opened.
fn load(id: Option<Uuid>) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let accounts = match accounts::view(&ctx.db, Filters::default(), By::Name, Order::Asc) {
            Ok(items) => items
                .into_iter()
                .filter_map(|item| {
                    Some(Choice {
                        id: item.id,
                        name: item.name,
                        currency: Currency::try_from(item.currency).ok()?,
                    })
                })
                .collect::<Vec<_>>(),
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        let Some(id) = id else {
            return Msg::Loaded(Ok(Box::new(Loaded {
                entry: None,
                accounts,
            })));
        };

        let entry = match SQLiteEntryRepository::new(&ctx.db).read(EntryId::from(id)) {
            Ok(entry) => entry,
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        let postings = entry
            .postings()
            .clone()
            .into_vec()
            .into_iter()
            .map(|posting| {
                let account = accounts
                    .iter()
                    .position(|choice| choice.id == posting.account_id().to_uuid())
                    .unwrap_or(0);

                PostingLine {
                    account,
                    side: posting.side(),
                    amount: text_amount(posting.amount()),
                }
            })
            .collect();

        let date = entry
            .date()
            .format(&DATE_FORMAT)
            .unwrap_or_else(|_| String::new());

        Msg::Loaded(Ok(Box::new(Loaded {
            entry: Some(Stored {
                date,
                description: entry.description().to_string(),
                postings,
            }),
            accounts,
        })))
    })
}

/// Writes a stored amount the way it is typed: digits and a decimal point, no
/// currency symbol — that is the account's business.
fn text_amount(amount: Magnitude) -> String {
    let money = amount.to_money();
    let decimals = usize::from(money.currency().decimals());
    let scale = 10i64.pow(money.currency().decimals().into());
    let (whole, fraction) = (money.amount() / scale, money.amount() % scale);

    if decimals == 0 {
        return whole.to_string();
    }

    format!("{whole}.{fraction:0decimals$}")
}
// endregion

// region: Writing
/// Records the entry, or saves the corrections to an existing one.
fn save(id: Option<Uuid>, draft: Draft) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let entries = SQLiteEntryRepository::new(&ctx.db);

        let Some(id) = id else {
            let recorded = add_journal_entry::AddJournalEntryService::new(entries)
                .execute(add_journal_entry::AddJournalEntryCommand {
                    date: draft.date,
                    description: draft.description,
                    postings: draft.postings,
                })
                .map(EntryId::to_uuid)
                .map_err(|error| error.to_string());

            return Msg::Saved(recorded);
        };

        let edited = edit_journal_entry::Service::new(entries)
            .execute(edit_journal_entry::Command {
                id: EntryId::from(id),
                date: draft.date,
                description: draft.description,
                postings: draft.postings,
            })
            .map(|()| id)
            .map_err(|error| error.to_string());

        Msg::Saved(edited)
    })
}

/// Strikes the entry from the ledger.
fn delete(id: Uuid) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let struck = delete_journal_entry::Service::new(SQLiteEntryRepository::new(&ctx.db))
            .execute(delete_journal_entry::Command {
                id: EntryId::from(id),
            })
            .map_err(|error| error.to_string());

        Msg::Struck(struck)
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

/// An entry that has passed every check and is ready for a service.
#[derive(Debug)]
pub struct Draft {
    date: Date,
    description: String,
    postings: BalancedPostings,
}

/// What the entry screen loads in one go.
#[derive(Debug)]
pub struct Loaded {
    /// The stored entry, or `None` for one that has not been recorded yet.
    entry: Option<Stored>,
    accounts: Vec<Choice>,
}

/// An entry as it was stored, in the form the screen edits it.
#[derive(Debug)]
pub struct Stored {
    date: String,
    description: String,
    postings: Vec<PostingLine>,
}

#[derive(Debug)]
pub enum Msg {
    Loaded(Result<Box<Loaded>, String>),
    FocusHeader,
    FocusPostings,
    SelectNext,
    SelectPrev,
    AddPosting,
    RemovePosting,
    ToggleSide,
    CycleAccount(bool),
    Edit,
    Typed(char),
    Backspaced,
    Committed,
    Cancelled,
    Save,
    Saved(Result<Uuid, String>),
    Delete,
    Struck(Result<(), String>),
}
