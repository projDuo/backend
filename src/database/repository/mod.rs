pub mod error;

use crate::service::ServiceError;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ColumnTypeTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, IntoActiveModel, PaginatorTrait, PrimaryKeyTrait, TryIntoModel,
};
use serde::Serialize;
use std::marker::PhantomData;

impl From<DbErr> for ServiceError {
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::RecordNotFound(_) => Self::NotFound,
            DbErr::RecordNotInserted => Self::Exists,
            DbErr::RecordNotUpdated => Self::NotModified,
            _ => Self::Internal,
        }
    }
}

pub struct Repository<D, E>
where
    E: EntityTrait,
{
    db: DatabaseConnection,
    _phantom_d: PhantomData<D>,
    _phantom_e: PhantomData<E>,
}

impl<D, E> Repository<D, E>
where
    D: From<E::Model> + Send + Sync,
    E: EntityTrait,
    E::ActiveModel: From<D> + ActiveModelTrait<Entity = E> + TryIntoModel<E::Model> + Send,
    E::Model: IntoActiveModel<E::ActiveModel> + Send + Sync,
{
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _phantom_d: PhantomData,
            _phantom_e: PhantomData,
        }
    }
}

pub trait CRUD<D>
where
    D: Send + Sync,
{
    type PKValue;

    async fn create(&self, model: D) -> Result<D, ServiceError>;
    async fn read(&self, id: Self::PKValue) -> Result<Option<D>, ServiceError>;
    async fn read_all(&self) -> Result<Vec<D>, ServiceError>;
    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, ServiceError>;
    async fn update(&self, model: D) -> Result<D, ServiceError>;
    async fn delete(&self, id: Self::PKValue) -> Result<DeleteResult, ServiceError>;
}

impl<D, E> CRUD<D> for Repository<D, E>
where
    D: From<E::Model> + Send + Sync,
    E: EntityTrait,
    E::ActiveModel: From<D> + ActiveModelTrait<Entity = E> + TryIntoModel<E::Model> + Send,
    E::Model: IntoActiveModel<E::ActiveModel> + Send + Sync,
{
    type PKValue = <E::PrimaryKey as PrimaryKeyTrait>::ValueType;

    async fn create(&self, model: D) -> Result<D, ServiceError> {
        let active_model: E::ActiveModel = model.into();
        let result_model = active_model
            .insert(&self.db)
            .await
            .map_err(ServiceError::from)?;
        Ok(D::from(result_model))
    }

    async fn read(&self, id: Self::PKValue) -> Result<Option<D>, ServiceError> {
        let result = E::find_by_id(id).one(&self.db).await?;
        Ok(result.map(D::from))
    }

    async fn read_all(&self) -> Result<Vec<D>, ServiceError> {
        let results = E::find().all(&self.db).await?;
        Ok(results.into_iter().map(D::from).collect())
    }

    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, ServiceError> {
        let results = E::find()
            .paginate(&self.db, amount)
            .fetch_page(page)
            .await?;
        Ok(results.into_iter().map(D::from).collect())
    }

    async fn update(&self, model: D) -> Result<D, ServiceError> {
        let active_model: E::ActiveModel = model.into();
        let result_model = active_model.save(&self.db).await?;

        result_model
            .try_into_model()
            .map(D::from)
            .map_err(ServiceError::from)
    }

    async fn delete(&self, id: Self::PKValue) -> Result<DeleteResult, ServiceError> {
        E::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(ServiceError::from)
    }
}
