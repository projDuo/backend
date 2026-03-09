pub mod error;
pub use error::RepositoryError;

use sea_orm::{
    DatabaseConnection, EntityTrait
};
use std::marker::PhantomData;

pub struct Repository<D, E>
where
    E: EntityTrait,
{
    pub(crate) db: DatabaseConnection,
    _phantom_d: PhantomData<D>,
    _phantom_e: PhantomData<E>,
}

impl<D, E> Repository<D, E>
where
    E: EntityTrait,
{
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _phantom_d: PhantomData,
            _phantom_e: PhantomData,
        }
    }
}