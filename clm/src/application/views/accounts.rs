//! Read model for the accounts list: one row per account, with its group and
//! its aggregated postings.

use rusqlite::{Connection, Error as SQLiteError, Row};
use sea_query::{Expr, ExprTrait, Iden, JoinType, Query, SelectStatement, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;
use uuid::Uuid;

use crate::application::views::Order;
use crate::db::idens::{Account, AccountGroup, JournalEntry, Posting};

/// One account, as the accounts list shows it.
#[derive(Debug)]
pub struct ResultItem {
    /// The account's unique id.
    pub id: Uuid,
    /// Classification, as [`AccountKind`](crate::domain::account::AccountKind) discriminant.
    pub kind: u8,
    /// Id of the group the account belongs to, or `None` when it is ungrouped.
    pub group_id: Option<Uuid>,
    /// Name of that group; `None` whenever [`group_id`](Self::group_id) is.
    pub group_name: Option<String>,
    /// Description of that group; `None` whenever [`group_name`](Self::group_name) is.
    pub group_description: Option<String>,
    /// The account's display name.
    pub name: String,
    /// The account's description, empty when it has none.
    pub description: String,
    /// Denomination, as [`Currency`](crate::domain::money::Currency) discriminant.
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
    // Moved out ahead of the builder, which takes the other fields one by one.
    let dates = filters.dates;

    let (sql, values) = Query::select()
        .columns([
            (Account::Table, Account::Id),
            (Account::Table, Account::Kind),
            (Account::Table, Account::Name),
            (Account::Table, Account::Description),
            (Account::Table, Account::Currency),
            (Account::Table, Account::AccountGroupId),
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
        // The postings, narrowed to `dates` when there are any. The narrowed
        // set answers to the posting table's own name, so the columns above
        // read the same either way — and because it is still a left join, an
        // account with nothing inside the range lists at zero rather than
        // dropping out of the result.
        .apply(|q| {
            let on = Expr::col((Account::Table, Account::Id))
                .equals((Posting::Table, Posting::AccountId));

            match dates {
                None => q.left_join(Posting::Table, on),
                Some(dates) => {
                    q.join_subquery(JoinType::LeftJoin, postings_in(dates), Posting::Table, on)
                }
            };
        })
        .apply_if(filters.kinds.filter(|k| !k.is_empty()), |q, v| {
            q.and_where(Expr::col((Account::Table, Account::Kind)).is_in(v));
        })
        .apply_if(filters.ids.filter(|i| !i.is_empty()), |q, v| {
            q.and_where(Expr::col((Account::Table, Account::Id)).is_in(v));
        })
        .apply_if(filters.group_ids.filter(|g| !g.is_empty()), |q, v| {
            q.and_where(Expr::col((Account::Table, Account::AccountGroupId)).is_in(v));
        })
        .group_by_col((Account::Table, Account::Id))
        .apply(|q| {
            match by {
                By::Name => q.order_by((Account::Table, Account::Name), order.into()),
                By::Balance => q.order_by(Computed::Balance, order.into()),
                By::Postings => q.order_by(Computed::PostingCounts, order.into()),
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
    /// Keep only these [`AccountKind`](crate::domain::account::AccountKind) discriminants:
    /// `0` asset, `1` liability, `2` equity, `3` income, `4` expense.
    ///
    /// `None` — and an empty `Vec` — mean every kind. Unknown values match
    /// nothing; the `kind` column is constrained to `0..=4`.
    pub kinds: Option<Vec<u8>>,
    /// Keep only these accounts.
    ///
    /// `None` — and an empty `Vec` — mean every account.
    pub ids: Option<Vec<Uuid>>,
    /// Keep only accounts in these groups.
    ///
    /// `None` — and an empty `Vec` — mean every group. Ungrouped accounts have
    /// no id to match, so naming any group leaves them out.
    pub group_ids: Option<Vec<Uuid>>,
    /// Count only the postings of entries dated inside this range.
    ///
    /// `None` counts every posting, whenever it was made. This narrows what
    /// [`balance`](ResultItem::balance) and
    /// [`posting_counts`](ResultItem::posting_counts) add up to; which
    /// accounts come back is the other filters' business.
    pub dates: Option<Dates>,
}

/// A stretch of days, as the journal stores them: ISO-8601 (`YYYY-MM-DD`).
///
/// Half-open, which is what lets a caller name a month or a year without
/// knowing how many days are in it.
#[derive(Debug, Clone)]
pub struct Dates {
    /// The first day the range takes in.
    pub from: String,
    /// The first day past the end of it.
    pub before: String,
}

/// The column [`view`] sorts on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum By {
    /// Signed ledger balance.
    #[default]
    Balance,
    /// Account name.
    Name,
    /// How many postings the account has — how busy it is.
    Postings,
}

/// The postings of the entries dated inside `dates`, shaped like the posting
/// table so that [`view`] can join it in the same table's place.
fn postings_in(dates: Dates) -> SelectStatement {
    let date = Expr::col((JournalEntry::Table, JournalEntry::Date));

    Query::select()
        .columns([
            (Posting::Table, Posting::AccountId),
            (Posting::Table, Posting::Amount),
        ])
        .from(Posting::Table)
        // Inner: a posting whose entry falls outside the range has no business
        // here, and one without an entry at all cannot be dated.
        .inner_join(
            JournalEntry::Table,
            Expr::col((Posting::Table, Posting::JournalEntryId))
                .equals((JournalEntry::Table, JournalEntry::Id)),
        )
        .and_where(date.clone().gte(dates.from))
        .and_where(date.lt(dates.before))
        .take()
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
            group_id: row.get(Account::AccountGroupId.unquoted())?,
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
