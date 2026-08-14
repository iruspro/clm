use db::idens::AccountGroup as AccountGroupIden;
use domain::account_group::{AccountGroup, AccountGroupRepository};
use domain::{AccountGroupId, Name, RepoError};
use rusqlite::{Connection, Error, Row};
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SQLiteAccountGroupRepository<'a> {
    db: &'a Connection,
}

impl<'a> SQLiteAccountGroupRepository<'a> {
    pub fn new(db: &'a Connection) -> Self {
        Self { db }
    }
}

impl<'a> AccountGroupRepository for SQLiteAccountGroupRepository<'a> {
    // region: Create
    fn create(&self, group: &AccountGroup) -> Result<(), RepoError> {
        let (sql, values) = Query::insert()
            .into_table(AccountGroupIden::Table)
            .columns([
                AccountGroupIden::Id,
                AccountGroupIden::Name,
                AccountGroupIden::Description,
            ])
            .values([
                group.id().to_uuid().into(),
                group.name().as_ref().into(),
                group.description().into(),
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
    fn read(&self, id: AccountGroupId) -> Result<AccountGroup, RepoError> {
        let (sql, values) = Query::select()
            .columns([
                (AccountGroupIden::Table, AccountGroupIden::Id),
                (AccountGroupIden::Table, AccountGroupIden::Name),
                (AccountGroupIden::Table, AccountGroupIden::Description),
            ])
            .from(AccountGroupIden::Table)
            .and_where(Expr::col((AccountGroupIden::Table, AccountGroupIden::Id)).eq(id.to_uuid()))
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
    fn update(&self, group: &AccountGroup) -> Result<(), RepoError> {
        let (sql, values) = Query::update()
            .table(AccountGroupIden::Table)
            .values([
                (AccountGroupIden::Name, group.name().as_ref().into()),
                (AccountGroupIden::Description, group.description().into()),
            ])
            .and_where(Expr::col(AccountGroupIden::Id).eq(group.id().to_uuid()))
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

    // region: Delete
    fn delete(&self, id: AccountGroupId) -> Result<(), RepoError> {
        let (sql, values) = Query::delete()
            .from_table(AccountGroupIden::Table)
            .and_where(Expr::col(AccountGroupIden::Id).eq(id.to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let deleted_rows = self
            .db
            .execute(&sql, &*values.as_params())
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        match deleted_rows {
            0 => Err(RepoError::NotFound),
            _ => Ok(()),
        }
    }
    // endregion
}

/// Reconstitutes an [`AccountGroup`] from one row of the `account_group` table.
fn from_row(row: &Row<'_>) -> Result<AccountGroup, Error> {
    Ok(AccountGroup::from_parts(
        AccountGroupId::from(row.get::<_, Uuid>(AccountGroupIden::Id.unquoted())?),
        Name::new_unchecked(row.get::<_, String>(AccountGroupIden::Name.unquoted())?),
        row.get(AccountGroupIden::Description.unquoted())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sample group.
    fn sample_group(id: AccountGroupId, name: &str, description: &str) -> AccountGroup {
        AccountGroup::new(
            id,
            Name::new(name).expect("non-empty name"),
            description.to_string(),
        )
    }

    #[test]
    fn create_then_read_round_trips_every_field() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let id = AccountGroupId::new();

        repo.create(&sample_group(id, "Bank", "high-street accounts"))
            .expect("create group");

        let loaded = repo.read(id).expect("read the group just created");
        assert_eq!(loaded.id(), id);
        assert_eq!(loaded.name().as_ref(), "Bank");
        assert_eq!(loaded.description(), "high-street accounts");
    }

    #[test]
    fn update_then_read_returns_the_edited_group() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let id = AccountGroupId::new();
        repo.create(&sample_group(id, "Bank", "high-street accounts"))
            .expect("create group");

        // Renaming is what the id buys: under a name-keyed table this edit had
        // no row to match.
        let mut group = repo.read(id).expect("read the group just created");
        group.set_name(Name::new("High street").expect("non-empty name"));
        group.set_description(String::new());
        repo.update(&group).expect("update group");

        let loaded = repo.read(id).expect("read the updated group");
        assert_eq!(loaded.name().as_ref(), "High street");
        assert_eq!(loaded.description(), "");
    }

    #[test]
    fn update_touches_only_the_matching_row() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let target_id = AccountGroupId::new();
        let bystander_id = AccountGroupId::new();
        repo.create(&sample_group(target_id, "Bank", "high-street accounts"))
            .expect("create target");
        repo.create(&sample_group(
            bystander_id,
            "Crypto",
            "self-custodied wallets",
        ))
        .expect("create bystander");

        let mut target = repo.read(target_id).expect("read target");
        target.set_description("online only".to_string());
        repo.update(&target).expect("update target");

        let bystander = repo.read(bystander_id).expect("read bystander");
        assert_eq!(bystander.name().as_ref(), "Crypto");
        assert_eq!(bystander.description(), "self-custodied wallets");

        // The row it did match still changed — otherwise this test would also
        // pass for an `update` that does nothing at all.
        let target = repo.read(target_id).expect("read target");
        assert_eq!(target.description(), "online only");
    }

    #[test]
    fn delete_removes_only_the_matching_row() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let target_id = AccountGroupId::new();
        let bystander_id = AccountGroupId::new();
        repo.create(&sample_group(target_id, "Bank", "high-street accounts"))
            .expect("create target");
        repo.create(&sample_group(
            bystander_id,
            "Crypto",
            "self-custodied wallets",
        ))
        .expect("create bystander");

        repo.delete(target_id).expect("delete target");

        match repo.read(target_id) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected the deleted group to be gone, got {other:?}"),
        }
        let bystander = repo.read(bystander_id).expect("read bystander");
        assert_eq!(bystander.description(), "self-custodied wallets");
    }

    #[test]
    fn read_reports_not_found_for_an_unknown_id() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        // One stored group, queried by a different id: the miss has to come
        // from the `WHERE`, not from an empty table.
        repo.create(&sample_group(
            AccountGroupId::new(),
            "Bank",
            "high-street accounts",
        ))
        .expect("create group");

        match repo.read(AccountGroupId::new()) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn update_reports_not_found_for_an_unknown_id() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let stored_id = AccountGroupId::new();
        repo.create(&sample_group(stored_id, "Bank", "high-street accounts"))
            .expect("create group");

        let unknown = sample_group(AccountGroupId::new(), "Crypto", "self-custodied wallets");
        match repo.update(&unknown) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }

        // And it left the row it did not match alone.
        let stored = repo.read(stored_id).expect("read the stored group");
        assert_eq!(stored.description(), "high-street accounts");
    }

    #[test]
    fn delete_reports_not_found_for_an_unknown_id() {
        let db = db::testing::test_db();
        let repo = SQLiteAccountGroupRepository::new(&db);
        let stored_id = AccountGroupId::new();
        repo.create(&sample_group(stored_id, "Bank", "high-street accounts"))
            .expect("create group");

        match repo.delete(AccountGroupId::new()) {
            Err(RepoError::NotFound) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }

        // And it left the row it did not match alone.
        repo.read(stored_id).expect("read the stored group");
    }
}
