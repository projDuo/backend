mod database;
mod gateway;
mod http;
mod game;
mod runtime_storage;

use poem::{
    get, handler, head, middleware::{ AddData, Cors }, patch, post, EndpointExt, Route
};
use shuttle_poem::ShuttlePoem;
use shuttle_runtime::SecretStore;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use http::*;

pub type Players = HashSet::<gateway::sessions::User>;
pub type Rooms = runtime_storage::DataTable::<game::rooms::Room>;

#[handler]
fn hello_world() -> &'static str {
    "Hello, world!"
}

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
            let app = Route::new() //Тоді створити новий екземпляр Route
            .at("/api/hello_world", get(hello_world)) //Задавання шляхів, методів та відповідних функцій
            .at("/api/gateway", get(gateway::gateway))
            .at("/api/auth/register", head(auth::exists).post(auth::register))
            .at("/api/auth/login", post(auth::login))
            .at("/api/auth/logout", post(auth::logout))
            .at("/api/auth/logout_all", post(auth::logout_all))
            .at("/api/users/:id", get(users::get))
            .at("/api/users/:id/stat", get(users::get_full))
            .at("/api/rooms", get(http::rooms::get_rooms_list).post(http::rooms::create))
            .at("/api/rooms/:id", patch(http::rooms::update))
            .at("/api/rooms/:id/join", post(http::rooms::join))
            .at("/api/rooms/:id/ready", post(http::rooms::ready))
            .at("/api/rooms/:id/leave", post(http::rooms::leave))
            .at("/api/rooms/:id/game", get(http::rooms::game::get).post(http::rooms::game::start))
            .at("/api/rooms/:id/game/play", post(http::rooms::game::play))
            .at("/api/rooms/:id/game/play/:card_id", post(http::rooms::game::play))
            .with(Cors::new().allow_origin_regex("*")) //Налаштування CORS політики
            .with(AddData::new(Arc::new(db))) //Передача посилання на з'єднання БД в аргументи функцій
            .with(AddData::new(Arc::new(RwLock::new(Players::new())))) //Передача посилання на список авторизованих по gateway гравців
            .with(AddData::new(Arc::new(RwLock::new(Rooms::new())))); //Передача посилання на список кімнат
            Ok(app.into()) //Завершення налаштування та передача Route в Shuttle Runtime.
        }
        Err(e) => {
            Err(e) //Інакше передати помилку.
        }
    }
}
