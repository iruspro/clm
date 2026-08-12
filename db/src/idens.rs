use sea_query::Iden;

#[derive(Iden)]
pub enum AccountGroup {
    Table,
    Id,
    Name,
    Description,
}

#[derive(Iden)]
pub enum Account {
    Table,
    Id,
    Name,
    Description,
    Kind,
    Currency,
    AccountGroupId,
}

#[derive(Iden)]
pub enum JournalEntry {
    Table,
    Id,
    Date,
    Description,
}

#[derive(Iden)]
pub enum Posting {
    Table,
    AccountId,
    Amount,
}
