pub mod routes;
pub mod errors;

use poem::{Route, get, patch};

pub fn routes() -> Route {
    Route::new()
        .at("/:id", get(routes::read_messages).post(routes::post_message))
        .at("/:id/:message_id", patch(routes::edit_message).delete(routes::delete_message))
}
