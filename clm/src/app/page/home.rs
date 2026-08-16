use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::List;
use tea::ElmModel;
use tea::browser::navigation;
use tea::core::{Publisher, TerminalEvent};

use crate::app::router::Route;
use crate::app::{Clm, Cmd};

/// The keys the menu answers to, spelled out under it.
const HINT: &str = "j/k to move · Enter to select · q to quit";

#[derive(Debug)]
pub struct Model {
    selected: MenuItem,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    Accounts,
    Journal,
    Help,
    Quit,
}

impl Model {
    /// Open the menu on the first entry.
    pub fn init() -> (Self, Cmd<Msg>) {
        (
            Model {
                selected: MenuItem::Accounts,
            },
            Cmd::none(),
        )
    }
}

impl ElmModel<Clm> for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Self::Msg) -> Cmd<Self::Msg> {
        match msg {
            Msg::SelectNext => {
                self.selected = match self.selected {
                    MenuItem::Accounts => MenuItem::Journal,
                    MenuItem::Journal => MenuItem::Help,
                    MenuItem::Help => MenuItem::Quit,
                    MenuItem::Quit => MenuItem::Accounts,
                };
                Cmd::none()
            }
            Msg::SelectPrev => {
                self.selected = match self.selected {
                    MenuItem::Accounts => MenuItem::Quit,
                    MenuItem::Journal => MenuItem::Accounts,
                    MenuItem::Help => MenuItem::Journal,
                    MenuItem::Quit => MenuItem::Help,
                };
                Cmd::none()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let items = ["Accounts", "Journal", "Help", "Quit"];
        let selected = match self.selected {
            MenuItem::Accounts => 0,
            MenuItem::Journal => 1,
            MenuItem::Help => 2,
            MenuItem::Quit => 3,
        };

        let lines = items.iter().enumerate().map(|(i, &item)| {
            let line = if i == selected {
                vec![Span::raw("> "), Span::raw(item)]
            } else {
                vec![Span::raw("  "), Span::raw(item)]
            };
            Line::from(line)
        });
        let list = List::new(lines).white();

        // The hint keeps the last line for itself; the menu takes the rest.
        let [menu_area, hint_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        frame.render_widget(list, menu_area);
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
                KeyCode::Enter => match self.selected {
                    MenuItem::Accounts => Cmd::request_route(Route::accounts()),
                    MenuItem::Journal => Cmd::request_route(Route::journal()),
                    MenuItem::Help => Cmd::request_route(Route::help()),
                    MenuItem::Quit => Cmd::quit(),
                },
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    SelectNext,
    SelectPrev,
}
