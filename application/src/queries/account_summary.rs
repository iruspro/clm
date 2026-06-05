//! Query: list every account's current standing — its stored details plus its
//! computed balance.

use domain::account::{AccountKind, AccountRepository};
use domain::money::{Currency, Money};
use domain::{AccountGroupId, AccountId};

use crate::error::AppError;

/// A read model describing one account's current state.
///
/// This is the query's output, kept deliberately separate from the
/// [`Account`](domain::account::Account) entity so the UI depends on this flat
/// view rather than on the domain aggregate. It mirrors the account's stored
/// fields and enriches them with the `balance` computed by the repository.
pub struct AccountSummary {
    pub id: AccountId,
    pub kind: AccountKind,
    pub currency: Currency,
    pub name: String,
    pub description: String,
    pub group_id: Option<AccountGroupId>,
    /// The account's current balance, in its own [`currency`](Self::currency).
    pub balance: Money,
}

/// Lists every account paired with its balance, as [`AccountSummary`] rows.
pub struct GetAccountsSummaryQuery<AR> {
    accounts: AR,
}

impl<AR> GetAccountsSummaryQuery<AR>
where
    AR: AccountRepository,
{
    pub fn new(accounts: AR) -> Self {
        GetAccountsSummaryQuery { accounts }
    }

    /// Returns one [`AccountSummary`] per account. Empty when there are no
    /// accounts; the balances come straight from the repository.
    pub fn execute(&self) -> Result<Vec<AccountSummary>, AppError> {
        let accounts = self.accounts.list_with_balances()?;

        Ok(accounts
            .into_iter()
            .map(|awb| {
                // `into_parts` moves `name`/`description` out instead of cloning.
                let balance = awb.balance;
                let (id, kind, currency, name, description, group_id) = awb.account.into_parts();
                AccountSummary {
                    id,
                    kind,
                    currency,
                    name: name.into_string(),
                    description,
                    group_id,
                    balance,
                }
            })
            .collect())
    }
}
