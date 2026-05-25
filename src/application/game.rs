use std::{sync::Arc, vec};

use std::collections::hash_map::HashMap;
use indexmap::IndexSet;
use uuid::Uuid;
use dashmap::DashMap;
use crate::domain::activity::ActivityInternalEventBus;
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
    event_bus: Arc<dyn GameEventBus + Sync + Send>,
    savefiles: Arc<dyn SavefilesService + Send + Sync>,
    activity_ievent_bus: Arc<dyn ActivityInternalEventBus + Sync + Send>,
    _shutdown_tx: broadcast::Sender<()>,
}

impl Service {
    pub fn new(
        event_bus: Arc<dyn GameEventBus + Sync + Send>,
        internal_event_bus: Arc<dyn EventBus<GameInternalEvents> + Send + Sync + 'static>,
        savefiles: Arc<dyn SavefilesService + Send + Sync>,
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
            event_bus,
            savefiles,
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

    async fn persist_game_over_stats(&self, players: &[PlayerResult]) {
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
                savefile.max_points = savefile.max_points.max(points as u16);

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
        }
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
            }
        };
    }
}

#[async_trait]
impl GameService for Service {
    async fn create_game_session(&self, initiator_player_id: Option<Uuid>, player_ids: Vec<Uuid>) -> Result<GameQuery, GameError> {
        if player_ids.len() < 2 { return Err(GameError::NotEnoughPlayers) }; 

        let game = Game::init(player_ids.clone());
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
        // Phase 1: mutate game state only — never call helpers that lock `players` while holding
        // `players.get_mut` (DashMap will deadlock on the same shard).
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

        // Phase 2: read-only queries and side effects — all DashMap guards from phase 1 are dropped.
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

            self.persist_game_over_stats(&leaderboard).await;

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