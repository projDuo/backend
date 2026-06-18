pub mod routes;
pub mod errors;
pub mod payloads;

use super::errors as error_code;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/history", get(routes::history))
        .at("/:id", get(routes::get))
        .at("/:id/play", post(routes::play))
        .at("/:id/play/:card_id", post(routes::play))
}