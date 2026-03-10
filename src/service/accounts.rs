
use crate::domain::{accounts as domain};
use uuid::Uuid;
use super::Error;

pub type Accounts<R> = super::Service<domain::Account, R>;

/*#[derive(Debug, Error)]
pub enum Error {
    #[error("Account with the requested ID or Login doesn't exist.")]
    NotFound,
    #[error("An account with this login already exists.")]
    Exists,
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    #[error("Incorrect display name provided.")]
    InvalidDisplayName,
    #[error("A password must be at least 6 characters long")]
    WeakPassword,
    #[error("An internal system error occurred. Please try again later.")]
    Internal,
}*/


impl<R> Accounts<R>
where
    R: super::Read<domain::Account> + Service,
    Error<domain::Error>: From<<R as super::Read<domain::Account>>::Error>,
    Error<domain::Error>: From<<R as Service>::Error>
{
        pub async fn read_by_id_or_login(&self, identifier: &str) 
            -> Result<domain::Account, Error<domain::Error>> {
            let account = if let Ok(uuid) = Uuid::parse_str(identifier) {
                self.repo.read(uuid).await?
            } else {
                self.repo.read_by_login(&identifier).await?
            };

            Ok(account)
        }

        pub async fn login_taken(&self, login: &str) -> Result<bool, Error<domain::Error>> {
            let account = self.repo.read_by_login(login).await
                .map_err(Into::into);

            match account {
                Ok(_) => Ok(true),
                Err(Error::NotFound) => Ok(false),
                Err(e) => Err(e),
            }
        }
}

pub trait Service {
    type Error;

    async fn read_by_login(&self, login: &str) -> Result<domain::Account, Self::Error>;
}

impl<R> Service for Accounts<R>
where
    R: Service,
    Error<domain::Error>: From<<R as Service>::Error>,
{
    type Error = Error<domain::Error>;

    async fn read_by_login(&self, login: &str) -> Result<domain::Account, Self::Error> {
        self.repo.read_by_login(login).await.map_err(Self::Error::from)
    }
}