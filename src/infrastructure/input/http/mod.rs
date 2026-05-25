pub mod v1;

use poem::Route;
use crate::domain::auth::AuthService;
use crate::domain::sessions::SessionsRepository;
use std::sync::Arc;

pub fn api_routes(
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>
) -> Route {
    Route::new()
        .nest("/api/v1", v1::routes(auth_service, sessions_repo))
}