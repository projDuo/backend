use crate::service::crud::*;
use super::Repository;
use super::RepositoryError;
use crate::domain::Identifiable;

use sea_orm::DbErr;
use sea_orm::{
    ActiveModelTrait, IntoActiveModel, EntityTrait, 
    PaginatorTrait, PrimaryKeyTrait, TryIntoModel
};

impl<D, E, C> Create<D, C> for Repository<D, E>
where
    D: From<E::Model> + Send + Sync,
    C: Send + Sync,
    E: EntityTrait,
    E::ActiveModel: From<C> + ActiveModelTrait<Entity = E> + Send,
    E::Model: IntoActiveModel<E::ActiveModel> + Send + Sync,
{
    type Error = RepositoryError;

    async fn create(&self, payload: C) -> Result<D, RepositoryError> {
        let active_model: E::ActiveModel = payload.into();
        let result_model = active_model
            .insert(&self.db)
            .await
            .map_err(RepositoryError::from)?;
            
        Ok(D::from(result_model))
    }
}

impl<D, E> Read<D> for Repository<D, E>
where
    D: Identifiable<Id = <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> 
        + From<E::Model> + Send + Sync,
    E: EntityTrait,
    E::Model: Send + Sync,
{
    type Error = RepositoryError;

    async fn read(&self, id: <D as Identifiable>::Id) -> Result<D, RepositoryError> {
        let result = E::find_by_id(id).one(&self.db).await
            .map_err(RepositoryError::from)?;

        if let Some(v) = result {
            Ok(v.into())
        } else { Err(DbErr::RecordNotFound("E".into()).into()) }
    }

    async fn read_all(&self) -> Result<Vec<D>, RepositoryError> {
        let results = E::find().all(&self.db).await.map_err(RepositoryError::from)?;
        Ok(results.into_iter().map(D::from).collect())
    }

    async fn read_page(&self, page: u64, amount: u64) -> Result<Vec<D>, RepositoryError> {
        let results: Vec<E::Model> = E::find()
            .paginate(&self.db, amount)
            .fetch_page(page)
            .await
            .map_err(RepositoryError::from)?;
            
        Ok(results.into_iter().map(D::from).collect())
    }
}

impl<D, E, U> Update<D, U> for Repository<D, E>
where
    D: Identifiable + From<E::Model> + Send + Sync,
    U: Identifiable<Id = <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send + Sync,
    E: EntityTrait,
    E::ActiveModel: From<U> + ActiveModelTrait<Entity = E> + TryIntoModel<E::Model> + Send,
    E::Model: IntoActiveModel<E::ActiveModel> + Send + Sync,
{
    type Error = RepositoryError;

    async fn update(&self, payload: U) -> Result<D, RepositoryError> {
        let active_model: E::ActiveModel = payload.into();
        
        let result_model = active_model
            .save(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        result_model
            .try_into_model()
            .map(D::from)
            .map_err(RepositoryError::from)
    }
}

impl<D, E> Delete<D> for Repository<D, E>
where
    D: Identifiable<Id = <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send + Sync,
    E: EntityTrait,
{
    type Error = RepositoryError;

    async fn delete(&self, id: <D as Identifiable>::Id) -> Result<(), RepositoryError> {
        E::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(RepositoryError::from)?;

        Ok(())
    }
}