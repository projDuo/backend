pub mod auth;
pub mod rooms;
pub mod accounts;
pub mod errors;
pub mod sessions;
pub mod savefiles;
pub mod chat;
pub mod muted;
pub mod games;
pub mod activities;

// src/http/v1/mod.rs
use poem::{EndpointExt, Route, get};
use std::sync::Arc;
use crate::domain::auth::AuthService;
use crate::domain::sessions::SessionsRepository;
// pub mod payloads; // If you moved them here!

use auth::middleware::AuthMiddleware;

pub fn routes(
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>
) -> Route {
    let auth_middleware = AuthMiddleware::new(auth_service, sessions_repo);
    Route::new()
        .at("/hello_world", get(hello_world))
        .nest("/auth", auth::routes(&auth_middleware))
        .nest("/accounts", accounts::routes(&auth_middleware))
        .nest("/savefiles", savefiles::routes())
        .nest("/rooms", rooms::routes().with(&auth_middleware))
        .nest("/games", games::routes().with(&auth_middleware))
        .nest("/chat", chat::routes().with(&auth_middleware))
        .nest("/muted", muted::routes().with(&auth_middleware))
}

#[poem::handler]
async fn hello_world() -> &'static str {
    "Hello from v1!"
}
