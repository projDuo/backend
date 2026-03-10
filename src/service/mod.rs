pub mod crud;
pub mod payloads;
pub mod accounts;


use crate::{database::core::RepositoryError};
use crate::domain::{ErrorType, Identifiable};
use crud::*;
use thiserror::Error;
use std::marker::PhantomData;


pub use accounts::Accounts;

#[derive(Debug, Error)]
pub enum Error<T> {
    #[error("Internal server error. Please try again later.")]
    Internal,
    #[error("The requested resource doesn't exist.")]
    NotFound,
    #[error("The requested resource doesn't exist.")]
    Unauthorized,
    #[error("The resource already exists.")]
    Exists,
    #[error("")]
    NotModified,
    #[error("Domain error: {0}")]
    Domain(T),
}

impl<T> From<RepositoryError> for Error<T> {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound(_) => Self::NotFound,
            RepositoryError::UniqueViolation(_) => Self::Exists, 
            RepositoryError::ForeignKeyViolation(err) => {
                eprintln!("CRITICAL DB BUG (Foreign Key): {:?}", err);
                Self::Internal
            }
            RepositoryError::NotNullViolation(err) => {
                eprintln!("CRITICAL DB BUG (Not Null): {:?}", err);
                Self::Internal
            }
            RepositoryError::ConnectionError(err) => {
                eprintln!("DB CONNECTION DROPPED: {:?}", err);
                Self::Internal
            }
            RepositoryError::Internal(err) => {
                eprintln!("DB INTERNAL ERROR: {:?}", err);
                Self::Internal
            }
        }
    }
}

impl<T> Error<T> {
    pub fn from_domain(value: T) -> Self {
        Self::Domain(value)
    }
}

pub struct Service<D, R>
where
    D: Identifiable + Send + Sync
{
    repo: R,
    _phantom_d: PhantomData<D>,
}

impl<D, R> Service<D, R>
where
    D: Identifiable + Send + Sync
{
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            _phantom_d: PhantomData,
        }
    }
}

impl<D, R, C> Create<D, C> for Service<D, R> 
where 
    D: Identifiable + ErrorType + Send + Sync,
    C: Send,
    R: Create<D, C>,
    Error<<D as ErrorType>::Error>: From<<R as Create<D, C>>::Error>,
{
    type Error = Error<<D as ErrorType>::Error>;

    async fn create(&self, payload: C) -> Result<D, Self::Error> {
        self.repo.create(payload).await.map_err(Self::Error::from)
    }
}

impl<D, R> Read<D> for Service<D, R> 
where 
    D: Identifiable + ErrorType + Send + Sync,
    R: Read<D>,
    Error<<D as ErrorType>::Error>: From<<R as Read<D>>::Error>,
{
    type Error = Error<<D as ErrorType>::Error>;

    async fn read(&self, id: <D as Identifiable>::Id) -> Result<D, Self::Error> {
        self.repo.read(id).await.map_err(Self::Error::from)
    }

    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, Self::Error> {
        self.repo.read_page(page, amount).await.map_err(Self::Error::from)
    }

    async fn read_all(&self) -> Result<Vec<D>, Self::Error> {
        self.repo.read_all().await.map_err(Self::Error::from)
    }
}


impl<D, R, U> Update<D, U> for Service<D, R> 
where 
    D: Identifiable + ErrorType + Send + Sync,
    U: Identifiable + Send,
    R: Update<D, U>,
    Error<<D as ErrorType>::Error>: From<<R as Update<D, U>>::Error>,
{
    type Error = Error<<D as ErrorType>::Error>;

    async fn update(&self, payload: U) -> Result<D, Self::Error> {
        self.repo.update(payload).await.map_err(Self::Error::from)
    }
}

impl<D, R> Delete<D> for Service<D, R> 
where 
    D: Identifiable + ErrorType + Send + Sync,
    R: Delete<D>,
    Error<<D as ErrorType>::Error>: From<<R as Delete<D>>::Error>,
{
    type Error = Error<<D as ErrorType>::Error>;

    async fn delete(&self, id: <D as Identifiable>::Id) -> Result<(), Self::Error> {
        self.repo.delete(id).await.map_err(Self::Error::from)
    }
}