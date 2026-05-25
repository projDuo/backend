pub mod gateway;
pub mod rest;

pub use rest::*;
pub use gateway::*;

use poem::Route;
use crate::domain::auth::AuthService;
use crate::domain::sessions::SessionsRepository;
use std::sync::Arc;

pub fn routes(
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>
) -> Route {
    Route::new()
        .nest("/", rest::routes(auth_service, sessions_repo))
        .nest("/gateway", gateway::routes())
}