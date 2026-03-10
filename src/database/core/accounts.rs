use crate::{database::core::RepositoryError, domain};

pub trait AccountQueries {
    async fn read_by_login(&self, login: &str) -> Result<domain::accounts::Account, RepositoryError>;
}