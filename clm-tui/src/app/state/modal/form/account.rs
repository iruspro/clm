//! The "new account" form.

use application::CreateAccountCommand;
use domain::Name;
use domain::account::AccountKind;
use domain::money::Currency;

use crate::tui::form::Form;
use crate::tui::form::fields::{Choice, FieldKind, FieldView, TextInput};
use crate::tui::form::validation::FieldError;

/// Which field of an [`AccountForm`] the keys currently act on.
///
/// Doubles as the tag on a [`FieldError`], so an error can be rendered next to
/// the field that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountField {
    /// Free text; must be a valid [`Name`].
    Name,
    /// Free text; anything goes, including empty.
    Description,
    /// Choice over [`AccountKind::ACCOUNTS`].
    Kind,
    /// Choice over [`Currency::ALL`].
    Currency,
}

impl AccountField {
    /// The next field in tab order, wrapping around.
    fn next(self) -> Self {
        match self {
            AccountField::Name => AccountField::Description,
            AccountField::Description => AccountField::Kind,
            AccountField::Kind => AccountField::Currency,
            AccountField::Currency => AccountField::Name,
        }
    }

    /// The previous field in tab order, wrapping around.
    fn prev(self) -> Self {
        match self {
            AccountField::Name => AccountField::Currency,
            AccountField::Description => AccountField::Name,
            AccountField::Kind => AccountField::Description,
            AccountField::Currency => AccountField::Kind,
        }
    }
}

/// The "new account" form: raw input parsed into a [`CreateAccountCommand`].
///
/// Holds *unvalidated* input. Nothing is checked while typing; validation
/// happens once, in [`to_command`](Self::to_command), and its failures are
/// stashed in `errors` until the next edit clears them.
#[derive(Debug, Clone)]
pub struct AccountForm {
    name: TextInput,
    description: TextInput,
    kind: Choice<AccountKind>,
    currency: Choice<Currency>,
    focus: AccountField,
    errors: Vec<FieldError<AccountField>>,
}

impl AccountForm {
    /// A blank form, focused on the name field.
    pub fn new() -> Self {
        Self {
            name: TextInput::default(),
            description: TextInput::default(),
            kind: Choice::new([AccountKind::Asset, AccountKind::Liability].to_vec()),
            currency: Choice::new(Currency::ALL.to_vec()),
            focus: AccountField::Name,
            errors: Vec::new(),
        }
    }

    /// The error message for `field`, if the last submit rejected it.
    fn error_for(&self, field: AccountField) -> Option<&str> {
        self.errors
            .iter()
            .find(|e| e.field == field)
            .map(|e| e.message.as_str())
    }

    /// Parses the current input into a validated [`CreateAccountCommand`],
    /// accumulating a [`FieldError`] per invalid field. Choice fields never fail
    /// — a selected value is already valid.
    ///
    /// This is the parse-at-the-boundary step: past this point the rest of the
    /// app deals in domain types, never in the raw strings the form holds.
    ///
    /// `group_id` is always `None` — the form has no group picker yet.
    ///
    /// # Errors
    /// Returns every field that failed at once, rather than stopping at the
    /// first, so a submit can surface all of its problems in one pass.
    pub fn to_command(&self) -> Result<CreateAccountCommand, Vec<FieldError<AccountField>>> {
        let mut errors = Vec::new();

        let name = match Name::new(self.name.value()) {
            Ok(name) => Some(name),
            Err(e) => {
                errors.push(FieldError::new(AccountField::Name, e.to_string()));
                None
            }
        };

        if let Some(name) = name
            && errors.is_empty()
        {
            Ok(CreateAccountCommand {
                name,
                description: self.description.value().to_string(),
                kind: self.kind.selected(),
                currency: self.currency.selected(),
                group_id: None,
            })
        } else {
            Err(errors)
        }
    }

    /// Records validation errors from a rejected submit.
    pub fn set_errors(&mut self, errors: Vec<FieldError<AccountField>>) {
        self.errors = errors;
    }
}

/// Editing and rendering behaviour shared with every other form.
///
/// The pattern throughout: match on `self.focus`, then act only on the field
/// kind the key makes sense for. A key aimed at the wrong kind of field — typing
/// into a choice, cycling a text input — is silently ignored rather than
/// rejected, which is what lets the reducer forward keys without knowing what is
/// focused.
impl Form for AccountForm {
    fn focus_next(&mut self) {
        self.focus = self.focus.next();
    }

    fn focus_prev(&mut self) {
        self.focus = self.focus.prev();
    }

    fn focus_kind(&self) -> FieldKind {
        match self.focus {
            AccountField::Name | AccountField::Description => FieldKind::Text,
            AccountField::Kind | AccountField::Currency => FieldKind::Choice,
        }
    }

    fn select_next(&mut self) {
        match self.focus {
            AccountField::Kind => self.kind.next(),
            AccountField::Currency => self.currency.next(),
            AccountField::Name | AccountField::Description => {}
        }
    }

    fn select_prev(&mut self) {
        match self.focus {
            AccountField::Kind => self.kind.prev(),
            AccountField::Currency => self.currency.prev(),
            AccountField::Name | AccountField::Description => {}
        }
    }

    fn input_char(&mut self, c: char) {
        match self.focus {
            AccountField::Name => self.name.push(c),
            AccountField::Description => self.description.push(c),
            AccountField::Kind | AccountField::Currency => {}
        }
    }

    fn backspace(&mut self) {
        match self.focus {
            AccountField::Name => self.name.pop(),
            AccountField::Description => self.description.pop(),
            AccountField::Kind | AccountField::Currency => {}
        }
    }

    fn clear_errors(&mut self) {
        self.errors.clear();
    }

    fn title(&self) -> &str {
        "New account"
    }

    fn fields(&self) -> Vec<FieldView<'_>> {
        vec![
            FieldView::new(
                "Name",
                self.name.value(),
                self.focus == AccountField::Name,
                self.error_for(AccountField::Name),
            ),
            FieldView::new(
                "Description",
                self.description.value(),
                self.focus == AccountField::Description,
                self.error_for(AccountField::Description),
            ),
            // Choice fields have no stored string, so they format their selected
            // option into an owned value; `FieldView::new` takes it as a `Cow`.
            FieldView::new(
                "Kind",
                format!("{:?}", self.kind.selected()),
                self.focus == AccountField::Kind,
                self.error_for(AccountField::Kind),
            ),
            FieldView::new(
                "Currency",
                format!("{:?}", self.currency.selected()),
                self.focus == AccountField::Currency,
                self.error_for(AccountField::Currency),
            ),
        ]
    }
}
