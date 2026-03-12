pub mod v1;

use poem::Route;

pub fn api_routes() -> Route {
    Route::new()
        .nest("/api/v1", v1::routes())
}