pub mod accounts;
pub mod sessions;
pub mod auth;
pub mod jwt;
pub mod savefiles;
pub mod room;
pub mod game;
pub mod game_history;
pub mod activity;
pub mod chat;
pub mod muted;

pub use accounts::Service as Accounts;
//pub use sessions::Service as Sessions;
pub use activity::Service as Activity;

pub use auth::Service as Auth;
pub use jwt::Service as Jwt;
pub use savefiles::Service as Savefiles;
pub use game_history::Service as GameHistory;
pub use room::Service as Room;
pub use game::Service as Game;
pub use chat::Chat as Chat;
pub use muted::Service as Muted;
use std::sync::{Arc, Weak};
use tokio::sync::broadcast::{ self, error::RecvError};

use async_trait::async_trait;

#[async_trait]
pub trait EventHandler<Event>: Send + Sync + 'static {
    async fn process(&self, event: Event);
}

pub fn spawn_service_listener<T, Event>(
    service: &Arc<T>,
    mut rx: broadcast::Receiver<Event>,
    mut shutdown_rx: broadcast::Receiver<()>,
) 
where
    T: EventHandler<Event>,
    Event: Clone + Send + Sync + 'static, 
{
    let weak_self: Weak<T> = Arc::downgrade(service);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = rx.recv() => {
                    tracing::info!("bruh");
                    match res {
                        Ok(event) => {
                            if let Some(this) = weak_self.upgrade() {
                                tokio::spawn(async move {
                                    this.process(event).await;
                                });
                            } else {
                                break; 
                            }
                        }

                        Err(RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(RecvError::Closed) => {
                            tracing::debug!("Event bus closed, exiting service listener.");
                            break;
                        }
                    }

                    
                }
                
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
    });
}