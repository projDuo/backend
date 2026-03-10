mod database;
mod gateway;
mod http;
mod domain;
mod service;
mod runtime_storage;

use poem::{
    middleware::{ AddData, Cors }, EndpointExt
};
use shuttle_poem::ShuttlePoem;
use shuttle_runtime::SecretStore;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use http::*;
use domain::game;
use database::postgres as pg_repo;


pub type Players = HashSet::<gateway::sessions::User>;
pub type Rooms = runtime_storage::DataTable::<game::rooms::Room>;

pub type AccountsService = service::Accounts<pg_repo::Accounts>;
pub type SavefilesService = service::Service<domain::savefiles::Savefile, pg_repo::Savefiles>;

#[shuttle_runtime::main]
async fn poem(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttlePoem<impl poem::Endpoint> {
    let db = match ( //Перевірка на наявність усих змінних необхідних для початку роботи
        secret_store.get("DB_HOST"),
        secret_store.get("DB_NAME"),
        secret_store.get("DB_PORT"),
        secret_store.get("DB_USER"),
        secret_store.get("DB_PASS"),
    ) {
        (Some(host), Some(name), Some(port), Some(user), Some(pass)) => { //Якщо усі змінні присутні
            let uri = format!(
                "postgres://{}:{}@{}:{}/{}",
                user, pass, host, port, name
            );
            match sea_orm::Database::connect(uri).await { //Спроба з'єднатися
                Ok(connection) => Ok(connection), //при з'єднані повернути у змінну db з'єднання
                Err(e) => Err(shuttle_runtime::Error::Database(shuttle_runtime::CustomError::new(e).to_string())), //В іншому випадку повернути у змінну db помилку
            }
        },
        _ => Err(shuttle_runtime::Error::Database("Not all database parameters have been provided. The execution is aborted!".to_string())), //В іншому випадку повернути у змінну db помилку
    };

    match db {
        Ok(db) => { //Якщо змінна db містить з'єднання
            let accounts_repo = pg_repo::Accounts::new(db.clone());
            let accounts_service = AccountsService::new(accounts_repo);

            let savefiles_repo = pg_repo::Savefiles::new(db.clone());
            let savefiles_service = SavefilesService::new(savefiles_repo);
            
            let app = api_routes()
            .with(Cors::new().allow_origin_regex("*")) //Налаштування CORS політики
            .with(AddData::new(Arc::new(db.clone()))) //Передача посилання на з'єднання БД в аргументи функцій
            .with(AddData::new(Arc::new(accounts_service)))
            .with(AddData::new(Arc::new(savefiles_service)))
            .with(AddData::new(Arc::new(RwLock::new(Players::new())))) //Передача посилання на список авторизованих по gateway гравців
            .with(AddData::new(Arc::new(RwLock::new(Rooms::new())))); //Передача посилання на список кімнат
            Ok(app.into()) //Завершення налаштування та передача Route в Shuttle Runtime.
        }
        Err(e) => {
            Err(e) //Інакше передати помилку.
        }
    }
}
