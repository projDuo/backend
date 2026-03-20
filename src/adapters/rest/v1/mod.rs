pub mod auth;
pub mod rooms;
pub mod accounts;
pub mod errors;
pub mod sessions;
pub mod savefiles;

// src/http/v1/mod.rs
use poem::{EndpointExt, Route, get, patch, post};
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
        .nest("/auth", auth::routes())
        .nest("/accounts", accounts::routes())
        .nest("/savefiles", savefiles::routes())
        .at("/", get(rooms::get_rooms_list).post(rooms::create.with(&auth_middleware)))
        
        .at("/:id", patch(rooms::update).with(&auth_middleware))
        .at("/:id/join", post(rooms::join).with(&auth_middleware))
        .at("/:id/ready", post(rooms::ready).with(&auth_middleware))
        .at("/:id/leave", post(rooms::leave).with(&auth_middleware))

        .at("/:id/game", 
            get(rooms::game::get).post(rooms::game::start)
            .with(&auth_middleware)
        )
        .at("/:id/game/play", post(rooms::game::play).with(&auth_middleware))
        .at("/:id/game/play/:card_id", post(rooms::game::play).with(&auth_middleware))
}

#[poem::handler]
async fn hello_world() -> &'static str {
    "Hello from v1!"
}