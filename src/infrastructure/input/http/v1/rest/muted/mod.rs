pub mod routes;
pub mod errors;

use poem::{Route, get, post};

pub fn routes() -> Route {
    Route::new()
        .at("/", get(routes::list))
        .at("/:id", post(routes::block_user).delete(routes::unblock_user))
}
