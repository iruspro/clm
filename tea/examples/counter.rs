use std::fmt::Display;
use std::time::Duration;

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use tea::browser::{Program, navigation};
use tea::core::{Cmd, Publisher, Result, TerminalEvent};
use tea::{ElmModel, World};

fn main() -> Result<()> {
    let program = Program::new(
        Route::Counter,
        App::init,
        Msg::RequestedPage,
        Msg::ChangedPage,
    );
    tea::run("Counter", (), program, Duration::from_millis(250))
}

struct App {
    model: Model,
}

enum Model {
    Counter(i64),
    About(String),
}

impl Model {
    /// The model a route asks for: one place builds pages, `init` included.
    fn open(route: Route) -> Self {
        match route {
            Route::Counter => Model::Counter(0),
            Route::About => Model::About("Some fancy text about the app.".into()),
        }
    }
}

impl App {
    /// A plain constructor: the trait does not ask for one, so it can take
    /// whatever a model happens to need — here, the route to open at.
    fn init(route: Route) -> (Self, Cmd<Counter, Msg>) {
        (
            App {
                model: Model::open(route),
            },
            Cmd::none(),
        )
    }
}

impl ElmModel<Counter> for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Counter, Msg> {
        match (msg, &self.model) {
            (Msg::RequestedPage(page), _) => navigation::push_route(page),
            (Msg::ChangedPage(page), _) => {
                self.model = Model::open(page);
                Cmd::none()
            }
            (Msg::Add, Model::Counter(count)) => {
                self.model = Model::Counter(*count + 1);
                Cmd::none()
            }
            (Msg::Sub, Model::Counter(count)) => {
                self.model = Model::Counter(*count - 1);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let widget = match &self.model {
            Model::Counter(count) => {
                format!("count: {}\n\n  k  up\n  j  down\n  q  quit", count)
            }
            Model::About(about) => about.into(),
        };

        frame.render_widget(widget, area);
    }

    fn subscriptions(&self, publisher: Publisher) -> Cmd<Counter, Msg> {
        match publisher {
            Publisher::Terminal(TerminalEvent::Key(e)) => match e.code {
                KeyCode::Char('q') => Cmd::quit(),
                KeyCode::Char('H') => navigation::back(),
                KeyCode::Char('L') => navigation::forward(),
                KeyCode::Char('j') => Cmd::msg(Msg::Add),
                KeyCode::Char('k') => Cmd::msg(Msg::Sub),
                KeyCode::Char('a') => Cmd::request_route(Route::About),
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
}

// region: World
struct Counter;

impl World for Counter {
    type Route = Route;
    type Ctx = ();
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Route {
    Counter,
    About,
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let route = match self {
            Route::Counter => "counter",
            Route::About => "about",
        };
        write!(f, "{}", route)
    }
}
// endregion

enum Msg {
    RequestedPage(Route),
    ChangedPage(Route),
    Add,
    Sub,
}
