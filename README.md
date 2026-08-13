# clm — Command Line Money

**clm (Command Line Money)** is a terminal-based personal finance manager written in **Rust**.
The application provides a full-featured financial tracking system accessible entirely from the command line. It allows users to manage expenses, income, accounts, and investment assets while keeping data locally stored.

The goal of the project is to build a practical TUI application that combines personal budgeting and investment tracking in a lightweight and efficient interface.

## Features

* Track **income and expenses**
* Manage multiple **accounts** (cash, bank, etc.)
* Support for **assets and investments**
  * cryptocurrencies
  * stocks
  * other financial instruments
* **Portfolio tracking** with profit/loss monitoring
* **Transaction categorization**
* **Budgeting and spending limits**
* **Financial summaries and statistics**
* Local data storage

## Installation
TODO

## Usage
TODO

## Technology

* **Rust**
* TUI (terminal user interface) interface
* **SQLite** database

## Project Status

Early development.

## Roadmap

### MVP

Goal: a usable single-user expense tracker with persistent storage.

- [x] Project skeleton and CI
- [x] Core domain model (accounts, transactions)
- [ ] SQLite persistence layer
- [ ] Account management
  - [ ] Create, edit, list accounts
- [ ] Category management
  - [ ] Income and expense categories with subcategories
  - [ ] Create, edit, list categories
- [ ] Transaction management
  - [ ] Record income and expenses
  - [ ] Transfer between accounts
  - [ ] List (with filters), edit, delete transactions
- [ ] TUI with three main views: accounts, categories, transactions
- [ ] Keyboard-driven navigation
- [ ] Basic monthly summary (total income / expenses per category)

### Post-MVP

- [ ] Sync between multiple devices
- [ ] Configuration file
- [ ] Import / export (CSV)
- [ ] Full multi-currency support
- [ ] Investment assets (stocks, crypto)
- [ ] Portfolio tracking with P/L
- [ ] Budgets and spending limits
- [ ] Charts and detailed statistics
- [ ] Split bills, debts

## Contributing

clm is a Rust (edition 2024) Cargo workspace built with a clean architecture.
See [CONTRIBUTING.md](CONTRIBUTING.md) for the toolchain setup (note: formatting
requires nightly `rustfmt`), project layout, git hooks, and the build/test/lint
workflow.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
