pub mod routes;
pub mod errors;
pub mod payloads;
pub mod middleware;

use super::errors as error_code;
use poem::{Route, post, EndpointExt};
use middleware::AuthMiddleware;

pub fn routes(auth_middleware: &AuthMiddleware) -> Route {
    Route::new()
        .at("/login", post(routes::login))
        .at("/refresh", post(routes::refresh))
        .at("/logout", post(routes::logout).with(auth_middleware))
        .at("/logout_all", post(routes::logout_all).with(auth_middleware))
}