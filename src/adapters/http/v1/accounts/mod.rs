pub mod payloads;
pub mod errors;
pub mod routes;

use super::errors as errors_generic;

use poem::{Route, get, head};

pub fn routes() -> Route {
    Route::new()
        .at(":id", get(routes::get))
        .at(":id", head(routes::exists))
}