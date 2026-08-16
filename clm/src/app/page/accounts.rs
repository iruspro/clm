use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, Row, Table, TableState};
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};
use time::{Month, OffsetDateTime};
use uuid::Uuid;

use crate::app::page::summary::{self, Book};
use crate::app::page::{pane, show, totals};
use crate::app::router::{Route, View};
use crate::app::{Clm, Cmd};
use crate::application::views::Order;
use crate::application::views::accounts::{self, By, Dates, Filters};
use crate::domain::Side;
use crate::domain::account::AccountKind;
use crate::domain::money::{Currency, Money};

/// Width the parameter names are padded to, so their values line up.
const PARAM_LABEL_WIDTH: usize = 10;

/// The header accounts that belong to no group are drawn under.
const NO_GROUP: &str = "Ungrouped";

/// The keys this page answers to, spelled out under it.
const HINT: &str = "J/K splits · j/k move · h/l change · Enter open · g group · H back · q quit";

#[derive(Debug)]
pub struct Model {
    params: Params,
    /// Whatever the view asked for, once the worker has read it.
    content: Content,
    /// What went wrong reading the ledger, if anything did.
    error: Option<String>,
    mode: Mode,
    /// Cursor in the parameters split; indexes [`Params::rows`].
    param: usize,
    /// Cursor in the accounts list.
    selected: usize,
    /// The line of the summary the reader has scrolled to. It has no cursor
    /// of its own, so this is only ever asked to stay on screen.
    scroll: usize,
}

#[derive(Debug)]
pub struct Params {
    /// The summary of the whole ledger, or one kind's accounts.
    view: View,
    by: By,
    order: Order,
    /// How much of the journal the balances are summed over.
    period: Period,
    /// The year [`Period::Year`] and [`Period::Month`] point at. Kept while
    /// the period is something else, so switching back lands where it left.
    year: i32,
    /// The month [`Period::Month`] points at, kept the same way.
    month: Month,
}

/// What the lower split holds.
#[derive(Debug)]
pub enum Content {
    /// One block per currency.
    Summary(Vec<Book>),
    /// The accounts of one kind, gathered into their groups.
    Accounts(Vec<Account>),
}

/// Which split the keys act on.
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Params,
    Content,
}

/// How much of the journal the balances are summed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    /// Every posting, whenever it was made.
    AllTime,
    /// One calendar year of them.
    Year,
    /// One calendar month of one year.
    Month,
}

/// One row of the parameters split: a value the keys change, or a page they
/// open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Param {
    View,
    By,
    Order,
    Period,
    Year,
    Month,
    NewAccount,
    NewGroup,
    Home,
}

/// Which way `l` and `h` step through a parameter's values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Next,
    Prev,
}

impl Model {
    pub fn init(view: View) -> (Self, Cmd<Msg>) {
        // The pickers start on today. UTC is near enough for choosing which
        // year or month to look at, and asks nothing of the host.
        let today = OffsetDateTime::now_utc().date();

        let params = Params {
            view,
            by: By::Balance,
            order: Order::Desc,
            period: Period::AllTime,
            year: today.year(),
            month: today.month(),
        };
        // Built before the model, which then takes ownership of `params`.
        let load = load(&params);

        (
            Model {
                content: empty(view),
                params,
                error: None,
                mode: Mode::Content,
                param: 0,
                selected: 0,
                scroll: 0,
            },
            load,
        )
    }

    /// The accounts on screen, or nothing when the summary is up instead.
    fn accounts(&self) -> &[Account] {
        match &self.content {
            Content::Accounts(accounts) => accounts,
            Content::Summary(_) => &[],
        }
    }

    /// Opens the page the parameters cursor sits on. The rows above them are
    /// values, which `h` and `l` change instead.
    fn open(&self) -> Cmd<Msg> {
        match self.params.rows().get(self.param) {
            Some(Param::NewAccount) => Cmd::request_route(Route::new_account()),
            Some(Param::NewGroup) => Cmd::request_route(Route::new_group()),
            Some(Param::Home) => Cmd::request_route(Route::home()),
            _ => Cmd::none(),
        }
    }

    /// Steps the parameter under the cursor to its next value, and asks for
    /// the accounts that value selects.
    fn cycle(&mut self, step: Step) -> Cmd<Msg> {
        // Nothing in the lower split cycles.
        if self.mode == Mode::Content {
            return Cmd::none();
        }

        match self.params.rows().get(self.param) {
            Some(Param::View) => self.params.view = cycle_view(self.params.view, step),
            Some(Param::By) => self.params.by = cycle_by(self.params.by, step),
            Some(Param::Order) => self.params.order = flip(self.params.order),
            Some(Param::Period) => self.params.period = cycle_period(self.params.period, step),
            Some(Param::Year) => {
                self.params.year = match step {
                    Step::Next => self.params.year.saturating_add(1),
                    Step::Prev => self.params.year.saturating_sub(1),
                }
            }
            Some(Param::Month) => {
                self.params.month = match step {
                    Step::Next => self.params.month.next(),
                    Step::Prev => self.params.month.previous(),
                }
            }
            // The rows that open a page answer to Enter, not to h and l.
            _ => return Cmd::none(),
        }

        load(&self.params)
    }
}

impl Params {
    /// The rows the parameters split draws, in order.
    ///
    /// Only the rows that mean something are there: the summary is not a
    /// sorted list and covers the whole ledger, so it takes none of them; the
    /// year and the month appear only when the period asks for them.
    fn rows(&self) -> Vec<Param> {
        let mut rows = vec![Param::View];

        if let View::Kind(_) = self.view {
            rows.extend([Param::By, Param::Order, Param::Period]);

            match self.period {
                Period::AllTime => {}
                Period::Year => rows.push(Param::Year),
                Period::Month => rows.extend([Param::Year, Param::Month]),
            }
        }

        rows.extend([Param::NewAccount, Param::NewGroup, Param::Home]);
        rows
    }

    /// The days the period covers, or `None` when it covers all of them.
    fn dates(&self) -> Option<Dates> {
        match self.period {
            Period::AllTime => None,
            Period::Year => Some(Dates {
                from: day(self.year, Month::January),
                before: day(self.year + 1, Month::January),
            }),
            Period::Month => {
                // December is the one month whose successor is next year's.
                let next = match self.month {
                    Month::December => day(self.year + 1, Month::January),
                    month => day(self.year, month.next()),
                };

                Some(Dates {
                    from: day(self.year, self.month),
                    before: next,
                })
            }
        }
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            // The parameters split keeps no cursor of its own: coming back to
            // it always lands on the first row.
            Msg::FocusNext => {
                self.mode = Mode::Content;
                Cmd::none()
            }
            Msg::FocusPrev => {
                self.mode = Mode::Params;
                self.param = 0;
                Cmd::none()
            }
            // The list has a cursor to move; the summary has none, so it
            // scrolls instead — and scrolling stops at the ends rather than
            // wrapping, which would lose the reader.
            Msg::SelectNext => {
                match (&self.mode, &self.content) {
                    (Mode::Params, _) => {
                        self.param = wrapping_next(self.param, self.params.rows().len());
                    }
                    (Mode::Content, Content::Accounts(accounts)) => {
                        self.selected = wrapping_next(self.selected, accounts.len());
                    }
                    (Mode::Content, Content::Summary(books)) => {
                        self.scroll = (self.scroll + 1).min(last_line(books));
                    }
                }
                Cmd::none()
            }
            Msg::SelectPrev => {
                match (&self.mode, &self.content) {
                    (Mode::Params, _) => {
                        self.param = wrapping_prev(self.param, self.params.rows().len());
                    }
                    (Mode::Content, Content::Accounts(accounts)) => {
                        self.selected = wrapping_prev(self.selected, accounts.len());
                    }
                    (Mode::Content, Content::Summary(_)) => {
                        self.scroll = self.scroll.saturating_sub(1);
                    }
                }
                Cmd::none()
            }
            Msg::CycleNext => self.cycle(Step::Next),
            Msg::CyclePrev => self.cycle(Step::Prev),
            Msg::Loaded(Ok(content)) => {
                self.content = content;
                self.error = None;
                // A shorter result may have left either cursor past the end.
                self.selected = self.selected.min(self.accounts().len().saturating_sub(1));
                if let Content::Summary(books) = &self.content {
                    self.scroll = self.scroll.min(last_line(books));
                }
                Cmd::none()
            }
            // Empty rather than the previous, now-wrong contents, with the
            // reason on screen in their place.
            Msg::Loaded(Err(error)) => {
                self.content = empty(self.params.view);
                self.error = Some(error);
                self.selected = 0;
                self.scroll = 0;
                Cmd::none()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // The upper split is as tall as it needs to be: a line per row, the
        // blank one above the pages, and its border. The hint keeps the last
        // line for itself; the list takes the rest.
        let params_height = self.params.rows().len() as u16 + 3;

        let [params_area, content_area, hint_area] = Layout::vertical([
            Constraint::Length(params_height),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.view_params(frame, params_area);
        self.view_content(frame, content_area);
        frame.render_widget(Line::from(HINT).dim(), hint_area);
    }

    fn subscriptions(&self, publisher: tea::core::Publisher) -> tea::core::Cmd<Clm, Self::Msg> {
        match publisher {
            Publisher::Terminal(TerminalEvent::Key(event)) => match event.code {
                // Window navigation
                KeyCode::Char('q') => Cmd::quit(),
                KeyCode::Char('H') => navigation::back(),
                KeyCode::Char('L') => navigation::forward(),
                // Split navigation
                KeyCode::Char('J') => Cmd::msg(Msg::FocusNext),
                KeyCode::Char('K') => Cmd::msg(Msg::FocusPrev),
                // Inside the focused split
                KeyCode::Char('j') => Cmd::msg(Msg::SelectNext),
                KeyCode::Char('k') => Cmd::msg(Msg::SelectPrev),
                KeyCode::Char('l') => Cmd::msg(Msg::CycleNext),
                KeyCode::Char('h') => Cmd::msg(Msg::CyclePrev),
                // Groups. An ungrouped account has no group screen to open.
                KeyCode::Char('g') => match self
                    .accounts()
                    .get(self.selected)
                    .and_then(|account| account.group_id)
                {
                    Some(id) => Cmd::request_route(Route::group(id)),
                    None => Cmd::none(),
                },
                // Whichever split has the keys decides what Enter opens. A new
                // account and a new group are rows of the upper one, so they
                // need no key of their own.
                KeyCode::Enter => match self.mode {
                    Mode::Params => self.open(),
                    Mode::Content => match self.accounts().get(self.selected) {
                        Some(account) => Cmd::request_route(Route::account(account.id)),
                        None => Cmd::none(),
                    },
                },
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
}

// region: View
impl Model {
    /// The upper split: one row per parameter, the selected one wrapped in the
    /// arrows that change it, and under them the pages this one leads to.
    fn view_params(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.mode, Mode::Params);
        let block = pane(String::from(" Parameters "), focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = self.params.rows();
        let mut lines = Vec::with_capacity(rows.len() + 1);

        for (index, row) in rows.iter().enumerate() {
            // The pages sit under a blank line, away from the values.
            if *row == Param::NewAccount {
                lines.push(Line::from(""));
            }

            let on = focused && index == self.param;
            lines.push(match row {
                Param::View => param_line("View", view_label(self.params.view), on),
                Param::By => param_line("Order by", by_label(self.params.by), on),
                Param::Order => param_line("Direction", order_label(self.params.order), on),
                Param::Period => param_line("Period", period_label(self.params.period), on),
                Param::Year => param_line("Year", self.params.year.to_string(), on),
                Param::Month => param_line("Month", self.params.month.to_string(), on),
                Param::NewAccount => page_line("New account", on),
                Param::NewGroup => page_line("New group", on),
                Param::Home => page_line("Home", on),
            });
        }

        frame.render_widget(Text::from(lines), inner);
    }

    /// The lower split, framed and titled by whatever the view put in it.
    fn view_content(&self, frame: &mut Frame, area: Rect) {
        let focused = matches!(self.mode, Mode::Content);

        let mut block = pane(format!(" {} ", view_label(self.params.view)), focused);
        // A kind's frame carries its own totals; the summary is nothing but
        // totals already.
        if let Content::Accounts(accounts) = &self.content {
            let totals = totals(accounts.iter().map(|account| account.balance));
            if !totals.is_empty() {
                block = block.title(Line::from(format!(" {} ", show(&totals))).right_aligned());
            }
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(error) = &self.error {
            frame.render_widget(Line::from(error.clone()).red(), inner);
            return;
        }

        match &self.content {
            Content::Summary(books) => summary::view(frame, inner, books, self.scroll),
            Content::Accounts(_) => self.view_accounts(frame, inner),
        }
    }

    /// The accounts of one kind: every group, its per-currency totals, and the
    /// accounts under it.
    fn view_accounts(&self, frame: &mut Frame, area: Rect) {
        if self.accounts().is_empty() {
            frame.render_widget(Line::from("No accounts").dim(), area);
            return;
        }

        let (rows, selected) = self.rows();
        // The amount column is only as wide as the widest amount in it.
        let width = rows
            .iter()
            .map(|row| row.amount.chars().count())
            .max()
            .unwrap_or(0);

        let table = Table::new(
            rows.iter().map(ListRow::render),
            [Constraint::Fill(1), Constraint::Length(width as u16)],
        );

        // The cursor is drawn into the rows themselves; this state only carries
        // it into the widget, which derives the scroll offset from it.
        let mut state = TableState::new().with_selected(Some(selected));
        frame.render_stateful_widget(table, area, &mut state);
    }

    /// The list as it is drawn — a header per group, then its accounts — and
    /// the row the cursor sits on, which is an account and never a header.
    fn rows(&self) -> (Vec<ListRow>, usize) {
        let accounts = self.accounts();
        // The cursor is only drawn while the keys are down here. It is still
        // tracked either way, so coming back lands where it left.
        let focused = matches!(self.mode, Mode::Content);
        let mut rows = Vec::new();
        let mut selected = 0;
        let mut start = 0;

        // The accounts arrive grouped, so each group is one run of them.
        while let Some(first) = accounts.get(start) {
            let len = accounts[start..]
                .iter()
                .take_while(|account| account.group == first.group)
                .count();
            let group = &accounts[start..start + len];

            rows.push(ListRow {
                label: group_label(&first.group),
                amount: show(&totals(group.iter().map(|account| account.balance))),
                heading: true,
            });

            for (offset, account) in group.iter().enumerate() {
                if start + offset == self.selected {
                    selected = rows.len();
                }

                let marker = if focused && start + offset == self.selected {
                    "> "
                } else {
                    "  "
                };

                rows.push(ListRow {
                    label: format!("{marker}{}", account.name),
                    amount: account.balance.to_string(),
                    heading: false,
                });
            }

            start += len;
        }

        (rows, selected)
    }
}

/// One drawn line of the list: a group and its totals, or an account and its
/// balance.
struct ListRow {
    /// The left column, already carrying its cursor or its indent.
    label: String,
    /// The right column: one balance, or a group's totals side by side.
    amount: String,
    /// Set on a group's own line, which is picked out from its accounts.
    heading: bool,
}

impl ListRow {
    /// Draws the row: label left, amount right, group headings picked out.
    fn render(&self) -> Row<'static> {
        let row = Row::new(vec![
            Cell::from(self.label.clone()),
            Cell::from(Line::from(self.amount.clone()).right_aligned()),
        ]);

        if self.heading {
            row.bold().magenta()
        } else {
            row
        }
    }
}

/// The header a group of accounts is drawn under.
fn group_label(group: &str) -> String {
    if group.is_empty() {
        String::from(NO_GROUP)
    } else {
        String::from(group)
    }
}

/// One parameter row: `> Kind       < Assets >` when the keys act on it, and
/// the same columns without the arrows when they do not.
fn param_line(label: &str, value: impl Into<String>, selected: bool) -> Line<'static> {
    let name = Span::raw(format!("{label:<PARAM_LABEL_WIDTH$}"));
    let value = Span::raw(value.into());

    if selected {
        Line::from(vec![
            Span::raw("> "),
            name,
            Span::raw("< ").dim(),
            value.bold(),
            Span::raw(" >").dim(),
        ])
    } else {
        Line::from(vec![Span::raw("  "), name, Span::raw("  "), value])
    }
}

/// One row that opens a page: no value, and so no arrows around one.
fn page_line(label: &'static str, selected: bool) -> Line<'static> {
    if selected {
        Line::from(vec![Span::raw("> "), Span::raw(label).bold()])
    } else {
        Line::from(vec![Span::raw("  "), Span::raw(label)])
    }
}

/// How a view is spelled on screen — the title of the lower split, and the
/// value of the row that chooses it.
fn view_label(view: View) -> &'static str {
    match view {
        View::Summary => "Summary",
        View::Kind(kind) => kind_label(kind),
    }
}

/// How an account kind is spelled on screen.
fn kind_label(kind: AccountKind) -> &'static str {
    match kind {
        AccountKind::Asset => "Assets",
        AccountKind::Liability => "Liabilities",
        AccountKind::Equity => "Equity",
        AccountKind::Income => "Incomes",
        AccountKind::Expense => "Expenses",
    }
}

/// How a sort column is spelled on screen.
fn by_label(by: By) -> &'static str {
    match by {
        By::Balance => "Balance",
        By::Name => "Name",
        By::Postings => "Postings",
    }
}

/// How a sort direction is spelled on screen.
fn order_label(order: Order) -> &'static str {
    match order {
        Order::Asc => "Ascending",
        Order::Desc => "Descending",
    }
}

/// How a period is spelled on screen.
fn period_label(period: Period) -> &'static str {
    match period {
        Period::AllTime => "All time",
        Period::Year => "Year",
        Period::Month => "Month",
    }
}
// endregion

// region: Loading
/// What the lower split holds before the worker has read anything into it.
fn empty(view: View) -> Content {
    match view {
        View::Summary => Content::Summary(Vec::new()),
        View::Kind(_) => Content::Accounts(Vec::new()),
    }
}

/// Asks the worker for whatever `params` selects.
///
/// Takes the parameters by reference and copies what the query needs, so the
/// caller keeps them; the closure has to own its data to cross to the worker
/// thread.
fn load(params: &Params) -> Cmd<Msg> {
    let (view, by, order) = (params.view, params.by, params.order);
    let dates = params.dates();

    Cmd::task(move |ctx| {
        // The summary covers the whole ledger, so none of the narrowing
        // parameters reach it — which is why the split does not offer them.
        let View::Kind(kind) = view else {
            return Msg::Loaded(summary::read(&ctx.db).map(Content::Summary));
        };

        let filters = Filters {
            kinds: Some(vec![kind.as_u8()]),
            dates,
            ..Filters::default()
        };

        let items = match accounts::view(&ctx.db, filters, by, order) {
            Ok(items) => items,
            Err(error) => return Msg::Loaded(Err(error.to_string())),
        };

        let accounts = items
            .into_iter()
            .filter_map(|item| row(item, kind))
            .collect();

        Msg::Loaded(Ok(Content::Accounts(grouped(accounts))))
    })
}

/// Gathers the accounts of one group into one run.
///
/// Groups keep the order they first appear in, and the sort the query applied
/// survives inside each of them — so "by balance, descending" still means the
/// richest account leads its group, and its group leads the list.
fn grouped(mut accounts: Vec<Account>) -> Vec<Account> {
    let mut order: Vec<&str> = Vec::new();
    for account in &accounts {
        if !order.contains(&account.group.as_str()) {
            order.push(&account.group);
        }
    }
    let order: Vec<String> = order.into_iter().map(String::from).collect();

    // `sort_by_key` is stable, which is what keeps the query's order intact.
    accounts.sort_by_key(|account| order.iter().position(|group| *group == account.group));
    accounts
}

/// Turns a read-model row into the row this page draws, dropping any account
/// whose stored currency this build does not know.
///
/// `kind` comes from the query rather than from the row: the filter pins every
/// row to that one kind.
fn row(item: accounts::ResultItem, kind: AccountKind) -> Option<Account> {
    let currency = Currency::try_from(item.currency).ok()?;

    // The read model's balance is debit-positive. Credit-normal kinds — a
    // liability, equity, an income — read the other way round on screen: owing
    // $500 is `$500`, not `-$500`.
    let balance = Money::new(item.balance, currency);
    let balance = match kind.normal_balance() {
        Side::Debit => balance,
        Side::Credit => -balance,
    };

    Some(Account {
        id: item.id,
        group_id: item.group_id,
        group: item.group_name.unwrap_or_default(),
        name: item.name,
        balance,
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

/// The furthest the summary can be scrolled, so it cannot leave the screen.
fn last_line(books: &[Book]) -> usize {
    summary::height(books).saturating_sub(1)
}

/// The other sort direction.
fn flip(order: Order) -> Order {
    match order {
        Order::Asc => Order::Desc,
        Order::Desc => Order::Asc,
    }
}

/// The view one step from `view`, wrapping round the ends.
///
/// The summary leads the five kinds, so the screen opens on it and one press
/// of `h` reaches it from anywhere in the ring.
fn cycle_view(view: View, step: Step) -> View {
    use AccountKind::*;

    match step {
        Step::Next => match view {
            View::Summary => View::Kind(Asset),
            View::Kind(Asset) => View::Kind(Liability),
            View::Kind(Liability) => View::Kind(Equity),
            View::Kind(Equity) => View::Kind(Income),
            View::Kind(Income) => View::Kind(Expense),
            View::Kind(Expense) => View::Summary,
        },
        Step::Prev => match view {
            View::Summary => View::Kind(Expense),
            View::Kind(Expense) => View::Kind(Income),
            View::Kind(Income) => View::Kind(Equity),
            View::Kind(Equity) => View::Kind(Liability),
            View::Kind(Liability) => View::Kind(Asset),
            View::Kind(Asset) => View::Summary,
        },
    }
}

/// The sort column one step from `by`, wrapping round the ends.
fn cycle_by(by: By, step: Step) -> By {
    match step {
        Step::Next => match by {
            By::Balance => By::Name,
            By::Name => By::Postings,
            By::Postings => By::Balance,
        },
        Step::Prev => match by {
            By::Balance => By::Postings,
            By::Postings => By::Name,
            By::Name => By::Balance,
        },
    }
}

/// The period one step from `period`, wrapping round the ends.
fn cycle_period(period: Period, step: Step) -> Period {
    match step {
        Step::Next => match period {
            Period::AllTime => Period::Year,
            Period::Year => Period::Month,
            Period::Month => Period::AllTime,
        },
        Step::Prev => match period {
            Period::AllTime => Period::Month,
            Period::Month => Period::Year,
            Period::Year => Period::AllTime,
        },
    }
}

/// The first day of `month` in `year`, as the journal stores its dates.
fn day(year: i32, month: Month) -> String {
    format!("{year:04}-{:02}-01", u8::from(month))
}

#[derive(Debug)]
pub struct Account {
    id: Uuid,
    /// The group the account belongs to, or `None` when it is ungrouped.
    group_id: Option<Uuid>,
    group: String,
    name: String,
    balance: Money,
}

#[derive(Debug)]
pub enum Msg {
    FocusNext,
    FocusPrev,
    SelectNext,
    SelectPrev,
    CycleNext,
    CyclePrev,
    Loaded(Result<Content, String>),
}
