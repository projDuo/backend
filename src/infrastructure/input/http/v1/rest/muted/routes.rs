use poem::{handler, web::{Data, Json, Path}, Result};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use super::super::auth::middleware::AuthenticatedUser;

#[derive(Debug, Serialize)]
pub struct MutedEntryResponse {
    pub blocked_id: Uuid,
    pub blocked_at: String,
}

impl From<crate::domain::muted::MutedEntry> for MutedEntryResponse {
    fn from(value: crate::domain::muted::MutedEntry) -> Self {
        Self {
            blocked_id: value.blocked_id,
            blocked_at: value.blocked_at.to_rfc3339(),
        }
    }
}

#[handler]
pub async fn list(
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<Json<Vec<MutedEntryResponse>>> {
    let entries = state
        .muted
        .get_muted_users(user.account_id)
        .await?;

    Ok(Json(entries.into_iter().map(Into::into).collect()))
}

#[handler]
pub async fn block_user(
    Path(blocked_id): Path<Uuid>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<()> {
    state
        .muted
        .block_user(user.account_id, blocked_id)
        .await?;
    Ok(())
}

#[handler]
pub async fn unblock_user(
    Path(blocked_id): Path<Uuid>,
    state: Data<&Arc<AppState>>,
    user: Data<&AuthenticatedUser>,
) -> Result<()> {
    state
        .muted
        .unblock_user(user.account_id, blocked_id)
        .await?;
    Ok(())
}
