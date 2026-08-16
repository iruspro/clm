//! Read model for the summary: what every kind of account holds, per currency.

use rusqlite::{Connection, Error as SQLiteError, Row};
use sea_query::{Expr, ExprTrait, Iden, Order as SeaOrder, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;

use crate::db::idens::{Account, Posting};

/// What one kind of account holds in one currency.
#[derive(Debug)]
pub struct ResultItem {
    /// Classification, as [`AccountKind`](crate::domain::account::AccountKind) discriminant.
    pub kind: u8,
    /// Denomination, as [`Currency`](crate::domain::money::Currency) discriminant.
    pub currency: u16,
    /// Sum of the postings in minor units — the **raw signed** total, debits
    /// positive, before any kind's own reading of it.
    pub total: i64,
    /// How many accounts of this kind and currency there are.
    pub account_counts: u32,
}

/// Anything that can go wrong while reading the summary.
#[derive(Debug, Error)]
pub enum Error {
    /// The query failed, or a column held a type its field cannot take.
    #[error("{0}")]
    Internal(#[from] SQLiteError),
}

/// Totals every account, grouped by kind and currency.
///
/// The join is a left join, so a kind with accounts but no postings still comes
/// back — at zero, which is a fact worth showing.
pub fn view(connection: &Connection) -> Result<Vec<ResultItem>, Error> {
    let amount = Expr::col((Posting::Table, Posting::Amount));
    let account_id = Expr::col((Account::Table, Account::Id));

    let (sql, values) = Query::select()
        .columns([
            (Account::Table, Account::Kind),
            (Account::Table, Account::Currency),
        ])
        .expr_as(amount.sum().if_null(0), Computed::Total)
        .expr_as(account_id.count_distinct(), Computed::AccountCounts)
        .from(Account::Table)
        .left_join(
            Posting::Table,
            Expr::col((Account::Table, Account::Id)).equals((Posting::Table, Posting::AccountId)),
        )
        .add_group_by([
            Expr::col((Account::Table, Account::Kind)),
            Expr::col((Account::Table, Account::Currency)),
        ])
        .order_by((Account::Table, Account::Currency), SeaOrder::Asc)
        .order_by((Account::Table, Account::Kind), SeaOrder::Asc)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let totals = stmt.query_map(&*values.as_params(), ResultItem::from_row)?;

    totals.map(|total| total.map_err(Error::from)).collect()
}

/// The aggregate columns the query adds to each row.
#[derive(Iden)]
enum Computed {
    Total,
    AccountCounts,
}

impl ResultItem {
    /// Reads one row of the [`view`] result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            kind: row.get(Account::Kind.unquoted())?,
            currency: row.get(Account::Currency.unquoted())?,
            total: row.get(Computed::Total.unquoted())?,
            account_counts: row.get(Computed::AccountCounts.unquoted())?,
        })
    }
}
