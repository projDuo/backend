pub mod routes;
pub mod errors;
pub mod payloads;
pub mod middleware;

use poem::{Route, post};

pub fn routes() -> Route {
    Route::new()
        .at("/auth/login", post(routes::login))
        .at("/auth/logout", post(routes::logout))
        .at("/auth/logout_all", post(routes::logout_all))
}