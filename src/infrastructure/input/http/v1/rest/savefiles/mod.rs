pub mod payloads;
pub mod errors;
pub mod routes;

use super::errors as error_code;

use poem::{Route, get};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(routes::get_the_best))
        .at(":id", get(routes::get))
}