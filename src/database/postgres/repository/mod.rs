pub mod crud;
pub mod error;

pub mod accounts;

use super::entities;
use crate::domain;

pub type Accounts = Repository<domain::accounts::Account, entities::prelude::Accounts>;

use super::generic_repository::{ 
    Repository,
    RepositoryError,
    //crud as generic_crud,
};
