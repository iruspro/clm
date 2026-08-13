//! Service: create a new account in the chart of accounts.
use domain::account::{Account, AccountKind, AccountRepository};
use domain::money::Currency;
use domain::{AccountGroupId, AccountId, Name, RepoError};
use thiserror::Error;

/// Input for [`CreateAccountService`].
#[derive(Debug)]
pub struct CreateAccountCommand {
    pub kind: AccountKind,
    pub currency: Currency,
    pub name: Name,
    pub description: String,
    pub group_id: Option<AccountGroupId>,
}

/// Builds an [`Account`] with a freshly generated id and stores it, returning
/// that id.
pub struct CreateAccountService<AR> {
    accounts: AR,
}

impl<AR> CreateAccountService<AR>
where
    AR: AccountRepository,
{
    pub fn new(accounts: AR) -> Self {
        CreateAccountService { accounts }
    }

    pub fn execute(&self, cmd: CreateAccountCommand) -> Result<AccountId, ServiceError> {
        let id = AccountId::new();

        let account = Account::new(
            id,
            cmd.kind,
            cmd.currency,
            cmd.name,
            cmd.description,
            cmd.group_id,
        );
        self.accounts.create(&account)?;

        Ok(id)
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    /// A repository operation failed (not found or storage error).
    #[error("{0}")]
    Repo(#[from] RepoError),
}

#[cfg(test)]
mod tests {
    use domain::RepoError;
    use rusqlite::Connection;

    use super::*;
    use crate::infrastructure::repository::account::SQLiteAccountRepository;

    /// Inserts a group for the new account to point at, returning its id.
    fn seed_group(db: &Connection, name: &str) -> AccountGroupId {
        let id = AccountGroupId::new();
        db.execute(
            "INSERT INTO account_group (id, name, description) VALUES (?1, ?2, '')",
            (id.to_uuid(), name),
        )
        .expect("seed account group");

        id
    }

    /// A sample command. `AccountKind::Expense` and `Currency::BTC` are
    /// deliberately not the first variant, so a mix-up between a value and a
    /// default discriminant of 0 would show up.
    fn sample_command(group_id: Option<AccountGroupId>) -> CreateAccountCommand {
        CreateAccountCommand {
            kind: AccountKind::Expense,
            currency: Currency::BTC,
            name: Name::new("Cash").expect("non-empty name"),
            description: "petty cash".to_string(),
            group_id,
        }
    }

    #[test]
    fn execute_stores_an_account_built_from_the_command() {
        let db = db::testing::test_db();
        let group_id = seed_group(&db, "Bank");
        let service = CreateAccountService::new(SQLiteAccountRepository::new(&db));

        let id = service
            .execute(sample_command(Some(group_id)))
            .expect("create the account");

        // Read through a second repository over the same connection: the
        // account has to be in the database, not merely in the service's hands.
        let stored = SQLiteAccountRepository::new(&db)
            .read(id)
            .expect("read the account just created");
        assert_eq!(stored.id(), id);
        assert_eq!(stored.kind(), AccountKind::Expense);
        assert_eq!(stored.currency(), Currency::BTC);
        assert_eq!(stored.name().as_ref(), "Cash");
        assert_eq!(stored.description(), "petty cash");
        assert_eq!(stored.group_id(), Some(group_id));
    }

    #[test]
    fn execute_gives_each_account_its_own_id() {
        let db = db::testing::test_db();
        let service = CreateAccountService::new(SQLiteAccountRepository::new(&db));

        // Two identical commands: the ids can only differ if the service mints
        // a fresh one per call, rather than deriving it from the input or
        // leaving it at `AccountId::default()` — which, id being the primary
        // key, would make the second insert collide with the first.
        let first = service.execute(sample_command(None)).expect("create first");
        let second = service
            .execute(sample_command(None))
            .expect("create second");

        assert_ne!(first, second);
        let repo = SQLiteAccountRepository::new(&db);
        assert_eq!(repo.read(first).expect("read first").id(), first);
        assert_eq!(repo.read(second).expect("read second").id(), second);
    }

    #[test]
    fn execute_reports_a_failed_save() {
        let db = db::testing::test_db();
        // Take the table away so the insert cannot succeed. Contrived, but it
        // is the failure the service must not swallow: returning the generated
        // id for an account that was never stored.
        db.execute_batch("DROP TABLE account")
            .expect("drop the account table");
        let service = CreateAccountService::new(SQLiteAccountRepository::new(&db));

        match service.execute(sample_command(None)) {
            Err(ServiceError::Repo(RepoError::Storage(_))) => {}
            other => panic!("expected a storage error, got {other:?}"),
        }
    }
}
