//! Composition root for the application layer: builds use cases and views
//! over a live connection.

use application::CreateAccountUseCase;
use application::infrastructure::SQLiteAccountRepository;
use application::views::accounts;
use rusqlite::Connection;

/// The repository implementation every builder below is wired with.
type AccountRepo<'a> = SQLiteAccountRepository<'a>;

// region: Use cases
/// Builds the "create an account" use case over `conn`.
pub fn create_account_use_case<'a>(conn: &'a Connection) -> CreateAccountUseCase<AccountRepo<'a>> {
    CreateAccountUseCase::new(AccountRepo::new(conn))
}
// endregion
