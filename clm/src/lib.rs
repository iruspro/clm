//! **Command Line Money** — a double-entry ledger driven from the terminal.
//!
//! ```text
//! app ──► application ─────────────► domain
//!             │                        ▲
//!             └─► infrastructure ──────┘
//! ```
//!
//! - [`domain`] is the pure business model: aggregates, value objects, and the
//!   repository **traits**. It reaches for nothing else in the crate.
//! - [`application`] orchestrates the domain through those traits, and hosts
//!   [`application::infrastructure`], the only place allowed to implement them.
//! - [`db`] owns the SQL schema and the migrations.
//! - [`app`] is the terminal UI, laid out as The Elm Architecture on top of the
//!   [`tea`] crate; [`ctx`] is its composition root.

pub mod app;
pub mod application;
pub mod ctx;
pub mod db;
pub mod domain;
