use rusqlite::{Connection, Error, Row};
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use time::Date;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use uuid::Uuid;

use crate::db::idens::{Account, JournalEntry, Posting as PostingIden};
use crate::domain::journal::{BalancedPostings, Entry, EntryRepository, Magnitude, Posting};
use crate::domain::money::{Currency, Money};
use crate::domain::{AccountId, EntryId, RepoError, Side};

/// How the `date` column is written and read: ISO-8601, as the schema says.
const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

/// The journal in SQLite: an entry row, plus one row per posting.
///
/// A posting stores only its signed amount — the currency comes from the
/// account it points at, which is therefore the authority on it. Writing a
/// posting in another currency than its account's would be read back in the
/// account's, so the caller is expected to keep them in step.
pub struct SQLiteEntryRepository<'a> {
    db: &'a Connection,
}

impl<'a> SQLiteEntryRepository<'a> {
    pub fn new(db: &'a Connection) -> Self {
        Self { db }
    }

    /// Writes the postings of `entry`, which the caller has already cleared of
    /// any previous ones.
    fn write_postings(&self, entry: &Entry) -> Result<(), RepoError> {
        for posting in entry.postings().clone().into_vec() {
            let (sql, values) = Query::insert()
                .into_table(PostingIden::Table)
                .columns([
                    PostingIden::JournalEntryId,
                    PostingIden::AccountId,
                    PostingIden::Amount,
                ])
                .values([
                    entry.id().to_uuid().into(),
                    posting.account_id().to_uuid().into(),
                    // Signed: the side lives in the sign, as the schema says.
                    posting.signed().amount().into(),
                ])
                .map_err(sea_query_error)?
                .build_rusqlite(SqliteQueryBuilder);

            self.db
                .execute(&sql, &*values.as_params())
                .map_err(storage)?;
        }

        Ok(())
    }

    /// Removes every posting of one entry.
    fn clear_postings(&self, id: EntryId) -> Result<(), RepoError> {
        let (sql, values) = Query::delete()
            .from_table(PostingIden::Table)
            .and_where(Expr::col(PostingIden::JournalEntryId).eq(id.to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        self.db
            .execute(&sql, &*values.as_params())
            .map(|_| ())
            .map_err(storage)
    }

    /// Reads the postings of one entry, taking each one's currency from the
    /// account it points at.
    fn read_postings(&self, id: EntryId) -> Result<Vec<Posting>, RepoError> {
        let (sql, values) = Query::select()
            .columns([
                (PostingIden::Table, PostingIden::AccountId),
                (PostingIden::Table, PostingIden::Amount),
            ])
            .column((Account::Table, Account::Currency))
            .from(PostingIden::Table)
            .inner_join(
                Account::Table,
                Expr::col((PostingIden::Table, PostingIden::AccountId))
                    .equals((Account::Table, Account::Id)),
            )
            .and_where(
                Expr::col((PostingIden::Table, PostingIden::JournalEntryId)).eq(id.to_uuid()),
            )
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = self.db.prepare_cached(sql.as_str()).map_err(storage)?;
        let postings = stmt
            .query_map(&*values.as_params(), posting_from_row)
            .map_err(storage)?;

        postings
            .collect::<Result<Vec<_>, Error>>()
            .map_err(storage)?
            .into_iter()
            .collect::<Result<Vec<_>, RepoError>>()
    }
}

impl EntryRepository for SQLiteEntryRepository<'_> {
    // region: Create
    /// Stores the entry and its postings together: either the whole entry
    /// lands or none of it does.
    fn create(&self, entry: &Entry) -> Result<(), RepoError> {
        let tx = self.db.unchecked_transaction().map_err(storage)?;

        let (sql, values) = Query::insert()
            .into_table(JournalEntry::Table)
            .columns([
                JournalEntry::Id,
                JournalEntry::Date,
                JournalEntry::Description,
            ])
            .values([
                entry.id().to_uuid().into(),
                to_text(entry.date())?.into(),
                entry.description().into(),
            ])
            .map_err(sea_query_error)?
            .build_rusqlite(SqliteQueryBuilder);

        self.db
            .execute(&sql, &*values.as_params())
            .map_err(storage)?;
        self.write_postings(entry)?;

        tx.commit().map_err(storage)
    }
    // endregion

    // region: Read
    fn read(&self, entry_id: EntryId) -> Result<Entry, RepoError> {
        let (sql, values) = Query::select()
            .columns([
                (JournalEntry::Table, JournalEntry::Id),
                (JournalEntry::Table, JournalEntry::Date),
                (JournalEntry::Table, JournalEntry::Description),
            ])
            .from(JournalEntry::Table)
            .and_where(Expr::col((JournalEntry::Table, JournalEntry::Id)).eq(entry_id.to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = self.db.prepare_cached(sql.as_str()).map_err(storage)?;
        let (id, date, description) = stmt
            .query_one(&*values.as_params(), entry_from_row)
            .map_err(|err| match err {
                Error::QueryReturnedNoRows => RepoError::NotFound,
                err => storage(err),
            })?;

        let date = from_text(&date)?;
        // Trusted: the postings were balanced when they were written.
        let postings = BalancedPostings::new_unchecked(self.read_postings(entry_id)?);

        Ok(Entry::from_parts(id, date, description, postings))
    }
    // endregion

    // region: Update
    /// Replaces the entry and all of its postings.
    ///
    /// The postings are rewritten wholesale rather than matched up one by one:
    /// a posting has no identity of its own, so "the same posting, edited" is
    /// not a thing the schema can express.
    fn update(&self, entry: &Entry) -> Result<(), RepoError> {
        let tx = self.db.unchecked_transaction().map_err(storage)?;

        let (sql, values) = Query::update()
            .table(JournalEntry::Table)
            .values([
                (JournalEntry::Date, to_text(entry.date())?.into()),
                (JournalEntry::Description, entry.description().into()),
            ])
            .and_where(Expr::col(JournalEntry::Id).eq(entry.id().to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let updated_rows = self
            .db
            .execute(&sql, &*values.as_params())
            .map_err(storage)?;

        // The id is the primary key, so this is 0 or 1: no row means there was
        // nothing to update.
        if updated_rows == 0 {
            return Err(RepoError::NotFound);
        }

        self.clear_postings(entry.id())?;
        self.write_postings(entry)?;

        tx.commit().map_err(storage)
    }
    // endregion

    // region: Delete
    /// Removes the entry and its postings.
    ///
    /// The postings go first and explicitly: the schema cascades them, but
    /// SQLite only acts on that with `PRAGMA foreign_keys` on.
    fn delete(&self, entry_id: EntryId) -> Result<(), RepoError> {
        let tx = self.db.unchecked_transaction().map_err(storage)?;

        self.clear_postings(entry_id)?;

        let (sql, values) = Query::delete()
            .from_table(JournalEntry::Table)
            .and_where(Expr::col(JournalEntry::Id).eq(entry_id.to_uuid()))
            .build_rusqlite(SqliteQueryBuilder);

        let deleted_rows = self
            .db
            .execute(&sql, &*values.as_params())
            .map_err(storage)?;

        if deleted_rows == 0 {
            return Err(RepoError::NotFound);
        }

        tx.commit().map_err(storage)
    }
    // endregion
}

/// Reads the entry's own columns; its postings are a query of their own.
fn entry_from_row(row: &Row<'_>) -> Result<(EntryId, String, String), Error> {
    Ok((
        EntryId::from(row.get::<_, Uuid>(JournalEntry::Id.unquoted())?),
        row.get(JournalEntry::Date.unquoted())?,
        row.get(JournalEntry::Description.unquoted())?,
    ))
}

/// Rebuilds one posting: the sign says which side it is on, the account says
/// which currency it is in.
///
/// The outer `Result` is the read itself; the inner one is what the row says,
/// which can be a currency this build does not know.
fn posting_from_row(row: &Row<'_>) -> Result<Result<Posting, RepoError>, Error> {
    let account_id: Uuid = row.get(PostingIden::AccountId.unquoted())?;
    let amount: i64 = row.get(PostingIden::Amount.unquoted())?;
    let currency: u16 = row.get(Account::Currency.unquoted())?;

    let Ok(currency) = Currency::try_from(currency) else {
        return Ok(Err(RepoError::Storage(format!(
            "unknown currency {currency}"
        ))));
    };

    let side = if amount < 0 {
        Side::Credit
    } else {
        Side::Debit
    };
    // Trusted: the amount was a magnitude when it was written, and the schema
    // forbids a zero one.
    let magnitude = Magnitude::new_unchecked(Money::new(amount.abs(), currency));

    Ok(Ok(Posting::new(
        AccountId::from(account_id),
        side,
        magnitude,
    )))
}

/// Writes a date the way the `date` column expects it.
fn to_text(date: Date) -> Result<String, RepoError> {
    date.format(&DATE_FORMAT)
        .map_err(|err| RepoError::Storage(err.to_string()))
}

/// Reads a date back out of the `date` column.
fn from_text(text: &str) -> Result<Date, RepoError> {
    Date::parse(text, &DATE_FORMAT).map_err(|err| RepoError::Storage(err.to_string()))
}

/// Every rusqlite failure that is not "no such row" is a storage failure.
fn storage(err: Error) -> RepoError {
    RepoError::Storage(err.to_string())
}

/// A query that could not even be built — a mismatch between its columns and
/// its values, which is a bug here rather than a fault of the database.
fn sea_query_error(err: sea_query::error::Error) -> RepoError {
    RepoError::Storage(err.to_string())
}
