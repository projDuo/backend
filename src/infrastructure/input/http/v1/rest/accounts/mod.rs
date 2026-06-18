pub mod payloads;
pub mod errors;
pub mod routes;

use super::errors as error_code;
use super::auth::middleware::AuthMiddleware;
use poem::{Route, get, post, EndpointExt};

pub fn routes(auth_middleware: &AuthMiddleware) -> Route {
    Route::new()
        // Prevent `/accounts/:id` from capturing `register`.
        .at("/register", post(routes::register))
        .at(":id", get(routes::get)
            .head(routes::exists)
            .patch(routes::update).with(auth_middleware)
        )
}