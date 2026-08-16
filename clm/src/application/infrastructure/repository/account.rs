use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use rusqlite::{Connection, Error, Row};
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use crate::db::idens::Account as AccountIden;
use crate::domain::account::{Account, AccountKind, AccountRepository};
use crate::domain::money::Currency;
use crate::domain::{AccountGroupId, AccountId, Name, RepoError};

pub struct SQLiteAccountRepository<'a> {
    db: &'a Connection,
}

impl<'a> SQLiteAccountRepository<'a> {
    pub fn new(db: &'a Connection) -> Self {
        Self { db }
    }
}

impl<'a> AccountRepository for SQLiteAccountRepository<'a> {
    // region: Create
    fn create(&self, account: &Account) -> Result<(), RepoError> {
        let group_id: Option<Uuid> = account.group_id().map(AccountGroupId::to_uuid);

        let (sql, values) = Query::insert()
            .into_table(AccountIden::Table)
            .columns([
                AccountIden::Id,
                AccountIden::Name,
                AccountIden::Description,
                AccountIden::Kind,
                AccountIden::Currency,
                AccountIden::AccountGroupId,
            ])
            .values([
                account.id().to_uuid().into(),
                account.name().as_ref().into(),
                account.description().into(),
                account.kind().as_u8().into(),
                account.currency().as_u16().into(),
                group_id.into(),
            ])
            .map_err(|err| RepoError::Storage(err.to_string()))?
            .build_rusqlite(SqliteQueryBuilder);

        self.db
            .execute(&sql, &*values.as_params())
            .map(|_| ())
            .map_err(|err| RepoError::Storage(err.to_string()))
    }
    // endregion

    // region: Read
    fn read(&self, id: crate::domain::AccountId) -> Result<Account, RepoError> {
        let (sql, values) = Query::select()
            .columns([
                (AccountIden::Table, AccountIden::Id),
                (AccountIden::Table, AccountIden::Name),
                (AccountIden::Table, AccountIden::Description),
                (AccountIden::Table, AccountIden::Kind),
                (AccountIden::Table, AccountIden::Currency),
                (AccountIden::Table, AccountIden::AccountGroupId),
            ])
            .from(AccountIden::Table)
            .and_where(Expr::col((AccountIden::Table, AccountIden::Id)).eq(id.to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = self
            .db
            .prepare_cached(sql.as_str())
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        stmt.query_one(&*values.as_params(), from_row)
            .map_err(|err| match err {
                Error::QueryReturnedNoRows => RepoError::NotFound,
                err => RepoError::Storage(err.to_string()),
            })
    }
    // endregion

    // region: Update
    fn update(&self, account: &Account) -> Result<(), RepoError> {
        let group_id: Option<Uuid> = account.group_id().map(AccountGroupId::to_uuid);

        let (sql, values) = Query::update()
            .table(AccountIden::Table)
            .values([
                (AccountIden::Name, account.name().as_ref().into()),
                (AccountIden::Description, account.description().into()),
                (AccountIden::Kind, account.kind().as_u8().into()),
                (AccountIden::Currency, account.currency().as_u16().into()),
                (AccountIden::AccountGroupId, group_id.into()),
            ])
            .and_where(Expr::col(AccountIden::Id).eq(account.id().to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let updated_rows = self
            .db
            .execute(&sql, &*values.as_params())
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        // The id is the primary key, so this is 0 or 1: no row means there was
        // nothing to update.
        match updated_rows {
            0 => Err(RepoError::NotFound),
            _ => Ok(()),
        }
    }
    // endregion
}

/// Reconstitutes an [`Account`] from one row of the `account` table.
fn from_row(row: &Row<'_>) -> Result<Account, Error> {
    let group_id: Option<Uuid> = row.get(AccountIden::AccountGroupId.unquoted())?;

    Ok(Account::from_parts(
        AccountId::from(row.get::<_, Uuid>(AccountIden::Id.unquoted())?),
        row.get::<_, SqlAccountKind>(AccountIden::Kind.unquoted())?
            .0,
        row.get::<_, SqlCurrency>(AccountIden::Currency.unquoted())?
            .0,
        Name::new_unchecked(row.get::<_, String>(AccountIden::Name.unquoted())?),
        row.get(AccountIden::Description.unquoted())?,
        group_id.map(AccountGroupId::from),
    ))
}

/// Reads an [`AccountKind`] from its integer column.
struct SqlAccountKind(AccountKind);

impl FromSql for SqlAccountKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        AccountKind::try_from(u8::column_result(value)?)
            .map(SqlAccountKind)
            .map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

/// Reads a [`Currency`] from its integer column.
struct SqlCurrency(Currency);

impl FromSql for SqlCurrency {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Currency::try_from(u16::column_result(value)?)
            .map(SqlCurrency)
            .map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inserts a group for accounts to join, returning the id they reference.
    ///
    /// Raw SQL rather than the group repository: these tests are about the
    /// account repository, and should not fail because a neighbouring adapter
    /// broke.
    fn seed_group(db: &Connection, name: &str) -> AccountGroupId {
        let id = AccountGroupId::new();
        db.execute(
            "INSERT INTO account_group (id, name, description) VALUES (?1, ?2, '')",
            (id.to_uuid(), name),
        )
        .expect("seed account group");

        id
    }

    /// A sample account, ungrouped. `Currency::BTC` and `AccountKind::Expense`
    /// are deliberately not the first variant, so a mix-up between a value and
    /// a default discriminant of 0 would show up.
    fn sample_account(id: AccountId) -> Account {
        Account::new(
            id,
            AccountKind::Expense,
            Currency::BTC,
            Name::new("Cash").expect("non-empty name"),
            "petty cash".to_string(),
            None,
        )
    }

    #[test]
    fn create_then_read_round_trips_every_field() {
        let db = crate::db::testing::test_db();
        let repo = SQLiteAccountRepository::new(&db);
        let id = AccountId::new();

        repo.create(&sample_account(id)).expect("create account");

        let loaded = repo.read(id).expect("read the account just created");
        assert_eq!(loaded.id(), id);
        assert_eq!(loaded.name().as_ref(), "Cash");
        assert_eq!(loaded.description(), "petty cash");
        assert_eq!(loaded.kind(), AccountKind::Expense);
        assert_eq!(loaded.currency(), Currency::BTC);
        assert_eq!(loaded.group_id(), None);
    }

    #[test]
    fn update_then_read_returns_the_edited_account() {
        let db = crate::db::testing::test_db();
        let repo = SQLiteAccountRepository::new(&db);
        let id = AccountId::new();
        let group_id = seed_group(&db, "Bank");
        repo.create(&sample_account(id)).expect("create account");

        let mut account = repo.read(id).expect("read the account just created");
        account.set_name(Name::new("Wallet").expect("non-empty name"));
        account.set_description(String::new());
        account.set_group_id(Some(group_id));

        repo.update(&account).expect("update account");

        let loaded = repo.read(id).expect("read the updated account");
        assert_eq!(loaded.name().as_ref(), "Wallet");
        assert_eq!(loaded.description(), "");
        assert_eq!(loaded.group_id(), Some(group_id));
        // Untouched by the edit: `Account` has no setter for either, and the
        // update must not disturb the columns it rewrites unchanged.
        assert_eq!(loaded.kind(), AccountKind::Expense);
        assert_eq!(loaded.currency(), Currency::BTC);
    }

    #[test]
    fn update_touches_only_the_matching_row() {
        let db = crate::db::testing::test_db();
        let repo = SQLiteAccountRepository::new(&db);

        let target_id = AccountId::new();
        repo.create(&sample_account(target_id))
            .expect("create target");

        // A bystander differing from the target in *every* column, so an
        // `UPDATE` that forgot its `WHERE` — and so copied the target over both
        // rows — cannot slip past unnoticed.
        let bystander_id = AccountId::new();
        let group_id = seed_group(&db, "Bank");
        repo.create(&Account::new(
            bystander_id,
            AccountKind::Asset,
            Currency::EUR,
            Name::new("Savings").expect("non-empty name"),
            "rainy day".to_string(),
            Some(group_id),
        ))
        .expect("create bystander");

        let mut target = repo.read(target_id).expect("read target");
        target.set_name(Name::new("Wallet").expect("non-empty name"));
        repo.update(&target).expect("update target");

        let bystander = repo.read(bystander_id).expect("read bystander");
        assert_eq!(bystander.name().as_ref(), "Savings");
        assert_eq!(bystander.description(), "rainy day");
        assert_eq!(bystander.kind(), AccountKind::Asset);
        assert_eq!(bystander.currency(), Currency::EUR);
        assert_eq!(bystander.group_id(), Some(group_id));

        // The row it did match still changed — otherwise this test would also
        // pass for an `update` that does nothing at all.
        let target = repo.read(target_id).expect("read target");
        assert_eq!(target.name().as_ref(), "Wallet");
    }

    #[test]
    fn read_reports_not_found_for_an_unknown_id() {
        let db = crate::db::testing::test_db();
        let repo = SQLiteAccountRepository::new(&db);
        // One stored account, queried by a different id: the miss has to come
        // from the `WHERE`, not from an empty table.
        repo.create(&sample_account(AccountId::new()))
            .expect("create account");

        match repo.read(AccountId::new()) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn update_reports_not_found_for_an_unknown_id() {
        let db = crate::db::testing::test_db();
        let repo = SQLiteAccountRepository::new(&db);
        let stored_id = AccountId::new();
        repo.create(&sample_account(stored_id))
            .expect("create account");

        // Same account, a fresh id — nothing to update.
        let unknown = sample_account(AccountId::new());
        match repo.update(&unknown) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }

        // And it left the row it did not match alone.
        let stored = repo.read(stored_id).expect("read the stored account");
        assert_eq!(stored.name().as_ref(), "Cash");
    }
}
