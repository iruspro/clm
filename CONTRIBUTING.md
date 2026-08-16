# Contributing to clm

Thanks for your interest in **clm**! This document covers everything you need to
build, test, and submit changes. The project is in early development, so the
codebase and these guidelines may still move quickly.

## Prerequisites

clm is a Cargo **workspace** on Rust **edition 2024**. You need two toolchains:

| Toolchain | Used for | Why |
| --- | --- | --- |
| **stable** | build, test, clippy, docs | everyday development |
| **nightly** | `cargo fmt` only | the formatting rules in [`rustfmt.toml`](rustfmt.toml) (`group_imports`, `imports_granularity`) are unstable and only take effect on nightly |

Install both, with the components the hooks and CI expect:

```sh
# stable: build / test / lint
rustup toolchain install stable --component clippy

# nightly: formatting only
rustup toolchain install nightly --component rustfmt
```

If you skip nightly, `cargo fmt` falls back to stable, which **silently ignores**
the import-grouping rules — your formatting will then fail CI even though it
looked fine locally. Always format with `cargo +nightly fmt`.

## Getting started

```sh
git clone <your-fork-url>
cd clm

# install the git hooks once per clone (see "Git hooks" below)
./scripts/setup-hooks.sh

# build everything
cargo build --workspace
```

## Project layout

The workspace holds two crates: [`tea`](tea), a reusable Elm Architecture
runtime over ratatui, and [`clm`](clm), the application itself.

Inside `clm` the layering is **clean architecture** — dependencies point
*inward*, toward the domain, and nothing in `domain` knows about the outer
layers.
```
app ──► application ─────────────► domain
             │                       ▲
             └─► infrastructure ─────┘
```

| Module | Role | May use |
| --- | --- | --- |
| [`domain`](clm/src/domain.rs) | Pure business model — aggregates, entities, value objects, and repository **traits**. No I/O, no dependencies on other layers. | — |
| [`application`](clm/src/application.rs) | Services that orchestrate the domain through the repository traits; read-side views; Also hosts the `infrastructure` module below. | `domain`, `db` |
| [`application::infrastructure`](clm/src/application/infrastructure.rs) | Concrete adapters that **implement** the domain's repository traits (SQLite persistence). | `domain`, `db` |
| [`db`](clm/src/db.rs) | The SQL schema in one place: `sea_query` identifiers, migrations, and the functions that apply them. | — |
| [`app`](clm/src/app.rs) | Terminal UI; wires `application::infrastructure` into `application` at the composition root ([`ctx`](clm/src/ctx.rs)). | everything |

A practical consequence: the domain defines `trait *Repository`, and
`application::infrastructure` is the only place allowed to implement them.

Inside `app` the layout follows **The Elm Architecture**: `App` is the root
model, `update` is the one pure `(Model, Msg) -> (Model, Cmd)` step, `view`
renders it, and the `tea` runtime runs the resulting commands against the
database on a background thread. `cargo doc -p clm --open` starts at that
overview.

## Development workflow

These mirror the CI jobs, so running them locally is the fastest way to get a
green build. From the workspace root:

```sh
# format (nightly!) — check, or drop --check to apply
cargo +nightly fmt --all --check

# lint, warnings denied
cargo clippy --workspace --all-targets --all-features -- -D warnings

# build + test
cargo test --workspace --all-features

# docs, warnings denied (catches broken intra-doc links)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
```

### Code conventions

- **Document every public item.** `pub` types, functions, and modules carry a
  doc comment; modules open with a `//!` summary. The `doc` CI job denies
  warnings, so broken `[`intra-doc links`]` fail the build.
- **Document a module inside the module**, with `//!` — never with a `///`
  comment on its `mod foo;` declaration. Rustdoc merges the two, then resolves
  the merged comment's links against the *declaring* module, which silently
  breaks every relative link in the module's own header.
- **Imports** are grouped std → external → crate and merged per module; this is
  enforced by `rustfmt.toml`. Don't hand-organize — let `cargo +nightly fmt`
  sort it.
- **Keep the domain pure.** No I/O, no framework or persistence types in
  `domain`. Errors are co-located with the type that produces them.

## Git hooks

Hooks live in the tracked [`.githooks/`](.githooks) directory. Install them once
per clone:

```sh
./scripts/setup-hooks.sh
```

This points git at `.githooks/` via `core.hooksPath` (a local, per-clone setting
that isn't committed, so every clone must run the script) and enables:

| Hook | Runs | Purpose |
| --- | --- | --- |
| **pre-commit** | `cargo +nightly fmt --all --check` | fast formatting check on every commit |
| **pre-push** | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | lint before sharing |

Bypass a hook for a work-in-progress change with `git commit --no-verify` /
`git push --no-verify` — CI enforces the same checks regardless.

## Branching and pull requests

- **`main`** — stable/release branch. **`develop`** — active integration branch.
- Branch off `develop` and open pull requests **against `develop`** (not `main`).
- All CI jobs must pass before a PR is merged.

### Commit messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/).
Use a `type: summary` subject in the imperative mood, e.g.:

```
feat: add AccountGroup::into_parts method
fix: application layer errors
refactor: domain public API visibility
chore: update rustfmt config
docs: document from_parts reconstruction contract
```

Common types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `perf`.

## Continuous integration

Every push and pull request runs [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

| Job | Toolchain | Command |
| --- | --- | --- |
| **rustfmt** | nightly | `cargo +nightly fmt --all --check` |
| **clippy** | stable | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| **test** | stable + beta | `cargo build` then `cargo test --workspace --all-features` |
| **doc** | stable | `cargo doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings` |

The `test` and `doc` jobs run on pull requests and on the `main`/`develop`
branches.

## License

By contributing, you agree that your contributions are dual-licensed under the
[Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) licenses, matching the
project's license.
