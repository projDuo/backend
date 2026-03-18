pub mod auth;
pub mod rooms;
pub mod accounts;
pub mod errors;
pub mod sessions;

// src/http/v1/mod.rs
use poem::{get, head, patch, post, Route};
// pub mod payloads; // If you moved them here!


pub fn routes() -> Route {
    Route::new()
        // Notice: No "/api/" prefix here anymore!
        .at("/hello_world", get(hello_world)) 
        
        // Auth
        .at("/auth/register", head(auth::exists).post(auth::register))
        .at("/auth/login", post(auth::login))
        .at("/auth/logout", post(auth::logout))
        .at("/auth/logout_all", post(auth::logout_all))
        
        // Users
        .nest("/accounts", accounts::routes())
        
        // Rooms & Game (Removed the `http::` prefix from the function calls)
        .at("/rooms", get(rooms::get_rooms_list).post(rooms::create))
        .at("/rooms/:id", patch(rooms::update))
        .at("/rooms/:id/join", post(rooms::join))
        .at("/rooms/:id/ready", post(rooms::ready))
        .at("/rooms/:id/leave", post(rooms::leave))
        .at("/rooms/:id/game", get(rooms::game::get).post(rooms::game::start))
        .at("/rooms/:id/game/play", post(rooms::game::play))
        .at("/rooms/:id/game/play/:card_id", post(rooms::game::play))
}

#[poem::handler]
async fn hello_world() -> &'static str {
    "Hello from v1!"
}