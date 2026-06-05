//! Use case: create a new account in the chart of accounts.

use domain::account::{Account, AccountKind, AccountRepository};
use domain::money::Currency;
use domain::{AccountGroupId, AccountId, Name};

use crate::error::AppError;

/// Input for [`CreateAccountUseCase`]; `name` is already validated and
/// `description` may be empty.
pub struct CreateAccountCommand {
    pub kind: AccountKind,
    pub currency: Currency,
    pub name: Name,
    pub description: String,
    pub group_id: Option<AccountGroupId>,
}

pub struct CreateAccountUseCase<AR> {
    accounts: AR,
}

impl<AR> CreateAccountUseCase<AR>
where
    AR: AccountRepository,
{
    pub fn new(accounts: AR) -> Self {
        CreateAccountUseCase { accounts }
    }

    pub fn execute(&self, cmd: CreateAccountCommand) -> Result<AccountId, AppError> {
        let account = Account::new(
            AccountId::new(),
            cmd.kind,
            cmd.currency,
            cmd.name,
            cmd.description,
            cmd.group_id,
        );

        self.accounts.add(&account)?;

        Ok(account.id())
    }
}
