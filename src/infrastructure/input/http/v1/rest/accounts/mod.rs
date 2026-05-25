pub mod payloads;
pub mod errors;
pub mod routes;

use super::errors as error_code;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        // Prevent `/accounts/:id` from capturing `register`.
        .at("/register", post(routes::register))
        .at(":id", get(routes::get).head(routes::exists))
}