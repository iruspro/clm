//! The concrete forms behind [`ConcreteForm`](super::ConcreteForm).
//!
//! One module per form. Each owns its fields, its focus and its validation
//! errors, implements [`Form`](crate::tui::form::Form) for the shared editing and
//! rendering behaviour, and adds its own `to_command` for the part that can't be
//! shared — parsing raw input into the domain command it produces.

mod account;

pub use self::account::AccountForm;
