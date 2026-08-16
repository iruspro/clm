use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Row, Table, TableState};
use rusqlite::Connection;
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};
use uuid::Uuid;

use crate::app::page::{pane, show, totals};
use crate::app::router::Route;
use crate::app::{Clm, Cmd};
use crate::application::infrastructure::repository::account::SQLiteAccountRepository;
use crate::application::infrastructure::repository::account_group::SQLiteAccountGroupRepository;
use crate::application::services::{add_account_group, edit_account_group, move_account_to_group};
use crate::application::views::accounts::{self, By, Filters};
use crate::application::views::{Order, groups};
use crate::domain::account::AccountKind;
use crate::domain::money::{Currency, Money};
use crate::domain::{AccountGroupId, AccountId, Name, Side};

/// Height of the group split: one line per field, plus its border.
const INFO_HEIGHT: u16 = 5;

/// Width the field names are padded to, so their values line up.
const LABEL_WIDTH: usize = 13;

/// Width the account kinds are padded to.
const KIND_WIDTH: usize = 10;

/// Marks where typing lands, so an empty field still shows the caret.
const CARET: &str = "▌";

/// Stands in for a field that has nothing in it.
const EMPTY: &str = "—";

/// Title of a group that has not been saved yet.
const UNSAVED: &str = " New group ";

/// The fields of a group, in the order they are drawn.
const FIELDS: [Field; 2] = [Field::Name, Field::Description];

#[derive(Debug)]
pub struct Model {
    /// The group's id, or `None` until it has been saved for the first time.
    id: Option<Uuid>,
    name: String,
    description: String,
    /// The accounts in this group, by name.
    accounts: Vec<Account>,
    /// Every other group — where an account can be moved to.
    destinations: Vec<Destination>,
    mode: Mode,
    /// Cursor in [`FIELDS`].
    field: usize,
    /// Cursor in [`accounts`](Self::accounts).
    selected: usize,
    /// Cursor in [`destinations`](Self::destinations), while one is being picked.
    target: usize,
    status: Status,
}

/// What the keys currently do.
#[derive(Debug)]
enum Mode {
    /// Moving between the group's fields.
    Fields,
    /// Typing into one of them, with the text to put back on Esc.
    Editing { backup: String },
    /// Browsing the group's accounts.
    Accounts,
    /// Choosing which group the selected account moves to.
    Moving,
}

/// A field of the group the cursor can sit on.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Name,
    Description,
}

/// The last thing that happened, and whether it worked.
#[derive(Debug)]
enum Status {
    /// Nothing has happened yet.
    Idle,
    Done(String),
    Failed(String),
}

impl Model {
    /// Opens the group `id` names, or an empty screen for a group that does
    /// not exist yet.
    pub fn init(id: Option<Uuid>) -> (Self, Cmd<Msg>) {
        let model = Model {
            id,
            name: String::new(),
            description: String::new(),
            accounts: Vec::new(),
            destinations: Vec::new(),
            mode: Mode::Fields,
            field: 0,
            selected: 0,
            target: 0,
            status: Status::Idle,
        };

        (model, load(id))
    }

    /// The account the cursor is on, if the group holds any.
    fn current(&self) -> Option<&Account> {
        self.accounts.get(self.selected)
    }

    /// The field the cursor is on.
    fn current_field(&self) -> Field {
        FIELDS[self.field.min(FIELDS.len() - 1)]
    }

    /// The text of a typed-into field.
    fn buffer(&mut self, field: Field) -> &mut String {
        match field {
            Field::Name => &mut self.name,
            Field::Description => &mut self.description,
        }
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::Loaded(Ok(loaded)) => {
                let loaded = *loaded;
                if let Some(group) = loaded.group {
                    self.name = group.name;
                    self.description = group.description;
                }
                self.accounts = loaded.accounts;
                self.destinations = loaded.destinations;
                // A shorter list may have left either cursor past the end.
                self.selected = self.selected.min(self.accounts.len().saturating_sub(1));
                self.target = self.target.min(self.destinations.len().saturating_sub(1));
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
            Msg::FocusAccounts => {
                self.mode = Mode::Accounts;
                Cmd::none()
            }
            Msg::SelectNext => {
                match self.mode {
                    Mode::Accounts => {
                        self.selected = wrapping_next(self.selected, self.accounts.len());
                    }
                    _ => self.field = wrapping_next(self.field, FIELDS.len()),
                }
                Cmd::none()
            }
            Msg::SelectPrev => {
                match self.mode {
                    Mode::Accounts => {
                        self.selected = wrapping_prev(self.selected, self.accounts.len());
                    }
                    _ => self.field = wrapping_prev(self.field, FIELDS.len()),
                }
                Cmd::none()
            }
            // endregion

            // region: Typing
            Msg::Edit => {
                let field = self.current_field();
                let backup = self.buffer(field).clone();
                self.mode = Mode::Editing { backup };
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
                // The service would reject this too; catching it here means the
                // answer is instant and the message names the field.
                if self.name.trim().is_empty() {
                    self.status = Status::Failed("A group needs a name.".to_string());
                    return Cmd::none();
                }

                save(self.id, self.name.clone(), self.description.clone())
            }
            Msg::Saved(Ok(id)) => {
                // A group that has just been created has a route of its own:
                // go there, so the location bar and the history point at the
                // group rather than at the blank screen it was made on. That
                // rebuilds the page, which is also what loads it.
                if self.id.is_none() {
                    return Cmd::request_route(Route::group(id));
                }

                self.status = Status::Done(format!("Saved {}.", self.name));

                load(self.id)
            }
            Msg::Saved(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            }
            // endregion

            // region: Moving an account out
            Msg::StartMove => {
                match (self.current(), self.destinations.is_empty()) {
                    (None, _) => self.status = Status::Failed("No account to move.".to_string()),
                    (Some(_), true) => {
                        self.status = Status::Failed("There is no other group.".to_string());
                    }
                    (Some(_), false) => self.mode = Mode::Moving,
                }
                Cmd::none()
            }
            Msg::TargetNext => {
                self.target = wrapping_next(self.target, self.destinations.len());
                Cmd::none()
            }
            Msg::TargetPrev => {
                self.target = wrapping_prev(self.target, self.destinations.len());
                Cmd::none()
            }
            // A move is made from the account list, and hands the keys back
            // to it either way.
            Msg::ConfirmMove => {
                self.mode = Mode::Accounts;

                match (self.current(), self.destinations.get(self.target)) {
                    (Some(account), Some(destination)) => move_account(
                        account.id,
                        Some(destination.id),
                        format!("Moved {} to {}.", account.name, destination.name),
                    ),
                    _ => Cmd::none(),
                }
            }
            Msg::CancelMove => {
                self.mode = Mode::Accounts;
                Cmd::none()
            }
            Msg::RemoveAccount => match (self.id, self.current()) {
                (Some(_), Some(account)) => move_account(
                    account.id,
                    None,
                    format!("{} left the group.", account.name),
                ),
                _ => {
                    self.status = Status::Failed("No account to remove.".to_string());
                    Cmd::none()
                }
            },
            Msg::Moved(Ok(message)) => {
                self.status = Status::Done(message);
                load(self.id)
            }
            Msg::Moved(Err(error)) => {
                self.status = Status::Failed(error);
                Cmd::none()
            } // endregion
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Two splits, then the outcome of the last action, then the keys.
        let [info_area, list_area, status_area, hint_area] = Layout::vertical([
            Constraint::Length(INFO_HEIGHT),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.view_fields(frame, info_area);
        match self.mode {
            Mode::Moving => self.view_destinations(frame, list_area),
            _ => self.view_accounts(frame, list_area),
        }
        frame.render_widget(self.status_line(), status_area);
        frame.render_widget(Line::from(self.hint()).dim(), hint_area);
    }

    fn subscriptions(&self, publisher: Publisher) -> Cmd<Msg> {
        let Publisher::Terminal(TerminalEvent::Key(event)) = publisher else {
            return Cmd::none();
        };

        // While a field is open every printable key is text, so the ordinary
        // bindings — `q` included — have to stand aside for it.
        match self.mode {
            Mode::Editing { .. } => match event.code {
                KeyCode::Char(character) => Cmd::msg(Msg::Typed(character)),
                KeyCode::Backspace => Cmd::msg(Msg::Deleted),
                KeyCode::Enter => Cmd::msg(Msg::Committed),
                KeyCode::Esc => Cmd::msg(Msg::Cancelled),
                _ => Cmd::none(),
            },
            // A move takes the keys over until it is settled one way or other.
            Mode::Moving => match event.code {
                KeyCode::Char('j') => Cmd::msg(Msg::TargetNext),
                KeyCode::Char('k') => Cmd::msg(Msg::TargetPrev),
                KeyCode::Enter => Cmd::msg(Msg::ConfirmMove),
                KeyCode::Esc => Cmd::msg(Msg::CancelMove),
                _ => Cmd::none(),
            },
            Mode::Fields | Mode::Accounts => match event.code {
                // Window navigation
                KeyCode::Char('q') => Cmd::quit(),
                KeyCode::Char('H') => navigation::back(),
                KeyCode::Char('L') => navigation::forward(),
                // Split navigation
                KeyCode::Char('J') => Cmd::msg(Msg::FocusAccounts),
                KeyCode::Char('K') => Cmd::msg(Msg::FocusFields),
                // Inside the focused split
                KeyCode::Char('j') => Cmd::msg(Msg::SelectNext),
                KeyCode::Char('k') => Cmd::msg(Msg::SelectPrev),
                // Only a field is typed into, so only the upper split answers.
                KeyCode::Enter if matches!(self.mode, Mode::Fields) => Cmd::msg(Msg::Edit),
                KeyCode::Char('s') => Cmd::msg(Msg::Save),
                // The accounts, from either split — they are what the group is.
                KeyCode::Char('m') => Cmd::msg(Msg::StartMove),
                KeyCode::Char('r') => Cmd::msg(Msg::RemoveAccount),
                _ => Cmd::none(),
            },
        }
    }
}

// region: View
impl Model {
    /// The upper split: what the group is, and how much it holds.
    fn view_fields(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.mode, Mode::Fields | Mode::Editing { .. });
        let title = match self.id {
            Some(_) => String::from(" Group "),
            None => String::from(UNSAVED),
        };

        let block = pane(title, focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let on = |field: Field| focused && self.current_field() == field;
        let typing = matches!(self.mode, Mode::Editing { .. });

        let lines = vec![
            text_field("Name", &self.name, on(Field::Name), typing),
            text_field(
                "Description",
                &self.description,
                on(Field::Description),
                typing,
            ),
            read_only("Accounts", &self.accounts.len().to_string()),
        ];

        frame.render_widget(Text::from(lines), inner);
    }

    /// The lower split: the accounts in this group, with the group's totals.
    fn view_accounts(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.mode, Mode::Accounts);
        let totals = totals(self.accounts.iter().map(|account| account.balance));

        let mut block = pane(String::from(" Accounts "), focused);
        if !totals.is_empty() {
            block = block.title(Line::from(format!(" {} ", show(&totals))).right_aligned());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.accounts.is_empty() {
            let empty = match self.id {
                Some(_) => "No accounts in this group",
                None => "Save the group, then move accounts into it",
            };
            frame.render_widget(Line::from(empty).dim(), inner);
            return;
        }

        let amounts: Vec<String> = self
            .accounts
            .iter()
            .map(|account| account.balance.to_string())
            .collect();
        let width = amounts
            .iter()
            .map(|amount| amount.chars().count())
            .max()
            .unwrap_or(0);

        let rows =
            self.accounts
                .iter()
                .zip(&amounts)
                .enumerate()
                .map(|(index, (account, amount))| {
                    Row::new(vec![
                        // The cursor is only drawn while the keys are down
                        // here, the same as the fields above.
                        Cell::from(format!(
                            "{} {}",
                            marker(focused && index == self.selected),
                            account.name
                        )),
                        Cell::from(kind_label(account.kind)).dim(),
                        Cell::from(Line::from(amount.clone()).right_aligned()),
                    ])
                });

        let table = Table::new(
            rows,
            [
                Constraint::Fill(1),
                Constraint::Length(KIND_WIDTH as u16),
                Constraint::Length(width as u16),
            ],
        );

        let mut state = TableState::new().with_selected(Some(self.selected));
        frame.render_stateful_widget(table, inner, &mut state);
    }

    /// The lower split while a move is being made: where the account can go.
    fn view_destinations(&self, frame: &mut Frame, area: Rect) {
        let moving = self
            .current()
            .map(|account| account.name.as_str())
            .unwrap_or_default();

        let block = pane(format!(" Move {moving} to "), true);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = self
            .destinations
            .iter()
            .enumerate()
            .map(|(index, destination)| {
                Row::new(vec![
                    Cell::from(format!(
                        "{} {}",
                        marker(index == self.target),
                        destination.name
                    )),
                    Cell::from(Line::from(count_label(destination.accounts)).right_aligned()).dim(),
                ])
            });

        let table = Table::new(rows, [Constraint::Fill(1), Constraint::Length(12)]);

        let mut state = TableState::new().with_selected(Some(self.target));
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
        match self.mode {
            Mode::Editing { .. } => "type to edit · Enter to keep it · Esc to put it back",
            Mode::Moving => "j/k to choose · Enter to move · Esc to cancel",
            Mode::Accounts => "j/k move · m move out · r remove · K fields · H back · q quit",
            Mode::Fields => "j/k field · Enter type · s save · J accounts · H back · q quit",
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

/// How a group's size is spelled on screen.
fn count_label(accounts: u32) -> String {
    match accounts {
        1 => String::from("1 account"),
        other => format!("{other} accounts"),
    }
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
// endregion

// region: Loading
/// Asks the worker for the group, its accounts, and the other groups.
fn load(id: Option<Uuid>) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let group = match id.map(|id| groups::one(&ctx.db, id)) {
            None => None,
            Some(Ok(group)) => group.map(|group| Info {
                name: group.name,
                description: group.description,
            }),
            Some(Err(error)) => return Msg::Loaded(Err(error.to_string())),
        };

        let accounts = match id {
            Some(id) => match accounts_in(&ctx.db, id) {
                Ok(accounts) => accounts,
                Err(error) => return Msg::Loaded(Err(error)),
            },
            None => Vec::new(),
        };

        let destinations = match groups::view(&ctx.db) {
            Ok(groups) => groups
                .into_iter()
                // A group is not somewhere its own accounts can move to.
                .filter(|group| Some(group.id) != id)
                .map(|group| Destination {
                    id: group.id,
                    name: group.name,
                    accounts: group.account_counts,
                })
                .collect(),
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        Msg::Loaded(Ok(Box::new(Loaded {
            group,
            accounts,
            destinations,
        })))
    })
}

/// Reads the accounts of one group, by name.
fn accounts_in(db: &Connection, id: Uuid) -> Result<Vec<Account>, String> {
    let filters = Filters {
        group_ids: Some(vec![id]),
        ..Filters::default()
    };

    let items = accounts::view(db, filters, By::Name, Order::Asc).map_err(|err| err.to_string())?;

    Ok(items.into_iter().filter_map(row).collect())
}

/// Turns a read-model row into the row this page draws, dropping any account
/// whose stored kind or currency this build does not know.
fn row(item: accounts::ResultItem) -> Option<Account> {
    let kind = AccountKind::try_from(item.kind).ok()?;
    let currency = Currency::try_from(item.currency).ok()?;

    // The read model's balance is debit-positive; credit-normal kinds read the
    // other way round on screen.
    let balance = Money::new(item.balance, currency);
    let balance = match kind.normal_balance() {
        Side::Debit => balance,
        Side::Credit => -balance,
    };

    Some(Account {
        id: item.id,
        name: item.name,
        kind,
        balance,
    })
}
// endregion

// region: Writing
/// Creates the group, or saves the edits to an existing one.
fn save(id: Option<Uuid>, name: String, description: String) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let name = match Name::new(&name) {
            Ok(name) => name,
            Err(error) => return Msg::Saved(Err(error.to_string())),
        };

        let groups = SQLiteAccountGroupRepository::new(&ctx.db);

        let saved = match id {
            None => add_account_group::Service::new(groups)
                .execute(add_account_group::Command { name, description })
                .map(AccountGroupId::to_uuid)
                .map_err(|error| error.to_string()),
            Some(id) => edit_account_group::Service::new(groups)
                .execute(edit_account_group::Command {
                    id: AccountGroupId::from(id),
                    name,
                    description,
                })
                .map(|()| id)
                .map_err(|error| error.to_string()),
        };

        Msg::Saved(saved)
    })
}

/// Repoints one account at `target`, or at no group when it is `None`.
///
/// `message` is what the status line says once it worked; it is built by the
/// caller, which still has the names in hand.
fn move_account(account_id: Uuid, target: Option<Uuid>, message: String) -> Cmd<Msg> {
    Cmd::task(move |ctx| {
        let service = move_account_to_group::Service::new(
            SQLiteAccountRepository::new(&ctx.db),
            SQLiteAccountGroupRepository::new(&ctx.db),
        );

        let moved = service
            .execute(move_account_to_group::Command {
                account_id: AccountId::from(account_id),
                group_id: target.map(AccountGroupId::from),
            })
            .map(|()| message)
            .map_err(|error| error.to_string());

        Msg::Moved(moved)
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

/// What the group screen loads in one go.
#[derive(Debug)]
pub struct Loaded {
    /// The stored group, or `None` for one that has not been saved yet.
    group: Option<Info>,
    accounts: Vec<Account>,
    destinations: Vec<Destination>,
}

/// The group's own fields, as stored.
#[derive(Debug)]
struct Info {
    name: String,
    description: String,
}

/// One account in the group.
#[derive(Debug)]
pub struct Account {
    id: Uuid,
    name: String,
    kind: AccountKind,
    balance: Money,
}

/// Another group, as an account's possible destination.
#[derive(Debug)]
pub struct Destination {
    id: Uuid,
    name: String,
    accounts: u32,
}

#[derive(Debug)]
pub enum Msg {
    Loaded(Result<Box<Loaded>, String>),
    FocusFields,
    FocusAccounts,
    SelectNext,
    SelectPrev,
    Edit,
    Typed(char),
    Deleted,
    Committed,
    Cancelled,
    Save,
    Saved(Result<Uuid, String>),
    StartMove,
    TargetNext,
    TargetPrev,
    ConfirmMove,
    CancelMove,
    RemoveAccount,
    Moved(Result<String, String>),
}
