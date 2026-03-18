mod infrastructure;
mod adapters;
mod domain;
mod application;
mod runtime_storage;

use poem::{
    middleware::{ AddData, Cors }, EndpointExt
};
use sea_orm::DatabaseConnection;
use shuttle_poem::ShuttlePoem;
use shuttle_runtime::SecretStore;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;


pub type Players = HashSet::<gateway::sessions::User>;
pub type Rooms = runtime_storage::DataTable::<game::rooms::Room>;

use infrastructure::postgres::Postgres;

type AccountsService = application::Accounts<Postgres>;
type SessionsService = application::Sessions<Postgres>;
type TokenProvider = application::Jwt;
type AuthService = application::Auth<
        AccountsService,
        Postgres,
        TokenProvider>;

struct AppState {
    accounts: AccountsService,
    sessions: SessionsService,
    auth: AuthService,
}

impl AppState {
    pub fn new(db: Postgres, secret_store: SecretStore) -> Self {
        let accounts = AccountsService::new(db.clone());
        let sessions = SessionsService::new(db.clone());
        let jwt = TokenProvider::new(secret, refresh_expires_after, access_expires_after);
        let auth = AuthService::new(accounts, db.clone(), jwt);
        Self {
            accounts,
            sessions,
            auth
        }
    }
}

#[shuttle_runtime::main]
async fn poem(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttlePoem<impl poem::Endpoint> {
    let db = if let (Some(host), Some(name), Some(Ok(port)), Some(user), Some(pass)) = (
        secret_store.get("DB_HOST"),
        secret_store.get("DB_NAME"),
        secret_store.get("DB_PORT").map(|v| v.parse()),
        secret_store.get("DB_USER"),
        secret_store.get("DB_PASS"),
    ) {
        let pg_details = infrastructure::postgres::PostgresDetails::new(
            host.as_str(),
            name.as_str(),
            port,
            user.as_str(),
            pass.as_str()
        );

        infrastructure::postgres::Postgres::from_details(pg_details).await
            .map_err(|e| shuttle_runtime::Error::Database(shuttle_runtime::CustomError::new(e).to_string()))
    } else {
        Err(shuttle_runtime::Error::Database("Not all database parameters were provided. The execution has been aborted!".to_string())) //В іншому випадку повернути у змінну db помилку
    };
    
    match db {
        Ok(db) => { //Якщо змінна db містить з'єднання
            let state = AppState::new(db, secret_store);
            
            let app = api_routes()
            .with(Cors::new().allow_origin_regex("*")) //Налаштування CORS політики
            .with(AddData::new(Arc::new(db.clone()))) //Передача посилання на з'єднання БД в аргументи функцій
            .with(AddData::new(Arc::new(AppState)))
            .with(AddData::new(Arc::new(RwLock::new(Players::new())))) //Передача посилання на список авторизованих по gateway гравців
            .with(AddData::new(Arc::new(RwLock::new(Rooms::new())))); //Передача посилання на список кімнат
            Ok(app.into()) //Завершення налаштування та передача Route в Shuttle Runtime.
        }
        Err(e) => {
            Err(e) //Інакше передати помилку.
        }
    }
}
