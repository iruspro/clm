use std::rc::Rc;

use domain::account_group::AccountGroupRepository;
use rusqlite::Connection;

pub struct SQLiteAccountGroupRepository {
    #[expect(
        dead_code,
        reason = "read once the AccountGroupRepository methods stop being todo!()"
    )]
    db: Rc<Connection>,
}

impl SQLiteAccountGroupRepository {
    pub fn new(db: Rc<Connection>) -> Self {
        Self { db }
    }
}

impl AccountGroupRepository for SQLiteAccountGroupRepository {
    fn add(&self, _group: &domain::account_group::AccountGroup) -> Result<(), domain::RepoError> {
        todo!()
    }

    fn update(
        &self,
        _group: &domain::account_group::AccountGroup,
    ) -> Result<(), domain::RepoError> {
        todo!()
    }

    fn delete(&self, _group_id: domain::AccountGroupId) -> Result<(), domain::RepoError> {
        todo!()
    }
}
