pub mod v1;

use poem::{
    endpoint::StaticFilesEndpoint,
    http::StatusCode,
    Endpoint, Request, Response, Result, Route,
};
use crate::domain::auth::AuthService;
use crate::domain::sessions::SessionsRepository;
use std::sync::Arc;
use std::path::PathBuf;

pub struct SpaEndpoint {
    inner: StaticFilesEndpoint,
    index_path: PathBuf,
}

impl SpaEndpoint {
    pub fn new(dir: impl Into<PathBuf>, index_path: impl Into<PathBuf>) -> Self {
        Self {
            inner: StaticFilesEndpoint::new(dir).index_file("index.html"),
            index_path: index_path.into(),
        }
    }
}

impl Endpoint for SpaEndpoint {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let res = self.inner.call(req).await;
        match res {
            Ok(resp) => {
                if resp.status() == StatusCode::NOT_FOUND {
                    let html = tokio::fs::read_to_string(&self.index_path).await
                        .map_err(poem::error::InternalServerError)?;
                    Ok(Response::builder()
                        .header("content-type", "text/html")
                        .body(html))
                } else {
                    Ok(resp)
                }
            }
            Err(err) => {
                let resp = err.into_response();
                if resp.status() == StatusCode::NOT_FOUND {
                    let html = tokio::fs::read_to_string(&self.index_path).await
                        .map_err(poem::error::InternalServerError)?;
                    Ok(Response::builder()
                        .header("content-type", "text/html")
                        .body(html))
                } else {
                    Ok(resp)
                }
            }
        }
    }
}

pub fn api_routes(
    auth_service: Arc<dyn AuthService + Send + Sync>,
    sessions_repo: Arc<dyn SessionsRepository + Send + Sync>
) -> Route {
    Route::new()
        .nest("/api/v1", v1::routes(auth_service, sessions_repo))
        .nest("/", SpaEndpoint::new("client/dist", "client/dist/index.html"))
}