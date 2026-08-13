//! Read model for the accounts list: one row per account, with its group and
//! its aggregated postings.

use db::idens::{Account, AccountGroup, Posting};
use rusqlite::{Connection, Error as SQLiteError, Row};
use sea_query::{Expr, ExprTrait, Iden, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;
use uuid::Uuid;

use crate::views::Order;

/// One account, as the accounts list shows it.
#[derive(Debug)]
pub struct ResultItem {
    /// The account's unique id.
    pub id: Uuid,
    /// Classification, as [`AccountKind`](domain::account::AccountKind) discriminant.
    pub kind: u8,
    /// Name of the group the account belongs to, or `None` when it is ungrouped.
    pub group_name: Option<String>,
    /// Description of that group; `None` whenever [`group_name`](Self::group_name) is.
    pub group_description: Option<String>,
    /// The account's display name.
    pub name: String,
    /// The account's description, empty when it has none.
    pub description: String,
    /// Denomination, as [`Currency`](domain::money::Currency) discriminant.
    pub currency: u16,
    /// Sum of the account's postings in minor units — the **raw signed**
    /// ledger balance, not a display value.
    pub balance: i64,
    /// How many postings the account has; `0` when it has none.
    pub posting_counts: u32,
}

/// Anything that can go wrong while reading the accounts list.
#[derive(Debug, Error)]
pub enum Error {
    /// The query failed, or a column held a type its field cannot take.
    #[error("{0}")]
    Internal(#[from] SQLiteError),
}

/// Lists accounts matching `filters`, sorted by `by` in `order`.
pub fn view(
    connection: &Connection,
    filters: Filters,
    by: By,
    order: Order,
) -> Result<Vec<ResultItem>, Error> {
    let amount = Expr::col((Posting::Table, Posting::Amount));
    let account_id = Expr::col((Posting::Table, Posting::AccountId));

    let (sql, values) = Query::select()
        .columns([
            (Account::Table, Account::Id),
            (Account::Table, Account::Kind),
            (Account::Table, Account::Name),
            (Account::Table, Account::Description),
            (Account::Table, Account::Currency),
        ])
        // Aliased: unaliased these would land in the row as `name` and
        // `description`, colliding with the account's own columns.
        .expr_as(
            Expr::col((AccountGroup::Table, AccountGroup::Name)),
            Computed::GroupName,
        )
        .expr_as(
            Expr::col((AccountGroup::Table, AccountGroup::Description)),
            Computed::GroupDescription,
        )
        .expr_as(amount.sum().if_null(0), Computed::Balance)
        .expr_as(account_id.count(), Computed::PostingCounts)
        .from(Account::Table)
        .left_join(
            AccountGroup::Table,
            Expr::col((Account::Table, Account::AccountGroupId))
                .equals((AccountGroup::Table, AccountGroup::Id)),
        )
        .left_join(
            Posting::Table,
            Expr::col((Account::Table, Account::Id)).equals((Posting::Table, Posting::AccountId)),
        )
        .apply_if(filters.kinds.filter(|k| !k.is_empty()), |q, v| {
            q.and_where(Expr::col((Account::Table, Account::Kind)).is_in(v));
        })
        .group_by_col((Account::Table, Account::Id))
        .apply(|q| {
            match by {
                By::Name => q.order_by((Account::Table, Account::Name), order.into()),
                By::Balance => q.order_by(Computed::Balance, order.into()),
                By::PostingCounts => q.order_by(Computed::PostingCounts, order.into()),
            };
        })
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = connection.prepare_cached(sql.as_str())?;
    let account_iter = stmt.query_map(&*values.as_params(), ResultItem::from_row)?;

    account_iter.map(|a| a.map_err(Error::from)).collect()
}

/// Narrows which accounts [`view`] returns. [`Default`] keeps them all.
#[derive(Debug, Default)]
pub struct Filters {
    /// Keep only these [`AccountKind`](domain::account::AccountKind) discriminants:
    /// `0` asset, `1` liability, `2` equity, `3` income, `4` expense.
    ///
    /// `None` — and an empty `Vec` — mean every kind. Unknown values match
    /// nothing; the `kind` column is constrained to `0..=4`.
    pub kinds: Option<Vec<u8>>,
}

/// The column [`view`] sorts on.
#[derive(Debug, Default)]
pub enum By {
    /// Signed ledger balance.
    #[default]
    Balance,
    /// Account name.
    Name,
    /// Number of postings.
    PostingCounts,
}

/// The aggregate columns [`view`] adds to each row.
#[derive(Iden)]
enum Computed {
    Balance,
    PostingCounts,
    GroupName,
    GroupDescription,
}

impl ResultItem {
    /// Reads one row of the [`view`] result.
    fn from_row(row: &Row<'_>) -> Result<Self, SQLiteError> {
        Ok(Self {
            id: row.get(Account::Id.unquoted())?,
            kind: row.get(Account::Kind.unquoted())?,
            group_name: row.get(Computed::GroupName.unquoted())?,
            group_description: row.get(Computed::GroupDescription.unquoted())?,
            name: row.get(Account::Name.unquoted())?,
            description: row.get(Account::Description.unquoted())?,
            currency: row.get(Account::Currency.unquoted())?,
            balance: row.get(Computed::Balance.unquoted())?,
            posting_counts: row.get(Computed::PostingCounts.unquoted())?,
        })
    }
}
