use std::{sync::Arc, time::Duration, vec};

use std::collections::hash_map::HashMap;
use indexmap::IndexSet;
use uuid::Uuid;
use dashmap::DashMap;
use crate::domain::activity::ActivityInternalEventBus;
use crate::domain::game_history::{GameHistory, GameHistoryService};
use crate::domain::savefiles::{SavefilesService, UpdateSavefileRequest};
use crate::domain::{EventBus, game::*};
use async_trait::async_trait;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Service {
    games: DashMap<Uuid, Game>,
    players: DashMap<Uuid, Player>,
    ingame: DashMap<Uuid, IndexSet<Uuid>>,
    leaderboard: DashMap<Uuid, IndexSet<PlayerResult>>,
    internal_event_bus: Arc<dyn EventBus<GameInternalEvents> + Send + Sync>,
    event_bus: Arc<dyn GameEventBus + Sync + Send>,
    savefiles: Arc<dyn SavefilesService + Send + Sync>,
    game_history: Arc<dyn GameHistoryService + Send + Sync>,
    activity_ievent_bus: Arc<dyn ActivityInternalEventBus + Sync + Send>,
    _shutdown_tx: broadcast::Sender<()>,
}

const DEFAULT_TURN_TIMEOUT_SECS: u64 = 15;

impl Service {
    pub fn new(
        event_bus: Arc<dyn GameEventBus + Sync + Send>,
        internal_event_bus: Arc<dyn EventBus<GameInternalEvents> + Send + Sync + 'static>,
        savefiles: Arc<dyn SavefilesService + Send + Sync>,
        game_history: Arc<dyn GameHistoryService + Send + Sync>,
        activity_ievent_bus: Arc<dyn ActivityInternalEventBus + Sync + Send>,
    ) -> Arc<Self> {
        let games = DashMap::new();
        let players = DashMap::new();
        let ingame = DashMap::new();
        let leaderboard = DashMap::new();
        
        let (_shutdown_tx, shutdown_rx) = broadcast::channel(1);

        let service = Arc::new(Self {
            games,
            players,
            ingame,
            leaderboard,
            internal_event_bus: internal_event_bus.clone(),
            event_bus,
            savefiles,
            game_history,
            activity_ievent_bus,
            _shutdown_tx,
        });

        super::spawn_service_listener(&service, internal_event_bus.subscribe(), shutdown_rx);

        service
    }

    pub fn players_private_query(&self, game: &Game) -> Vec<PlayerPrivateQuery> {
        game.players().iter().map(|v| {
            self.players.get(v).expect("Game's players are supposed to be in the set.")
                .clone()
                .into()
        }).collect()
    }

    pub fn leaderboard_mut(&self, game_id: Uuid) -> dashmap::mapref::one::RefMut<'_, Uuid, IndexSet<PlayerResult>> {
        self.leaderboard.entry(game_id).or_default()
    }


    pub fn gather_hands(&self, players: &Vec<Uuid>) -> HashMap::<Uuid, Vec<Card>> {
        let mut hands = HashMap::<Uuid, Vec<Card>>::new();
        
        for player in players {
            let hand = self.players.get(player)
                .map(|v| v.cards()
                    .clone()
                );
            if let Some(v) = hand { hands.insert(*player, v); }
        };

        hands
    }

    fn points_for_placement(total_players: usize, placement_index: usize) -> u64 {
        if total_players == 0 {
            return 0;
        }
        ((total_players * 10) * (total_players - placement_index) / total_players) as u64
    }

    async fn persist_game_over_stats(&self, game_id: Uuid, players: &[PlayerResult], participants: &[Uuid]) {
        let total = players.len();
        for (index, player) in players.iter().enumerate() {
            let points = Self::points_for_placement(total, index);
            let player_id = *player.id();
            let cards_had = *player.cards_had();

            let result = async {
                let mut savefile = self.savefiles.load(player_id).await?;
                savefile.games_played += 1;
                if index < total / 2 {
                    savefile.wins += 1;
                } else {
                    savefile.loses += 1;
                }
                savefile.cards_had += cards_had as u64;
                savefile.points += points;
                savefile.max_points = savefile.max_points.max(points as u64);

                self.savefiles
                    .save(UpdateSavefileRequest {
                        id: player_id,
                        games_played: Some(savefile.games_played),
                        points: Some(savefile.points),
                        cards_had: Some(savefile.cards_had),
                        wins: Some(savefile.wins),
                        loses: Some(savefile.loses),
                        max_points: Some(savefile.max_points),
                    })
                    .await?;
                Ok::<(), crate::domain::savefiles::SavefileError>(())
            }
            .await;

            if let Err(e) = result {
                tracing::error!(
                    "Failed to update savefile after game over for {}: {}",
                    player_id,
                    e
                );
            }

            if let Err(err) = self.game_history.record_history(GameHistory::create(
                player_id,
                game_id,
                (total - index) as u32,
                points,
                cards_had as u64,
                participants.to_vec(),
            )).await {
                tracing::error!(
                    "Failed to persist game history after game over for {}: {}",
                    player_id,
                    err
                );
            }
        }
    }

    fn schedule_turn_timeout(&self, game: &Game) {
        let game_id = *game.id();
        let turn = game.turn;
        let turn_enforced_at = game.turn_enforced_at;

        let Some(ingame) = self.ingame.get(&game_id) else { return; };
        let Some(player_id) = ingame.get_index(turn).copied() else { return; };

        let event_bus = self.internal_event_bus.clone();
        tokio::spawn(async move {
            let now = chrono::Utc::now().timestamp_millis();
            let wait_ms = (turn_enforced_at - now).max(0) as u64;
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            event_bus.publish(GameInternalEvents::ForceTurnIfUnchanged {
                game_id,
                player_id,
                turn,
                turn_enforced_at,
            });
        });
    }

}

#[async_trait]
impl super::EventHandler<GameInternalEvents> for Service {
    async fn process(&self, event: GameInternalEvents) {
        match event {
            GameInternalEvents::RequestNewGameSession { player_ids } => {
                if let Err(e) = self.create_game_session(None, player_ids).await {
                    tracing::error!("Failed to create a Game session through internal event bus: {}", e);
                }
            },
            GameInternalEvents::RemovePlayer{ game_id, player_id } => {
                let res = self.remove_player(game_id, player_id).await;
                if let Err(e) = res {
                    tracing::error!("Failed to remove a Game player through internal event bus: {}", e);
                }
            },
            GameInternalEvents::ForceTurnIfUnchanged { game_id, player_id, turn, turn_enforced_at } => {
                let should_force = self.games.get(&game_id)
                    .map(|game| game.turn == turn && game.turn_enforced_at == turn_enforced_at)
                    .unwrap_or(false);
                if !should_force {
                    return;
                }

                let current_turn_player = self.ingame.get(&game_id)
                    .and_then(|ingame| ingame.get_index(turn).copied());
                if current_turn_player != Some(player_id) {
                    return;
                }

                if let Err(e) = self.play_card(game_id, player_id, None).await {
                    tracing::error!("Failed to force timed-out player move: {}", e);
                }
            }
        };
    }
}

#[async_trait]
impl GameService for Service {
    async fn create_game_session(&self, initiator_player_id: Option<Uuid>, player_ids: Vec<Uuid>) -> Result<GameQuery, GameError> {
        if player_ids.len() < 2 { return Err(GameError::NotEnoughPlayers) }; 

        let game = Game::init(player_ids.clone(), DEFAULT_TURN_TIMEOUT_SECS);
        let ingame = player_ids.iter().cloned().collect();
        let mut players_query = Vec::new();

        let id = *game.id();
        self.games.insert(id, game.clone());
        for player_id in player_ids {
            let player: Player = player_id.into();
            players_query.push(player.clone().into());
            self.players.insert(player_id, player);
        }
        self.ingame.insert(id, ingame);

        for player in game.players() {
            let event_bus = self.event_bus.clone();
            let activity_ievent_bus = self.activity_ievent_bus.clone();
            let game_id = *game.id();
            let id = *player;

            tokio::spawn(async move {
                event_bus.new_channel(id);
                activity_ievent_bus.user_joined_game(game_id, id).await;
            });
        }

        let initiator_hand = initiator_player_id.map(|v| 
            self.players.get(&v)
                .map(|v| v.cards()
                    .clone()
                )
        ).unwrap_or_default();

        let res = GameQuery::from_game(
            game,
            players_query,
            vec![],
            initiator_hand
        );

        if let Some(game) = self.games.get(&id) {
            self.schedule_turn_timeout(&game);
        }

        Ok(res)
    }

    async fn read_game_session_as_player(&self, player_id: Uuid, game_id: Uuid) -> Result<GameQuery, GameError> {
        let game = self.games.get(&game_id)
            .ok_or(GameError::GameNotFound)?
            .clone();
        let hand = self.players.get(&player_id)
            .map(|v| v.cards()
                .clone()
            );
        let players = self.players_private_query(&game);

        let res = GameQuery::from_game(
            game,
            players,
            self.leaderboard_mut(game_id).iter().cloned().collect(),
            hand
        );

        Ok(res)
    }

    async fn play_card(&self, game_id: Uuid, player_id: Uuid, card_id: Option<usize>) -> Result<PlayCard, GameError> {
        let (game, acting_player_hand) = {
            let mut game = self.games.get_mut(&game_id)
                .ok_or(GameError::GameNotFound)?;
            let mut ingame = self.ingame.get_mut(&game_id)
                .ok_or(GameError::GameNotFound)?;

            let mut step = 1;
            let index = game.get_player_index(player_id)?;
            if index != game.turn {
                return Err(GameError::WrongTurn);
            }
            let mut cards_to_pick = 0;

            if let Some(card_index) = card_id {
                let (effect, eliminated) = {
                    let mut player = self.players.get_mut(&player_id)
                        .ok_or(GameError::PlayerNotFound)?;
                    let card = player.get_card(card_index).ok_or(GameError::CardNotFound)?;
                    let effect = card
                        .play(
                            game.history
                                .last()
                                .expect("Every game is supposed to have at least the starting card in history")
                                .card(),
                        )
                        .map_err(|_| GameError::WrongCard)?;

                    game.history.push(Turn::new(Some(player_id), card.clone()));
                    player.remove_card(card_index);

                    let eliminated = if player.cards().is_empty() {
                        Some(player.clone().into())
                    } else {
                        None
                    };
                    (effect, eliminated)
                };

                if let Some(player_result) = eliminated {
                    self.leaderboard_mut(game_id).insert(player_result);
                    ingame.shift_remove(&player_id);
                }

                match effect {
                    Effect::Stun => step += 1,
                    Effect::Flow => {
                        game.direction.switch();
                    }
                    Effect::Add(num) => cards_to_pick = num,
                    _ => {}
                }
            } else {
                let mut player = self.players.get_mut(&player_id)
                    .ok_or(GameError::PlayerNotFound)?;
                player.add_card(rand::random());
            }

            let next_turn = match game.direction {
                Direction::Next => game.turn as isize + step,
                Direction::Previous => game.turn as isize - step,
            };

            if next_turn < 0 {
                game.turn = (next_turn + ingame.len() as isize) as usize;
            } else if next_turn >= ingame.len() as isize {
                game.turn = (next_turn - ingame.len() as isize) as usize;
            } else {
                game.turn = next_turn as usize;
            }
            game.turn_enforced_at = chrono::Utc::now().timestamp_millis() + (game.turn_timeout_secs as i64 * 1000);

            for _ in 0..cards_to_pick {
                let next_player_id = *ingame
                    .get_index(game.turn)
                    .expect("game.turn isn't supposed to be outside the active players set");
                let mut next_player = self.players.get_mut(&next_player_id)
                    .ok_or(GameError::PlayerNotFound)?;
                next_player.add_card(rand::random());
            }

            let acting_player_hand = self
                .players
                .get(&player_id)
                .map(|p| p.cards().clone());

            (game.clone(), acting_player_hand)
        };

        let mut leaderboard: Vec<PlayerResult> = self.leaderboard_mut(game_id).iter().cloned().collect();
        let hands = self.gather_hands(game.players());
        let ingame_count = self
            .ingame
            .get(&game_id)
            .map(|set| set.len())
            .unwrap_or(0);

        if ingame_count <= 1 {
            if let Some(mut ingame) = self.ingame.get_mut(&game_id) {
                if let Some(winner_id) = ingame.pop() {
                    let winner = self
                        .players
                        .get(&winner_id)
                        .expect("Ingame player is supposed to have it's state in players var");
                    self.leaderboard_mut(game_id).insert(winner.clone().into());
                }
            }

            leaderboard = self.leaderboard_mut(game_id).iter().cloned().collect();

            self.persist_game_over_stats(game_id, &leaderboard, game.players()).await;

            for player in game.players() {
                let hand = hands.get(player).cloned();
                let event_bus = self.event_bus.clone();
                let game = game.clone();
                let leaderboard = leaderboard.clone();
                let id = *player;

                tokio::spawn(async move {
                    event_bus
                        .game_over(id, GameOverQuery::from_game(game, leaderboard, hand))
                        .await;
                    event_bus.close_channel(id);
                });
            }

            let game_id_copy = *game.id();
            self.games.remove(&game_id_copy);
            self.ingame.remove(&game_id_copy);
            self.leaderboard.remove(&game_id_copy);

            for player in leaderboard.iter() {
                let left_player_id = *player.id();
                let activity_ievent_bus = self.activity_ievent_bus.clone();
                tokio::spawn(async move {
                    activity_ievent_bus.user_left_game(left_player_id).await;
                });
            }

            let gameover = GameOverQuery::from_game(
                game.clone(),
                leaderboard,
                acting_player_hand,
            );

            return Ok(PlayCard::GameOver(gameover));
        }

        let players_private_query = self.players_private_query(&game);
        self.schedule_turn_timeout(&game);
        for player in game.players() {
            let hand = hands.get(player).cloned();
            let event_bus = self.event_bus.clone();
            let game = game.clone();
            let players_private_query = players_private_query.clone();
            let leaderboard = leaderboard.clone();
            let id = *player;

            tokio::spawn(async move {
                event_bus
                    .game_new_turn(
                        id,
                        GameNewTurnQuery::from_game(game, players_private_query, leaderboard, hand),
                    )
                    .await;
            });
        }

        let new_turn = GameNewTurnQuery::from_game(
            game,
            players_private_query,
            leaderboard,
            acting_player_hand,
        );

        Ok(PlayCard::NextTurn(new_turn))
    }

    async fn remove_player(&self, game_id: Uuid, player_id: Uuid) -> Result<(), GameError> {
        let mut ingame = self.ingame.get_mut(&game_id)
            .ok_or(GameError::GameNotFound)?;
        if ingame.shift_remove(&player_id) == false {
            return Err(GameError::PlayerNotFound);
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct DummyGameEventBus;
    #[async_trait]
    impl GameEventBus for DummyGameEventBus {
        fn new_channel(&self, _player_id: Uuid) {}
        fn close_channel(&self, _player_id: Uuid) {}
        fn subscribe(&self, _player_id: Uuid) -> Option<broadcast::Receiver<GameEvents>> { None }
        fn publish(&self, _player_id: Uuid, _event: GameEvents) {}
        async fn game_new_turn(&self, _player_id: Uuid, _query: GameNewTurnQuery) {}
        async fn game_over(&self, _player_id: Uuid, _query: GameOverQuery) {}
    }

    struct DummyEventBus;
    impl<T: Clone + Send + Sync + 'static> EventBus<T> for DummyEventBus {
        fn subscribe(&self) -> broadcast::Receiver<T> {
            let (sender, _) = broadcast::channel(1);
            sender.subscribe()
        }
        fn publish(&self, _event: T) {}
    }

    struct DummySavefilesService;
    #[async_trait]
    impl SavefilesService for DummySavefilesService {
        async fn init(&self, user_id: Uuid) -> Result<crate::domain::savefiles::Savefile, crate::domain::savefiles::SavefileError> { 
            Ok(crate::domain::savefiles::Savefile::new(user_id, 0, 0, 0, 0, 0, 0))
        }
        async fn load(&self, user_id: Uuid) -> Result<crate::domain::savefiles::Savefile, crate::domain::savefiles::SavefileError> { 
            Ok(crate::domain::savefiles::Savefile::new(user_id, 0, 0, 0, 0, 0, 0))
        }
        async fn save(&self, _cmd: UpdateSavefileRequest) -> Result<crate::domain::savefiles::Savefile, crate::domain::savefiles::SavefileError> { unimplemented!() }
        async fn delete(&self, _id: Uuid) -> Result<(), crate::domain::savefiles::SavefileError> { Ok(()) }
        async fn get_the_best(&self) -> Result<Vec<crate::domain::savefiles::Savefile>, crate::domain::InternalError> { Ok(vec![]) }
    }

    struct DummyGameHistoryService;
    #[async_trait]
    impl GameHistoryService for DummyGameHistoryService {
        async fn record_history(&self, _cmd: GameHistory) -> Result<GameHistory, crate::domain::game_history::GameHistoryError> { Ok(_cmd) }
        async fn list_player_history(&self, _account_id: Uuid, _after: Option<Uuid>, _limit: Option<u32>) -> Result<Vec<GameHistory>, crate::domain::InternalError> { Ok(vec![]) }
    }

    struct DummyActivityInternalEventBus;
    #[async_trait]
    impl ActivityInternalEventBus for DummyActivityInternalEventBus {
        async fn user_joined_room(&self, _room_id: String, _user_id: Uuid) {}
        async fn user_left_room(&self, _user_id: Uuid) {}
        async fn user_joined_game(&self, _game_id: Uuid, _user_id: Uuid) {}
        async fn user_left_game(&self, _user_id: Uuid) {}
    }

    #[tokio::test]
    async fn create_game_session_with_two_players() {
        let event_bus = Arc::new(DummyGameEventBus);
        let internal_event_bus = Arc::new(DummyEventBus);
        let savefiles = Arc::new(DummySavefilesService);
        let game_history = Arc::new(DummyGameHistoryService);
        let activity_ievent_bus = Arc::new(DummyActivityInternalEventBus);

        let service = Service::new(event_bus, internal_event_bus, savefiles, game_history, activity_ievent_bus);

        let player1 = Uuid::now_v7();
        let player2 = Uuid::now_v7();
        let players = vec![player1, player2];

        let result = service.create_game_session(Some(player1), players.clone()).await;
        assert!(result.is_ok());
        
        let game_query = result.unwrap();
        assert_eq!(game_query.players.len(), 2);
    }

    #[tokio::test]
    async fn create_game_fails_with_one_player() {
        let event_bus = Arc::new(DummyGameEventBus);
        let internal_event_bus = Arc::new(DummyEventBus);
        let savefiles = Arc::new(DummySavefilesService);
        let game_history = Arc::new(DummyGameHistoryService);
        let activity_ievent_bus = Arc::new(DummyActivityInternalEventBus);

        let service = Service::new(event_bus, internal_event_bus, savefiles, game_history, activity_ievent_bus);

        let player1 = Uuid::now_v7();
        let players = vec![player1];

        let result = service.create_game_session(Some(player1), players).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_player_from_game() {
        let event_bus = Arc::new(DummyGameEventBus);
        let internal_event_bus = Arc::new(DummyEventBus);
        let savefiles = Arc::new(DummySavefilesService);
        let game_history = Arc::new(DummyGameHistoryService);
        let activity_ievent_bus = Arc::new(DummyActivityInternalEventBus);

        let service = Service::new(event_bus, internal_event_bus, savefiles, game_history, activity_ievent_bus);

        let player1 = Uuid::now_v7();
        let player2 = Uuid::now_v7();
        let players = vec![player1, player2];

        let game = service.create_game_session(Some(player1), players).await.expect("create game");
        let game_id = game.id;

        let result1 = service.remove_player(game_id, player1).await;
        assert!(result1.is_ok());

        let result2 = service.remove_player(game_id, Uuid::now_v7()).await;
        assert!(result2.is_err());
    }
}