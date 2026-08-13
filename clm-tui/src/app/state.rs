//! The application state tree — TEA's "model".
//!
//! [`State`] is the single source of truth for what the UI shows. Nothing else
//! holds UI state: the reducer is the only writer, the renderer is a pure
//! function of it, and it contains no handles, channels or connections, so it
//! stays cheap to reason about.
//!
//! Its fields are private and reached through accessors, which is what keeps the
//! bookkeeping in [`open_request`](State::open_request) /
//! [`open_modal`](State::open_modal) from being bypassed.

mod accounts;
mod categories;
pub mod modal;
mod screen;
mod transactions;

use std::collections::{HashMap, VecDeque};

pub use self::accounts::AccountsState;
pub use self::categories::CategoriesState;
pub use self::modal::{ConcreteForm, Modal};
pub use self::screen::Screen;
pub use self::transactions::TransactionsState;
use crate::app::command::RequestId;
use crate::app::config::KeyBindings;

/// Everything the UI draws and the reducer mutates.
///
/// Two of the fields deserve a note:
///
/// - `modals` is a [`VecDeque`], not a `Vec`, because modals arrive from both
///   ends. A key press opens one immediately at the front
///   ([`open_modal`](Self::open_modal)); an error arriving from a background
///   write queues at the back ([`open_modal_delayed`](Self::open_modal_delayed))
///   so it cannot yank the screen out from under whatever the user is currently
///   answering. The front element is the one on screen.
/// - `request_forms` parks submitted forms by [`RequestId`] while their write is
///   in flight, so a rejected write can put the user's input back on screen
///   instead of discarding it.
#[derive(Debug)]
pub struct State {
    title: &'static str,
    keys: KeyBindings,

    screen: Screen,
    accounts: AccountsState,
    categories: CategoriesState,
    transactions: TransactionsState,

    /// Forms whose write is in flight, keyed by the request that carries them.
    request_forms: HashMap<RequestId, ConcreteForm>,
    /// Modal stack; the front element is the visible one.
    modals: VecDeque<Modal>,

    should_quit: bool,
}

impl State {
    /// The initial state: the Accounts screen, no modals, nothing loaded yet.
    pub fn new(title: &'static str, keys: KeyBindings) -> Self {
        Self {
            title,
            keys,
            screen: Screen::default(),
            accounts: AccountsState::default(),
            categories: CategoriesState::default(),
            transactions: TransactionsState::default(),
            request_forms: HashMap::new(),
            modals: VecDeque::new(),
            should_quit: false,
        }
    }

    // region: Getters
    /// The app title, as shown in the header.
    pub fn title(&self) -> &'static str {
        self.title
    }

    /// The active key bindings. Returned by value — [`KeyBindings`] is [`Copy`]
    /// — so callers can read them without holding a borrow on `self` while they
    /// mutate it.
    pub fn keys(&self) -> KeyBindings {
        self.keys
    }

    /// The screen currently selected in the tab bar.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// State of the Accounts screen.
    pub fn accounts(&self) -> &AccountsState {
        &self.accounts
    }

    /// Mutable state of the Accounts screen, for the reducer.
    pub fn accounts_mut(&mut self) -> &mut AccountsState {
        &mut self.accounts
    }

    /// State of the Categories screen.
    pub fn categories(&self) -> &CategoriesState {
        &self.categories
    }

    /// Mutable state of the Categories screen, for the reducer.
    #[expect(dead_code, reason = "the Categories screen has no reducer yet")]
    pub fn categories_mut(&mut self) -> &mut CategoriesState {
        &mut self.categories
    }

    /// State of the Transactions screen.
    pub fn transactions(&self) -> &TransactionsState {
        &self.transactions
    }

    /// Mutable state of the Transactions screen, for the reducer.
    #[expect(dead_code, reason = "the Transactions screen has no reducer yet")]
    pub fn transactions_mut(&mut self) -> &mut TransactionsState {
        &mut self.transactions
    }

    /// The modal on screen, or `None` when the screen is unobstructed.
    pub fn modal(&self) -> Option<&Modal> {
        self.modals.front()
    }

    /// Mutable access to the modal on screen.
    #[expect(
        dead_code,
        reason = "read half of the modal accessor pair; no handler mutates a modal in place yet"
    )]
    pub fn modal_mut(&mut self) -> Option<&mut Modal> {
        self.modals.front_mut()
    }

    /// The form on screen, or `None` if the visible modal isn't a form.
    ///
    /// A shortcut past the [`Modal::Form`] match, for the paths that only care
    /// about forms.
    #[expect(
        dead_code,
        reason = "read half of the form accessor pair; the renderer reaches forms via Modal::Form"
    )]
    pub fn form(&self) -> Option<&ConcreteForm> {
        match self.modals.front() {
            Some(Modal::Form(form)) => Some(form),
            _ => None,
        }
    }

    /// Mutable access to the form on screen; `None` if the visible modal isn't
    /// a form. This is what the key handler edits through.
    pub fn form_mut(&mut self) -> Option<&mut ConcreteForm> {
        match self.modals.front_mut() {
            Some(Modal::Form(form)) => Some(form),
            _ => None,
        }
    }

    /// Whether the loop should stop after this iteration.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    // endregion

    // region: Setters
    /// Advances the tab bar one screen to the right, wrapping around.
    pub fn next_screen(&mut self) {
        self.screen = self.screen.toggle_next()
    }

    /// Moves the tab bar one screen to the left, wrapping around.
    pub fn prev_screen(&mut self) {
        self.screen = self.screen.toggle_prev()
    }

    /// Parks `form` under a fresh [`RequestId`] and returns it, to be attached to
    /// the [`Command`](crate::app::Command) it produced.
    ///
    /// Pair every call with a [`close_request`](Self::close_request) on both the
    /// success and the failure path, or the form leaks for the life of the
    /// process.
    pub fn open_request(&mut self, form: ConcreteForm) -> RequestId {
        let rid = RequestId::new();
        self.request_forms.insert(rid, form);

        rid
    }

    /// Takes the parked form back out, if `rid` is still open.
    ///
    /// Returns `None` for an unknown or already-closed id, so calling it twice
    /// is harmless.
    pub fn close_request(&mut self, rid: RequestId) -> Option<ConcreteForm> {
        self.request_forms.remove(&rid)
    }

    /// Shows `modal` immediately, pushing any current one behind it.
    pub fn open_modal(&mut self, modal: Modal) {
        self.modals.push_front(modal);
    }

    /// Queues `modal` behind everything already open.
    ///
    /// This is the right call for anything the user didn't just ask for — a
    /// background write failing, say — so it waits its turn instead of replacing
    /// whatever is on screen mid-keystroke.
    pub fn open_modal_delayed(&mut self, modal: Modal) {
        self.modals.push_back(modal);
    }

    /// Dismisses the modal on screen and returns it, revealing the next one (if
    /// any). `None` when nothing was open.
    pub fn close_modal(&mut self) -> Option<Modal> {
        self.modals.pop_front()
    }

    /// Asks the loop to exit after this iteration. One-way.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    // endregion

    // region: Checkers
    /// Whether any modal is open. Decides whether a key press is routed to the
    /// modal handler or to the active screen.
    pub fn is_modal(&self) -> bool {
        !self.modals.is_empty()
    }
    // endregion
}
