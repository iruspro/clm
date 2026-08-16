//! **The business model.**
//!
//! Aggregates, entities, value objects, and the repository *traits* the outer
//! layers implement.

pub mod account;
pub mod account_group;
mod error;
mod ids;
pub mod journal;
pub mod money;
mod name;
mod side;

pub use crate::domain::error::repository::RepoError;
pub use crate::domain::ids::{AccountGroupId, AccountId, EntryId};
pub use crate::domain::name::{Name, NameError};
pub use crate::domain::side::Side;
