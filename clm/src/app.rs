mod page;
mod router;

use ratatui::Frame;
use ratatui::layout::Rect;
use tea::browser::navigation;
use tea::core::Publisher;
use tea::{ElmModel, World};

use crate::app::router::Route;
use crate::ctx::Ctx;

/// The Elm Architecture main model
#[derive(Debug)]
pub struct App {
    /// Current page; handled by the Elm model.
    page: Page,
}

#[derive(Debug)]
enum Page {
    Home(page::home::Model),
    Help(page::help::Model),
    Accounts(page::accounts::Model),
    Account(page::account::Model),
    Group(page::group::Model),
    Journal(page::journal::Model),
    Transaction(page::transaction::Model),
    // Transactions,
    // Account(Uuid),
    // Transaction(Uuid),
}

impl Page {
    /// Builds the page `route` points at, and the command that fills it.
    fn open(route: Route) -> (Self, Cmd<Msg>) {
        match route {
            Route::Home => {
                let (page, cmd) = page::home::Model::init();
                (Page::Home(page), cmd.map(Msg::Home))
            }
            Route::Help => {
                let (page, cmd) = page::help::Model::init();
                (Page::Help(page), cmd.map(Msg::Help))
            }
            Route::Accounts(view) => {
                let (page, cmd) = page::accounts::Model::init(view);
                (Page::Accounts(page), cmd.map(Msg::Accounts))
            }
            Route::Account(id) => {
                let (page, cmd) = page::account::Model::init(id);
                (Page::Account(page), cmd.map(Msg::Account))
            }
            Route::Group(id) => {
                let (page, cmd) = page::group::Model::init(id);
                (Page::Group(page), cmd.map(Msg::Group))
            }
            Route::Journal => {
                let (page, cmd) = page::journal::Model::init();
                (Page::Journal(page), cmd.map(Msg::Journal))
            }
            Route::Transaction { id, from } => {
                let (page, cmd) = page::transaction::Model::init(id, from);
                (Page::Transaction(page), cmd.map(Msg::Transaction))
            }
        }
    }
}

impl App {
    /// Returns the route the app opens at.
    pub fn start() -> Route {
        Route::home()
    }

    /// Builds the root model showing `route`.
    pub fn init(route: Route) -> (Self, Cmd<Msg>) {
        let (page, cmd) = Page::open(route);
        (Self { page }, cmd)
    }
}

impl ElmModel<Clm> for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match (msg, &mut self.page) {
            (Msg::RequestedRoute(route), _) => navigation::push_route(route),
            (Msg::ChangedRoute(route), _) => {
                let (page, cmd) = Page::open(route);
                self.page = page;
                cmd
            }
            (Msg::Home(msg), Page::Home(model)) => model.update(msg).map(Msg::Home),
            (Msg::Help(msg), Page::Help(model)) => model.update(msg).map(Msg::Help),
            (Msg::Accounts(msg), Page::Accounts(model)) => model.update(msg).map(Msg::Accounts),
            (Msg::Account(msg), Page::Account(model)) => model.update(msg).map(Msg::Account),
            (Msg::Group(msg), Page::Group(model)) => model.update(msg).map(Msg::Group),
            (Msg::Journal(msg), Page::Journal(model)) => model.update(msg).map(Msg::Journal),
            (Msg::Transaction(msg), Page::Transaction(model)) => {
                model.update(msg).map(Msg::Transaction)
            }
            // Disregard messages that arrived for the wrong page.
            (_, _) => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        match &self.page {
            Page::Home(model) => model.view(frame, area),
            Page::Help(model) => model.view(frame, area),
            Page::Accounts(model) => model.view(frame, area),
            Page::Account(model) => model.view(frame, area),
            Page::Group(model) => model.view(frame, area),
            Page::Journal(model) => model.view(frame, area),
            Page::Transaction(model) => model.view(frame, area),
        }
    }

    fn subscriptions(&self, publisher: Publisher) -> Cmd<Self::Msg> {
        match &self.page {
            Page::Home(model) => model.subscriptions(publisher).map(Msg::Home),
            Page::Help(model) => model.subscriptions(publisher).map(Msg::Help),
            Page::Accounts(model) => model.subscriptions(publisher).map(Msg::Accounts),
            Page::Account(model) => model.subscriptions(publisher).map(Msg::Account),
            Page::Group(model) => model.subscriptions(publisher).map(Msg::Group),
            Page::Journal(model) => model.subscriptions(publisher).map(Msg::Journal),
            Page::Transaction(model) => model.subscriptions(publisher).map(Msg::Transaction),
        }
    }
}

pub struct Clm;

impl World for Clm {
    type Route = Route;
    type Ctx = Ctx;
}

pub type Cmd<Msg> = tea::core::Cmd<Clm, Msg>;

#[derive(Debug)]
pub enum Msg {
    RequestedRoute(Route),
    ChangedRoute(Route),
    Home(page::home::Msg),
    Help(page::help::Msg),
    Accounts(page::accounts::Msg),
    Account(page::account::Msg),
    Group(page::group::Msg),
    Journal(page::journal::Msg),
    Transaction(page::transaction::Msg),
    // Transactions,
    // Account,
    // Transaction,
}
