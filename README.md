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
  - [ ] Create, edit, delete accounts
  - [ ] View account balances
- [ ] Category management
  - [ ] Income and expense categories with subcategories
  - [ ] Create, rename, delete categories
- [ ] Transaction management
  - [ ] Record income and expenses
  - [ ] Transfer between accounts
  - [ ] List, edit, delete transactions
- [ ] TUI with three main views: accounts, categories, transactions
- [ ] Keyboard-driven navigation
- [ ] Basic monthly summary (total income / expenses per category)

### Post-MVP

- [ ] Budgets and spending limits
- [ ] Transaction filtering and search
- [ ] Import / export (CSV)
- [ ] Investment assets (stocks, crypto)
- [ ] Portfolio tracking with P/L
- [ ] Charts and detailed statistics
- [ ] Multi-currency support
- [ ] Configuration file

## Development

The hooks require the `rustfmt` and `clippy` toolchain components:

```sh
rustup component add rustfmt clippy
```

After cloning, install the git hooks once:

```sh
./scripts/setup-hooks.sh
```

This points git at the tracked `.githooks/` directory (via `core.hooksPath`) and enables:

* **pre-commit** — `cargo fmt --check` (fast formatting check on every commit)
* **pre-push** — `cargo clippy` with warnings denied (lint before sharing)

`core.hooksPath` is a local, per-clone setting, so each clone must run the script once.
Use `git commit --no-verify` / `git push --no-verify` to bypass a hook for a WIP change;
CI enforces the same checks regardless.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.