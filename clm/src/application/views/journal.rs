//! Read model for the journal: one row per entry, and the postings behind one.

use rusqlite::{Connection, Error as SQLiteError, Row};
use sea_query::{Expr, ExprTrait, Iden, Order as SeaOrder, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;
use uuid::Uuid;

use crate::db::idens::{Account, JournalEntry, Posting};

/// One journal entry, as the transactions list shows it.
#[derive(Debug)]
pub struct EntryItem {
    /// The entry's unique id.
    pub id: Uuid,
    /// The day it happened, as stored: ISO-8601 (`YYYY-MM-DD`).
    pub date: String,
    /// What the entry says about itself, empty when it says nothing.
    pub description: String,
    /// How many postings the entry moves money between.
    pub posting_counts: u32,
}

/// One posting of an entry, with the account it lands in.
#[derive(Debug)]
pub struct PostingItem {
    /// The account's display name.
    pub account: String,
    /// Denomination, as [`Currency`](crate::domain::money::Currency) discriminant.
    pub currency: u16,
    /// The posting in minor units — **signed**: a debit is positive, a credit
    /// negative.
    pub amount: i64,
}

/// Anything that can go wrong while reading the journal.
#[derive(Debug, Error)]
pub enum Error {
    /// The query failed, or a column held a type its field cannot take.
    #[error("{0}")]
    Internal(#[from] SQLiteError),
}

/// Lists the `limit` most recent entries, newest first.
///
/// Ties on the date are broken by id, which is a UUID v7 and so orders by the
/// moment the entry was written — an entry added later sorts later.
pub fn view(connection: &Connection, limit: u64) -> Result<Vec<EntryItem>, Error> {
    let account_id = Expr::col((Posting::Table, Posting::AccountId));

    let (sql, values) = Query::select()
        .columns([
            (JournalEntry::Table, JournalEntry::Id),
            (JournalEntry::Table, JournalEntry::Date),
            (JournalEntry::Table, JournalEntry::Description),
        ])
        .expr_as(account_id.count(), Computed::PostingCounts)
        .from(JournalEntry::Table)
        // Left join, so an entry with no postings is still listed rather than
        // quietly dropped — it is a book-keeping mistake worth seeing.
        .left_join(
            Posting::Table,
            Expr::col((JournalEntry::Table, JournalEntry::Id))
                .equals((Posting::Table, Posting::JournalEntryId)),
        )
        .group_by_col((JournalEntry::Table, JournalEntry::Id))
        .order_by((JournalEntry::Table, JournalEntry::Date), SeaOrder::Desc)
        .order_by((JournalEntry::Table, JournalEntry::Id), SeaOrder::Desc)
        .limit(limit)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let entries = stmt.query_map(&*values.as_params(), EntryItem::from_row)?;

    entries.map(|entry| entry.map_err(Error::from)).collect()
}

/// Reads the postings of one entry, debits before credits.
pub fn postings(connection: &Connection, entry_id: Uuid) -> Result<Vec<PostingItem>, Error> {
    let (sql, values) = Query::select()
        .columns([
            (Account::Table, Account::Name),
            (Account::Table, Account::Currency),
        ])
        .column((Posting::Table, Posting::Amount))
        .from(Posting::Table)
        .inner_join(
            Account::Table,
            Expr::col((Posting::Table, Posting::AccountId)).equals((Account::Table, Account::Id)),
        )
        .and_where(Expr::col((Posting::Table, Posting::JournalEntryId)).eq(entry_id))
        // Signed amounts, so descending puts the debits first.
        .order_by((Posting::Table, Posting::Amount), SeaOrder::Desc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let postings = stmt.query_map(&*values.as_params(), PostingItem::from_row)?;

    postings
        .map(|posting| posting.map_err(Error::from))
        .collect()
}

/// One line of an account's statement: an entry, and what it moved here.
#[derive(Debug)]
pub struct StatementItem {
    /// The entry the posting belongs to.
    pub entry_id: Uuid,
    /// The day it happened, as stored: ISO-8601 (`YYYY-MM-DD`).
    pub date: String,
    /// What the entry says about itself, empty when it says nothing.
    pub description: String,
    /// The posting in minor units — **signed**: a debit is positive, a credit
    /// negative.
    pub amount: i64,
}

/// Lists what the `limit` most recent entries moved through one account,
/// newest first.
pub fn statement(
    connection: &Connection,
    account_id: Uuid,
    limit: u64,
) -> Result<Vec<StatementItem>, Error> {
    let (sql, values) = Query::select()
        .columns([
            (JournalEntry::Table, JournalEntry::Date),
            (JournalEntry::Table, JournalEntry::Description),
        ])
        .expr_as(
            Expr::col((JournalEntry::Table, JournalEntry::Id)),
            Computed::EntryId,
        )
        .column((Posting::Table, Posting::Amount))
        .from(Posting::Table)
        .inner_join(
            JournalEntry::Table,
            Expr::col((Posting::Table, Posting::JournalEntryId))
                .equals((JournalEntry::Table, JournalEntry::Id)),
        )
        .and_where(Expr::col((Posting::Table, Posting::AccountId)).eq(account_id))
        .order_by((JournalEntry::Table, JournalEntry::Date), SeaOrder::Desc)
        .order_by((JournalEntry::Table, JournalEntry::Id), SeaOrder::Desc)
        .limit(limit)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let lines = stmt.query_map(&*values.as_params(), StatementItem::from_row)?;

    lines.map(|line| line.map_err(Error::from)).collect()
}

/// The aggregate column [`view`] adds to each row.
#[derive(Iden)]
enum Computed {
    PostingCounts,
    /// Aliased: unaliased it would land in the row as `id`, next to the
    /// account's own.
    EntryId,
}

impl StatementItem {
    /// Reads one row of the [`statement`] result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            entry_id: row.get(Computed::EntryId.unquoted())?,
            date: row.get(JournalEntry::Date.unquoted())?,
            description: row.get(JournalEntry::Description.unquoted())?,
            amount: row.get(Posting::Amount.unquoted())?,
        })
    }
}

impl EntryItem {
    /// Reads one row of the [`view`] result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            id: row.get(JournalEntry::Id.unquoted())?,
            date: row.get(JournalEntry::Date.unquoted())?,
            description: row.get(JournalEntry::Description.unquoted())?,
            posting_counts: row.get(Computed::PostingCounts.unquoted())?,
        })
    }
}

impl PostingItem {
    /// Reads one row of the [`postings`] result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            account: row.get(Account::Name.unquoted())?,
            currency: row.get(Account::Currency.unquoted())?,
            amount: row.get(Posting::Amount.unquoted())?,
        })
    }
}
