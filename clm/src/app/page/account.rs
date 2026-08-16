use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Row, Table, TableState};
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};
use uuid::Uuid;

use crate::app::page::pane;
use crate::app::router::Route;
use crate::app::{Clm, Cmd};
use crate::application::infrastructure::repository::account::SQLiteAccountRepository;
use crate::application::infrastructure::repository::account_group::SQLiteAccountGroupRepository;
use crate::application::services::{add_account, edit_account, move_account_to_group};
use crate::application::views::accounts::{self, By, Filters};
use crate::application::views::{Order, groups, journal};
use crate::domain::account::AccountKind;
use crate::domain::money::{Currency, Money};
use crate::domain::{AccountGroupId, AccountId, Name, Side};

/// How many statement lines the page asks for.
const LIMIT: u64 = 200;

/// Width the field names are padded to, so their values line up.
const LABEL_WIDTH: usize = 13;

/// Marks where typing lands, so an empty field still shows the caret.
const CARET: &str = "▌";

/// Stands in for a field that has nothing in it.
const EMPTY: &str = "—";

/// How an account with no group is spelled on screen.
const NO_GROUP: &str = "Ungrouped";

/// The fields of an account, in the order they are drawn.
const FIELDS: [Field; 5] = [
    Field::Name,
    Field::Description,
    Field::Kind,
    Field::Currency,
    Field::Group,
];

#[derive(Debug)]
pub struct Model {
    /// The account's id, or `None` until it has been saved for the first time.
    id: Option<Uuid>,
    name: String,
    description: String,
    kind: AccountKind,
    currency: Currency,
    /// Which of [`groups`](Self::groups) the account belongs to; `None` is
    /// ungrouped.
    group: Option<usize>,
    groups: Vec<Group>,
    /// What the account holds, once it has been saved and read back.
    balance: Option<Money>,
    /// What the recent entries moved through this account, newest first.
    statement: Vec<Entry>,
    mode: Mode,
    /// Cursor in [`FIELDS`].
    field: usize,
    /// Cursor in [`statement`](Self::statement).
    selected: usize,
    status: Status,
}

/// What the keys currently do.
#[derive(Debug)]
enum Mode {
    /// Moving between the account's fields.
    Fields,
    /// Typing into one of them, with the text to put back on Esc.
    Editing { backup: String },
    /// Browsing the statement.
    Statement,
}

/// A field of the account the cursor can sit on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Name,
    Description,
    Kind,
    Currency,
    Group,
}

/// The last thing that happened, and whether it worked.
#[derive(Debug)]
enum Status {
    Idle,
    Done(String),
    Failed(String),
}

impl Model {
    /// Opens the account `id` names, or an empty form for one that does not
    /// exist yet.
    pub fn init(id: Option<Uuid>) -> (Self, Cmd<Msg>) {
        let model = Model {
            id,
            name: String::new(),
            description: String::new(),
            kind: AccountKind::Asset,
            currency: Currency::EUR,
            group: None,
            groups: Vec::new(),
            balance: None,
            statement: Vec::new(),
            mode: Mode::Fields,
            field: 0,
            selected: 0,
            status: Status::Idle,
        };

        (model, load(id))
    }

    /// The field the cursor is on.
    fn current_field(&self) -> Field {
        FIELDS[self.field.min(FIELDS.len() - 1)]
    }

    /// The text of a typed-into field.
    fn buffer(&mut self, field: Field) -> &mut String {
        match field {
            Field::Description => &mut self.description,
            _ => &mut self.name,
        }
    }

    /// The group id the form currently points at.
    fn group_id(&self) -> Option<Uuid> {
        self.group
            .and_then(|index| self.groups.get(index))
            .map(|group| group.id)
    }

    /// Steps the selected field's value, `forward` or back.
    ///
    /// Kind and currency are only offered on a new account: postings are
    /// recorded under both, so changing either afterwards would restate history
    /// rather than correct it.
    fn cycle(&mut self, forward: bool) {
        let fixed = self.id.is_some();

        match self.current_field() {
            Field::Kind if fixed => {
                self.status = Status::Failed("Kind is fixed once an account exists.".to_string());
            }
            Field::Currency if fixed => {
                self.status =
                    Status::Failed("Currency is fixed once an account exists.".to_string());
            }
            Field::Kind => self.kind = cycle_kind(self.kind, forward),
            Field::Currency => self.currency = cycle_currency(self.currency, forward),
            Field::Group => self.group = cycle_group(self.group, self.groups.len(), forward),
            // Text fields are typed into, not cycled.
            Field::Name | Field::Description => {}
        }
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::Loaded(Ok(loaded)) => {
                let loaded = *loaded;
                self.groups = loaded.groups;
                if let Some(account) = loaded.account {
                    self.name = account.name;
                    self.description = account.description;
                    self.kind = account.kind;
                    self.currency = account.currency;
                    self.balance = Some(account.balance);
                    self.group = account
                        .group_id
                        .and_then(|id| self.groups.iter().position(|group| group.id == id));
                }
                self.statement = loaded.statement;
                self.selected = self.selected.min(self.statement.len().saturating_sub(1));
                Cmd::none()
            }
            Msg::Loaded(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            }

            // region: Moving about
            Msg::FocusFields => {
                self.mode = Mode::Fields;
                Cmd::none()
            }
            Msg::FocusStatement => {
                self.mode = Mode::Statement;
                Cmd::none()
            }
            Msg::SelectNext => {
                match self.mode {
                    Mode::Statement => {
                        self.selected = wrapping_next(self.selected, self.statement.len());
                    }
                    _ => self.field = wrapping_next(self.field, FIELDS.len()),
                }
                Cmd::none()
            }
            Msg::SelectPrev => {
                match self.mode {
                    Mode::Statement => {
                        self.selected = wrapping_prev(self.selected, self.statement.len());
                    }
                    _ => self.field = wrapping_prev(self.field, FIELDS.len()),
                }
                Cmd::none()
            }
            Msg::CycleNext => {
                self.cycle(true);
                Cmd::none()
            }
            Msg::CyclePrev => {
                self.cycle(false);
                Cmd::none()
            }
            // endregion

            // region: Typing
            Msg::Edit => {
                // The other fields cycle; there is nothing to type into them.
                if let field @ (Field::Name | Field::Description) = self.current_field() {
                    let backup = self.buffer(field).clone();
                    self.mode = Mode::Editing { backup };
                }
                Cmd::none()
            }
            Msg::Typed(character) => {
                let field = self.current_field();
                self.buffer(field).push(character);
                Cmd::none()
            }
            Msg::Deleted => {
                let field = self.current_field();
                self.buffer(field).pop();
                Cmd::none()
            }
            Msg::Committed => {
                self.mode = Mode::Fields;
                Cmd::none()
            }
            Msg::Cancelled => {
                if let Mode::Editing { backup } = std::mem::replace(&mut self.mode, Mode::Fields) {
                    let field = self.current_field();
                    *self.buffer(field) = backup;
                }
                Cmd::none()
            }
            // endregion

            // region: Saving
            Msg::Save => {
                if self.name.trim().is_empty() {
                    self.status = Status::Failed("An account needs a name.".to_string());
                    return Cmd::none();
                }

                save(
                    self.id,
                    self.name.clone(),
                    self.description.clone(),
                    self.kind,
                    self.currency,
                    self.group_id(),
                )
            }
            Msg::Saved(Ok(id)) => {
                // A new account has a screen of its own: go there, which is
                // also what loads its balance and statement.
                if self.id.is_none() {
                    return Cmd::request_route(Route::account(id));
                }

                self.status = Status::Done(format!("Saved {}.", self.name));
                load(self.id)
            }
            Msg::Saved(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            } // endregion
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let [fields_area, statement_area, status_area, hint_area] = Layout::vertical([
            Constraint::Length(self.fields_height()),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.view_fields(frame, fields_area);
        self.view_statement(frame, statement_area);
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
                KeyCode::Backspace => Cmd::msg(Msg::Deleted),
                KeyCode::Enter => Cmd::msg(Msg::Committed),
                KeyCode::Esc => Cmd::msg(Msg::Cancelled),
                _ => Cmd::none(),
            };
        }

        // Only the fields are changed, and only from the split they are in:
        // the statement is read, and a key meant for a field would otherwise
        // reach one the cursor left behind up there.
        let fields = matches!(self.mode, Mode::Fields);

        match event.code {
            // Window navigation
            KeyCode::Char('q') => Cmd::quit(),
            KeyCode::Char('H') => navigation::back(),
            KeyCode::Char('L') => navigation::forward(),
            // Split navigation
            KeyCode::Char('J') => Cmd::msg(Msg::FocusStatement),
            KeyCode::Char('K') => Cmd::msg(Msg::FocusFields),
            // Inside the focused split
            KeyCode::Char('j') => Cmd::msg(Msg::SelectNext),
            KeyCode::Char('k') => Cmd::msg(Msg::SelectPrev),
            KeyCode::Char('l') if fields => Cmd::msg(Msg::CycleNext),
            KeyCode::Char('h') if fields => Cmd::msg(Msg::CyclePrev),
            KeyCode::Enter if fields => Cmd::msg(Msg::Edit),
            // From a statement line to the entry behind it, which opens on
            // the posting that touched this account.
            KeyCode::Enter => match (self.id, self.statement.get(self.selected)) {
                (Some(account), Some(entry)) => {
                    Cmd::request_route(Route::posting(entry.id, account))
                }
                _ => Cmd::none(),
            },
            KeyCode::Char('s') => Cmd::msg(Msg::Save),
            _ => Cmd::none(),
        }
    }
}

// region: View
impl Model {
    /// One row per field, plus the balance of a saved account, plus the border.
    fn fields_height(&self) -> u16 {
        let balance = u16::from(self.balance.is_some());
        FIELDS.len() as u16 + balance + 2
    }

    /// The upper split: what the account is.
    fn view_fields(&self, frame: &mut Frame, area: Rect) {
        let focused = !matches!(self.mode, Mode::Statement);
        let title = match self.id {
            Some(_) => String::from(" Account "),
            None => String::from(" New account "),
        };

        let block = pane(title, focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let on = |field: Field| focused && self.current_field() == field;
        let typing = matches!(self.mode, Mode::Editing { .. });
        let fixed = self.id.is_some();

        let mut lines = vec![
            text_field("Name", &self.name, on(Field::Name), typing),
            text_field(
                "Description",
                &self.description,
                on(Field::Description),
                typing,
            ),
            // A saved account's kind and currency can only be read, so they are
            // drawn without the arrows that would suggest otherwise.
            choice_field(
                "Kind",
                kind_label(self.kind),
                on(Field::Kind) && !fixed,
                on(Field::Kind),
            ),
            choice_field(
                "Currency",
                self.currency.code(),
                on(Field::Currency) && !fixed,
                on(Field::Currency),
            ),
            choice_field(
                "Group",
                self.group_label(),
                on(Field::Group),
                on(Field::Group),
            ),
        ];

        if let Some(balance) = self.balance {
            lines.push(read_only("Balance", &balance.to_string()));
        }

        frame.render_widget(Text::from(lines), inner);
    }

    /// The lower split: what the recent entries moved through this account.
    fn view_statement(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.mode, Mode::Statement);
        let block = pane(String::from(" Statement "), focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.statement.is_empty() {
            let empty = match self.id {
                Some(_) => "Nothing posted to this account",
                None => "Save the account, then post to it",
            };
            frame.render_widget(Line::from(empty).dim(), inner);
            return;
        }

        let amounts: Vec<String> = self
            .statement
            .iter()
            .map(|entry| entry.amount.to_string())
            .collect();
        let width = amounts
            .iter()
            .map(|amount| amount.chars().count())
            .max()
            .unwrap_or(0);

        let rows =
            self.statement
                .iter()
                .zip(&amounts)
                .enumerate()
                .map(|(index, (entry, amount))| {
                    let description = if entry.description.is_empty() {
                        Cell::from(EMPTY).dim()
                    } else {
                        Cell::from(entry.description.clone())
                    };

                    Row::new(vec![
                        // The cursor is only drawn while the keys are down
                        // here, the same as the fields above.
                        Cell::from(format!(
                            "{} {}",
                            marker(focused && index == self.selected),
                            entry.date
                        ))
                        .dim(),
                        description,
                        Cell::from(Line::from(amount.clone()).right_aligned()),
                    ])
                });

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Fill(1),
                Constraint::Length(width as u16),
            ],
        );

        let mut state = TableState::new().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, inner, &mut state);
    }

    /// How the chosen group is spelled on screen.
    fn group_label(&self) -> &str {
        match self.group.and_then(|index| self.groups.get(index)) {
            Some(group) => group.name.as_str(),
            None => NO_GROUP,
        }
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
        match self.mode {
            Mode::Editing { .. } => "type to edit · Enter to keep it · Esc to put it back",
            Mode::Statement => "j/k move · Enter the entry · K fields · H back · q quit",
            Mode::Fields => {
                "j/k field · h/l change · Enter type · s save · J statement · H back · q quit"
            }
        }
    }
}

/// A field that is typed into: its text, and the caret while it is open.
fn text_field(label: &str, value: &str, selected: bool, typing: bool) -> Line<'static> {
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

    Line::from(vec![Span::raw(marker_wide(selected)), name, value])
}

/// A field that is cycled: `< Asset >` when the arrows apply to it.
fn choice_field(label: &str, value: &str, cyclable: bool, selected: bool) -> Line<'static> {
    let name = Span::raw(format!("{label:<LABEL_WIDTH$}"));

    if cyclable {
        return Line::from(vec![
            Span::raw("> "),
            name,
            Span::raw("< ").dim(),
            Span::raw(value.to_string()).bold(),
            Span::raw(" >").dim(),
        ]);
    }

    Line::from(vec![
        Span::raw(marker_wide(selected)),
        name,
        Span::raw(value.to_string()),
    ])
}

/// A field that is only ever read.
fn read_only(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::raw(format!("{label:<LABEL_WIDTH$}")),
        Span::raw(value.to_string()),
    ])
}

/// The cursor, or the space it would take.
fn marker(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}

/// The cursor and the space after it.
fn marker_wide(selected: bool) -> &'static str {
    if selected { "> " } else { "  " }
}

/// How an account kind is spelled on screen, in the singular.
fn kind_label(kind: AccountKind) -> &'static str {
    match kind {
        AccountKind::Asset => "Asset",
        AccountKind::Liability => "Liability",
        AccountKind::Equity => "Equity",
        AccountKind::Income => "Income",
        AccountKind::Expense => "Expense",
    }
}

/// The next kind round the five, or the previous one.
fn cycle_kind(kind: AccountKind, forward: bool) -> AccountKind {
    let order = [
        AccountKind::Asset,
        AccountKind::Liability,
        AccountKind::Equity,
        AccountKind::Income,
        AccountKind::Expense,
    ];
    let at = order.iter().position(|k| *k == kind).unwrap_or(0);
    let next = if forward {
        wrapping_next(at, order.len())
    } else {
        wrapping_prev(at, order.len())
    };

    order[next]
}

/// The next currency round the supported ones, or the previous one.
fn cycle_currency(currency: Currency, forward: bool) -> Currency {
    let at = Currency::ALL
        .iter()
        .position(|c| *c == currency)
        .unwrap_or(0);
    let next = if forward {
        wrapping_next(at, Currency::ALL.len())
    } else {
        wrapping_prev(at, Currency::ALL.len())
    };

    Currency::ALL[next]
}

/// The next group round the list, with "ungrouped" as one of the options.
fn cycle_group(group: Option<usize>, len: usize, forward: bool) -> Option<usize> {
    // Ungrouped sits at the front, so there are `len + 1` places to be.
    let at = group.map_or(0, |index| index + 1);
    let next = if forward {
        wrapping_next(at, len + 1)
    } else {
        wrapping_prev(at, len + 1)
    };

    next.checked_sub(1)
}
// endregion

// region: Loading
/// Asks the worker for the account, its statement, and every group.
fn load(id: Option<Uuid>) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let groups = match groups::view(&ctx.db) {
            Ok(groups) => groups
                .into_iter()
                .map(|group| Group {
                    id: group.id,
                    name: group.name,
                })
                .collect(),
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        let Some(id) = id else {
            return Msg::Loaded(Ok(Box::new(Loaded {
                account: None,
                statement: Vec::new(),
                groups,
            })));
        };

        let filters = Filters {
            ids: Some(vec![id]),
            ..Filters::default()
        };
        let account = match accounts::view(&ctx.db, filters, By::Name, Order::Asc) {
            Ok(items) => items.into_iter().next().and_then(row),
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        let kind = account.as_ref().map(|account: &Info| account.kind);
        let currency = account.as_ref().map(|account: &Info| account.currency);

        let statement = match journal::statement(&ctx.db, id, LIMIT) {
            Ok(lines) => match (kind, currency) {
                (Some(kind), Some(currency)) => lines
                    .into_iter()
                    .map(|line| Entry::new(line, kind, currency))
                    .collect(),
                _ => Vec::new(),
            },
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        Msg::Loaded(Ok(Box::new(Loaded {
            account,
            statement,
            groups,
        })))
    })
}

/// Turns the read model's row into the account this page shows, dropping one
/// whose stored kind or currency this build does not know.
fn row(item: accounts::ResultItem) -> Option<Info> {
    let kind = AccountKind::try_from(item.kind).ok()?;
    let currency = Currency::try_from(item.currency).ok()?;

    Some(Info {
        name: item.name,
        description: item.description,
        kind,
        currency,
        group_id: item.group_id,
        balance: in_account_terms(Money::new(item.balance, currency), kind),
    })
}

/// Flips a debit-positive amount into what the account itself calls positive.
///
/// The read model counts debits up; a liability, equity or income account grows
/// on the credit side, so owing $500 reads as `$500`, not `-$500`.
fn in_account_terms(amount: Money, kind: AccountKind) -> Money {
    match kind.normal_balance() {
        Side::Debit => amount,
        Side::Credit => -amount,
    }
}
// endregion

// region: Writing
/// Creates the account, or saves the edits to an existing one.
///
/// Editing writes twice: the name and description are the account's own, while
/// its group is a move — the same operation the group screen performs.
fn save(
    id: Option<Uuid>,
    name: String,
    description: String,
    kind: AccountKind,
    currency: Currency,
    group_id: Option<Uuid>,
) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let name = match Name::new(&name) {
            Ok(name) => name,
            Err(error) => return Msg::Saved(Err(error.to_string())),
        };
        let group_id = group_id.map(AccountGroupId::from);

        let accounts = SQLiteAccountRepository::new(&ctx.db);

        let Some(id) = id else {
            let created = add_account::Service::new(accounts)
                .execute(add_account::Command {
                    kind,
                    currency,
                    name,
                    description,
                    group_id,
                })
                .map(AccountId::to_uuid)
                .map_err(|error| error.to_string());

            return Msg::Saved(created);
        };

        let edited = edit_account::Service::new(accounts)
            .execute(edit_account::Command {
                id: AccountId::from(id),
                name,
                description,
            })
            .map_err(|error| error.to_string());

        if let Err(error) = edited {
            return Msg::Saved(Err(error));
        }

        let moved = move_account_to_group::Service::new(
            SQLiteAccountRepository::new(&ctx.db),
            SQLiteAccountGroupRepository::new(&ctx.db),
        )
        .execute(move_account_to_group::Command {
            account_id: AccountId::from(id),
            group_id,
        })
        .map(|()| id)
        .map_err(|error| error.to_string());

        Msg::Saved(moved)
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

/// What the account screen loads in one go.
#[derive(Debug)]
pub struct Loaded {
    /// The stored account, or `None` for one that has not been saved yet.
    account: Option<Info>,
    statement: Vec<Entry>,
    groups: Vec<Group>,
}

/// The account's own fields, as stored.
#[derive(Debug)]
struct Info {
    name: String,
    description: String,
    kind: AccountKind,
    currency: Currency,
    group_id: Option<Uuid>,
    balance: Money,
}

/// One line of the statement.
#[derive(Debug)]
pub struct Entry {
    /// The entry this line came from — where the line leads.
    id: Uuid,
    date: String,
    description: String,
    /// What the entry did to this account, in the account's own terms.
    amount: Money,
}

impl Entry {
    /// Reads one statement line, in the terms the account keeps its balance in.
    fn new(item: journal::StatementItem, kind: AccountKind, currency: Currency) -> Self {
        Entry {
            id: item.entry_id,
            date: item.date,
            description: item.description,
            amount: in_account_terms(Money::new(item.amount, currency), kind),
        }
    }
}

/// A group the account can belong to.
#[derive(Debug)]
pub struct Group {
    id: Uuid,
    name: String,
}

#[derive(Debug)]
pub enum Msg {
    Loaded(Result<Box<Loaded>, String>),
    FocusFields,
    FocusStatement,
    SelectNext,
    SelectPrev,
    CycleNext,
    CyclePrev,
    Edit,
    Typed(char),
    Deleted,
    Committed,
    Cancelled,
    Save,
    Saved(Result<Uuid, String>),
}
