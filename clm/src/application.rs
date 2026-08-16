//! **Use cases.**
//!
//! [`services`] drive the domain through its repository traits, [`views`] read
//! the database directly for the screens, and [`infrastructure`] holds the
//! concrete adapters that implement those traits.

pub mod infrastructure;
pub mod services;
pub mod views;
