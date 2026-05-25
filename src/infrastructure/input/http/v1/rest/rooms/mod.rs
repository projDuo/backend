pub mod payloads;
pub mod routes;
pub mod errors;

use super::errors as error_code;
use super::auth::middleware::AuthenticatedUser;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(routes::get_rooms_list).post(routes::create))
        .at("/:id", get(routes::get).patch(routes::update))
        .at("/:id/join", post(routes::join))
        .at("/:id/ready", post(routes::ready))
        .at("/:id/leave", post(routes::leave))
}