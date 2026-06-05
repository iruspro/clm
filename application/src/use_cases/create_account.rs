//! Use case: create a new account in the chart of accounts.

use domain::{
    AccountId, Name,
    account::{Account, AccountKind, AccountRepository},
    money::Currency,
};

use crate::error::AppResult;

/// Input for [`CreateAccountUseCase`]; `name` is already validated and
/// `description` may be empty.
pub struct CreateAccountCommand {
    pub kind: AccountKind,
    pub currency: Currency,
    pub name: Name,
    pub description: String,
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

    pub fn execute(&self, cmd: CreateAccountCommand) -> AppResult<AccountId> {
        let account = Account::new(
            AccountId::generate(),
            cmd.kind,
            cmd.currency,
            cmd.name,
            cmd.description,
        );

        self.accounts.add(&account)?;

        Ok(account.id())
    }
}
