mod database;
mod gateway;
mod auth;
mod rooms;
mod game;

use poem::{
    get, handler, head, middleware::{ AddData, Cors }, post, EndpointExt, Route
};
use shuttle_poem::ShuttlePoem;
use shuttle_runtime::SecretStore;
use std::sync::{ Arc };
use std::collections::HashMap;
use tokio::sync::{ broadcast, RwLock };


#[handler]
fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn poem(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttlePoem<impl poem::Endpoint> {
    let db = match (
        secret_store.get("DB_HOST"),
        secret_store.get("DB_NAME"),
        secret_store.get("DB_PORT"),
        secret_store.get("DB_USER"),
        secret_store.get("DB_PASS"),
    ) {
        (Some(host), Some(name), Some(port), Some(user), Some(pass)) => {
            let uri = format!(
                "postgres://{}:{}@{}:{}/{}",
                user, pass, host, port, name
            );
            match sea_orm::Database::connect(uri).await {
                Ok(connection) => Ok(connection),
                Err(e) => Err(shuttle_runtime::Error::Database(shuttle_runtime::CustomError::new(e).to_string())),
            }
        },
        _ => Err(shuttle_runtime::Error::Database("Not all database parameters have been provided. The execution is aborted!".to_string())),
    };

    match db {
        Ok(db) => {
            let app = Route::new()
            .at("/api/hello_world", get(hello_world))
            .at("/api/gateway", get(gateway::gateway))
            .at("/api/auth/register", head(auth::exists).post(auth::register))
            .at("/api/auth/login", post(auth::login))
            .with(Cors::new())
            .with(AddData::new(Arc::new(db)))
            .with(AddData::new(Arc::new(RwLock::new(HashMap::<String, game::room::Room>::new()))));
            Ok(app.into())
        }
        Err(e) => {
            Err(e)
        }
    }
}
