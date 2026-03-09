pub mod crud;

pub mod accounts;

use serde::Serialize;

use crate::domain::Identifiable;
use crud::*;
use std::marker::PhantomData;


pub use accounts::Accounts;


#[derive(Debug, Serialize)]
pub enum ServiceError {
    Internal,
    NotFound,
    Unauthorized,
    Exists,
    NotModified,
    BadRequest(String),
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
    D: Identifiable + Send + Sync,
    C: Send,
    R: Create<D, C>,
    ServiceError: From<<R as Create<D, C>>::Error>,
{
    type Error = ServiceError;

    async fn create(&self, payload: C) -> Result<D, Self::Error> {
        self.repo.create(payload).await.map_err(|v| v.into())
    }
}

impl<D, R> Read<D> for Service<D, R> 
where 
    D: Identifiable + Send + Sync,
    R: Read<D>,
    ServiceError: From<<R as Read<D>>::Error>,
{
    type Error = ServiceError;

    async fn read(&self, id: <D as Identifiable>::Id) -> Result<D, Self::Error> {
        self.repo.read(id).await.map_err(|v| v.into())
    }

    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, Self::Error> {
        self.repo.read_page(page, amount).await.map_err(|v| v.into())
    }

    async fn read_all(&self) -> Result<Vec<D>, Self::Error> {
        self.repo.read_all().await.map_err(|v| v.into())
    }
}


impl<D, R, U> Update<D, U> for Service<D, R> 
where 
    D: Identifiable + Send + Sync,
    U: Identifiable + Send,
    R: Update<D, U>,
    ServiceError: From<<R as Update<D, U>>::Error>,
{
    type Error = ServiceError;

    async fn update(&self, payload: U) -> Result<D, Self::Error> {
        self.repo.update(payload).await.map_err(|v| v.into())
    }
}

impl<D, R> Delete<D> for Service<D, R> 
where 
    D: Identifiable + Send + Sync,
    R: Delete<D>,
    ServiceError: From<<R as Delete<D>>::Error>,
{
    type Error = ServiceError;

    async fn delete(&self, id: <D as Identifiable>::Id) -> Result<(), Self::Error> {
        self.repo.delete(id).await.map_err(|v| v.into())
    }
}