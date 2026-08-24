//! The window: a title, a location bar and a history,
//! drawn around whatever is put inside.

use std::collections::LinkedList;
use std::fmt::Display;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;

/// The frame around everything else.
pub(crate) struct Window<P: Display> {
    title: String,
    /// Where the window has been, and where it is: the top of this stack is
    /// the current address.
    back: LinkedList<P>,
    /// Where [`back`](Self::back) came from, newest on top.
    forward: LinkedList<P>,
}

impl<R: Display> Window<R> {
    /// Opens at `home`, with nowhere to go back to.
    pub(crate) fn new(title: String, home: R) -> Self {
        Self {
            title,
            back: LinkedList::from([home]),
            forward: LinkedList::new(),
        }
    }

    // region: Location
    /// Where the window is: the top of the back stack, which is never empty.
    pub(crate) fn route(&self) -> &R {
        self.back.front().expect("the root is never dropped")
    }

    /// Change the route, but do not trigger a page load.
    ///
    /// This will add a new entry to the browser history.
    ///
    /// # Note
    /// Adding a new route in that scenario will clear out any future
    /// routes.
    pub(crate) fn push_route(&mut self, route: R) {
        self.back.push_front(route);
        self.forward.clear();
    }

    /// Step back, if there is anywhere behind the root to step to.
    pub(crate) fn back(&mut self) -> bool {
        if self.can_go_back()
            && let Some(left) = self.back.pop_front()
        {
            self.forward.push_front(left);
            true
        } else {
            false
        }
    }

    /// Undo a [`back`](Self::back), if one is outstanding.
    pub(crate) fn forward(&mut self) -> bool {
        if let Some(next) = self.forward.pop_front() {
            self.back.push_front(next);
            true
        } else {
            false
        }
    }

    /// Whether anything but the root is on the back stack.
    fn can_go_back(&self) -> bool {
        self.back.len() > 1
    }
    // endregion

    // region: View
    /// Draw the chrome and return the space left over.
    pub(crate) fn view(&self, frame: &mut Frame) -> Rect {
        let block = Block::bordered()
            .title(Line::from(format!(" {} ", self.title)).bold().centered())
            .border_set(border::THICK);

        let whole_area = frame.area();
        let inner_area = block.inner(whole_area);

        frame.render_widget(block, whole_area);

        let [location_area, inside_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner_area);

        frame.render_widget(self.location_line(), location_area);

        inside_area
    }

    /// The location bar: which way the window can go, and where it is.
    fn location_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::raw(if self.can_go_back() { " <" } else { "  " }),
            Span::raw(if self.can_go_forward() { " > " } else { "   " }),
            Span::raw(format!("route://{}", self.route())).bold(),
        ])
    }

    fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
    // endregion
}
