//! Read model for account groups: one row per group, with how many accounts
//! it holds.

use rusqlite::{Connection, Error as SQLiteError, Row};
use sea_query::{
    Expr, ExprTrait, Iden, Order as SeaOrder, Query, SelectStatement, SqliteQueryBuilder,
};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;
use uuid::Uuid;

use crate::db::idens::{Account, AccountGroup};

/// One group, as the group screens show it.
#[derive(Debug)]
pub struct ResultItem {
    /// The group's unique id.
    pub id: Uuid,
    /// The group's display name.
    pub name: String,
    /// The group's description, empty when it has none.
    pub description: String,
    /// How many accounts point at the group; `0` when it holds none.
    pub account_counts: u32,
}

/// Anything that can go wrong while reading groups.
#[derive(Debug, Error)]
pub enum Error {
    /// The query failed, or a column held a type its field cannot take.
    #[error("{0}")]
    Internal(#[from] SQLiteError),
}

/// Lists every group, by name.
pub fn view(connection: &Connection) -> Result<Vec<ResultItem>, Error> {
    let (sql, values) = select().build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let groups = stmt.query_map(&*values.as_params(), ResultItem::from_row)?;

    groups.map(|group| group.map_err(Error::from)).collect()
}

/// Reads one group, or `None` when no group carries that id.
pub fn one(connection: &Connection, id: Uuid) -> Result<Option<ResultItem>, Error> {
    let (sql, values) = select()
        .and_where(Expr::col((AccountGroup::Table, AccountGroup::Id)).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let mut groups = stmt.query_map(&*values.as_params(), ResultItem::from_row)?;

    groups.next().transpose().map_err(Error::from)
}

/// The query both readers run: every group, with its accounts counted.
///
/// The join is a left join and the count is over the account side, so a group
/// holding nothing still comes back, with a count of zero.
fn select() -> SelectStatement {
    let account_id = Expr::col((Account::Table, Account::Id));

    Query::select()
        .columns([
            (AccountGroup::Table, AccountGroup::Id),
            (AccountGroup::Table, AccountGroup::Name),
            (AccountGroup::Table, AccountGroup::Description),
        ])
        .expr_as(account_id.count(), Computed::AccountCounts)
        .from(AccountGroup::Table)
        .left_join(
            Account::Table,
            Expr::col((AccountGroup::Table, AccountGroup::Id))
                .equals((Account::Table, Account::AccountGroupId)),
        )
        .group_by_col((AccountGroup::Table, AccountGroup::Id))
        .order_by((AccountGroup::Table, AccountGroup::Name), SeaOrder::Asc)
        .take()
}

/// The aggregate column the query adds to each row.
#[derive(Iden)]
enum Computed {
    AccountCounts,
}

impl ResultItem {
    /// Reads one row of the result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            id: row.get(AccountGroup::Id.unquoted())?,
            name: row.get(AccountGroup::Name.unquoted())?,
            description: row.get(AccountGroup::Description.unquoted())?,
            account_counts: row.get(Computed::AccountCounts.unquoted())?,
        })
    }
}
