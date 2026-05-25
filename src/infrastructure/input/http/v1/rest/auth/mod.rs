pub mod routes;
pub mod errors;
pub mod payloads;
pub mod middleware;

use super::errors as error_code;
use poem::{Route, post};

pub fn routes() -> Route {
    Route::new()
        .at("/login", post(routes::login))
        .at("/logout", post(routes::logout))
        .at("/logout_all", post(routes::logout_all))
}