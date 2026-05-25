use chrono::Utc;
use tokio::sync::broadcast;
use uuid::Uuid;
use dashmap::{DashMap, Entry};
use crate::domain::activity::events::ActivityInternalEvent;
use crate::domain::game::GameInternalEventBus;
use crate::domain::room::RoomInternalEventBus;
use crate::domain::{ EventBus, activity::* };
use async_trait::async_trait;
use tokio::time;
use std::time::Duration;
use std::sync::Arc;

/// Inactivity threshold for the stale-activity sweeper (seconds).
pub const ACTIVITY_HEARTBEAT_SECS: usize = 60;

#[derive(Clone)]
pub struct Service {
    activities: Arc<DashMap<Uuid, Activity>>,
    event_bus: Arc<dyn ActivityEventBus + Send + Sync>,
    rooms_internal_event_bus: Arc<dyn RoomInternalEventBus + Send + Sync>,
    games_internal_event_bus: Arc<dyn GameInternalEventBus + Send + Sync>,
    _shutdown_tx: broadcast::Sender<()>,
}

impl Service {
    pub fn new(
        heartbeat: usize,
        rooms_internal_event_bus: Arc<dyn RoomInternalEventBus + Send + Sync>,
        games_internal_event_bus: Arc<dyn GameInternalEventBus + Send + Sync>,
        event_bus: Arc<dyn ActivityEventBus + Send + Sync>,
        internal_event_bus: Arc<dyn EventBus<ActivityInternalEvent> + Sync + Send + 'static>,
    ) -> Arc<Self> {
        let activities: Arc<DashMap<Uuid, Activity>> = Arc::new(DashMap::new());
        let (_shutdown_tx, shutdown_rx) = broadcast::channel(1);

        let sweeper_map = activities.clone();

        let service = Arc::new(Self { activities, event_bus, rooms_internal_event_bus, games_internal_event_bus, _shutdown_tx });
        let weak_self = Arc::downgrade(&service);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                let Some(this) = weak_self.upgrade() else {
                    break;
                };

                let now = Utc::now();
                let mut expired_keys = Vec::new();

                for entry in sweeper_map.iter() {
                    let duration_since_update = now.signed_duration_since(entry.value().updated_at);
                    
                    if duration_since_update.num_seconds() >= heartbeat as i64 {
                        expired_keys.push(entry.key().clone());
                    }
                }

                for key in expired_keys {
                    let this_clone = this.clone();
                    tokio::spawn(async move { this_clone.delete_activity(key).await });
                }

            }
        });

        super::spawn_service_listener(&service, internal_event_bus.subscribe(), shutdown_rx);

        service
    }

    pub async fn on_value_expired(
        rooms: Arc<dyn RoomInternalEventBus + Send + Sync>,
        games: Arc<dyn GameInternalEventBus + Send + Sync>,
        k: Uuid,
        v: Activity
    ) {
        if let Some(_) = v.room {
            let player_id = k.clone();
            tokio::spawn(async move { 
                rooms.remove_room_player(player_id).await;
            });
        }
        if let Some(v) = v.game {
            tokio::spawn(async move { 
                games.remove_player(v, k).await;
            });
        }
    }
}

#[async_trait]
impl super::EventHandler<ActivityInternalEvent> for Service {
    async fn process(&self, event: ActivityInternalEvent) {
        let res = match event {
            ActivityInternalEvent::UserJoinedRoom { room_id, user_id } => {
                let activity_mark = MarkActivityCommand::new(user_id)
                    .room(Some(room_id))
                    .to_owned();
                self.mark_activity(activity_mark).await
            }
            ActivityInternalEvent::UserLeftRoom { user_id } => {
                let activity_mark = MarkActivityCommand::new(user_id)
                    .room(None)
                    .to_owned();
                self.mark_activity(activity_mark).await
            }
            ActivityInternalEvent::UserJoinedGame { game_id, user_id } => {
                let activity_mark = MarkActivityCommand::new(user_id)
                    .game(Some(game_id))
                    .to_owned();
                self.mark_activity(activity_mark).await                            
            }
            ActivityInternalEvent::UserLeftGame { user_id } => {
                let activity_mark = MarkActivityCommand::new(user_id)
                    .game(None)
                    .to_owned();
                self.mark_activity(activity_mark).await
            }
        };

        /*if let Err(e) = res.clone() {
            tracing::error!("ActivityInternalEvent error: {}", e);
        }*/

        tracing::info!("dsfsdfsd: {:?}", res)
    }
}

#[async_trait]
impl ActivityService for Service {
    async fn mark_activity(&self, cmd: MarkActivityCommand) -> Result<Activity, ActivityError> {
        let id = cmd.id;
        match self.activities.entry(cmd.id) {
            Entry::Occupied(mut v) => {
                let activity = v.get_mut();
                if let Some(room) = cmd.room {
                    let room_changed = room != activity.room;
                    if activity.room.is_some() && room_changed {
                        self.event_bus.left_room(cmd.id).await
                    }
                    activity.room = room;
                    if room_changed {
                        if let Some(room) = &activity.room {
                            self.event_bus.joined_room(room.to_string(), cmd.id).await
                        }
                    }
                }
                if let Some(game) = cmd.game {
                    let game_changed = game != activity.game;
                    if activity.game.is_some() && game_changed { self.event_bus.left_game(cmd.id).await }
                    activity.game = game;
                    if game_changed {
                        if let Some(game) = &activity.game { self.event_bus.joined_game(*game, cmd.id).await }
                    }
                }
                activity.updated_at = Utc::now().into();

                tracing::info!("acti: {:?}", activity)
            },
            Entry::Vacant(v) => {
                let id = cmd.id;
                let activity = v.insert(cmd.into());
                self.event_bus.new_channel(id);
                if let Some(room) = &activity.room { self.event_bus.joined_room(room.to_string(), id).await }
                if let Some(game) = &activity.game { self.event_bus.joined_game(*game, id).await }
            }
        }

        let activity = self.activities.get(&id)
            .expect("Activity is supposed to exist here")
            .clone();
        Ok(activity)
    }

    async fn read_activity(&self, user_id: Uuid) -> Result<Activity, ActivityError> {
        let res = self.activities.get(&user_id)
            .ok_or(ActivityError::NotFound)?
            .clone();
        
        Ok(res)
    }
    async fn delete_activity(&self, user_id: Uuid) -> Result<(), ActivityError> {
        let Some((id, activity)) = self.activities.remove(&user_id) else { return Err(ActivityError::NotFound) };

        Self::on_value_expired(
            self.rooms_internal_event_bus.clone(),
            self.games_internal_event_bus.clone(),
            id,
            activity
        ).await;
        self.event_bus.close_channel(user_id);
        Ok(())
    }
}