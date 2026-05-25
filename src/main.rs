mod infrastructure;
mod domain;
mod application;

use poem::{
    middleware::{ AddData, Cors }, EndpointExt
};
use chrono::TimeDelta;
use shuttle_poem::ShuttlePoem;
use shuttle_runtime::SecretStore;
use std::sync::Arc;

use seaorm_migration::{Migrator, MigratorTrait};
use infrastructure::http::*;

use infrastructure::postgres::Postgres;
use infrastructure::memory::room::RoomStorage;

use infrastructure::eventbus::room::RoomEventBus;
use infrastructure::eventbus::room::RoomIEventBus;
use infrastructure::eventbus::game::*;
use infrastructure::eventbus::activity::ActivityEventBus;
use infrastructure::eventbus::activity::ActivityIEventBus;

type AccountsService = application::Accounts<Postgres>;
type TokenProvider = application::Jwt;
type AuthService = application::Auth<
        AccountsService,
        Postgres,
        TokenProvider>;
type SavefilesService = Arc<dyn domain::savefiles::SavefilesService + Send + Sync>;
//type RolesService = application::Roles<Postgres>;
type RoomService = application::Room;
type GameService = application::Game;
type ActivityService = application::Activity;

struct AppState {
    accounts: Arc<AccountsService>,
    auth: AuthService,
    savefiles: SavefilesService,
    //roles: RolesService,
    rooms: Arc<RoomService>,
    games: Arc<GameService>,
    activities: Arc<ActivityService>,
    room_event_bus: Arc<RoomEventBus>,
    game_event_bus: Arc<GameEventBus>,
    activity_event_bus: Arc<ActivityEventBus>,
}

impl AppState {
    pub fn new(db: Postgres, secret_store: SecretStore) -> anyhow::Result<Self> {
        let secret = secret_store.get("JWT_SECRET")
            .ok_or_else(|| anyhow::anyhow!("JWT_SECRET is missing"))?;
            
        let refresh_expires_after = secret_store.get("JWT_REFRESH_EXPIRES")
            .and_then(|v| v.parse::<i64>().ok())
            .map(TimeDelta::days)
            .unwrap_or_else(|| TimeDelta::days(30));

        let access_expires_after = secret_store.get("JWT_ACCESS_EXPIRES")
            .and_then(|v| v.parse::<i64>().ok())
            .map(TimeDelta::minutes)
            .unwrap_or_else(|| TimeDelta::minutes(15));

        let room_storage = Arc::new(RoomStorage::new(Arc::new(db.clone())));

        let room_event_bus = Arc::new(RoomEventBus::new(12));
        let room_ievent_bus = Arc::new(RoomIEventBus::new(12));
        let game_event_bus = Arc::new(GameEventBus::new(12));
        let game_ievent_bus = Arc::new(GameIEventBus::new(12));
        let activity_event_bus = Arc::new(ActivityEventBus::new(12));
        let activity_ievent_bus = Arc::new(ActivityIEventBus::new(12));

        let accounts = Arc::new(AccountsService::new(db.clone()));
        let jwt = TokenProvider::new(secret, refresh_expires_after, access_expires_after);
        let auth = AuthService::new(accounts.clone(), db.clone(), jwt);
        let savefiles: SavefilesService =
            Arc::new(application::Savefiles::new(db.clone()));
        //let roles = RolesService::new(db);
        let games = GameService::new(
            game_event_bus.clone(),
            game_ievent_bus.clone(),
            savefiles.clone(),
            activity_ievent_bus.clone(),
        );
        let activities = ActivityService::new(
            application::activity::ACTIVITY_HEARTBEAT_SECS,
            room_ievent_bus.clone(),
            game_ievent_bus.clone(),
            activity_event_bus.clone(),
            activity_ievent_bus.clone(),
        );
        let rooms = RoomService::new(
            room_storage.clone(),
            room_storage,
            room_event_bus.clone(),
            room_ievent_bus.clone(),
            game_ievent_bus.clone(),
            activity_ievent_bus.clone(),
            activities.clone(),
        );

        Ok(Self {
            accounts,
            auth,
            savefiles,
            //roles,
            rooms,
            games,
            activities
            ,room_event_bus: room_event_bus.clone(),
            game_event_bus: game_event_bus.clone(),
            activity_event_bus: activity_event_bus.clone(),
        })
    }
}

#[shuttle_runtime::main]
async fn poem(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttlePoem<impl poem::Endpoint> {
    let db = if let (Some(host), Some(name), Some(Ok(port)), Some(user), Some(pass)) = (
        secret_store.get("DB_HOST"),
        secret_store.get("DB_NAME"),
        secret_store.get("DB_PORT").map(|v| v.parse()),
        secret_store.get("DB_USER"),
        secret_store.get("DB_PASS"),
    ) {
        let pg_details = infrastructure::postgres::PostgresDetails::new(
            host.as_str(),
            name.as_str(),
            port,
            user.as_str(),
            pass.as_str()
        );

        infrastructure::postgres::Postgres::from_details(pg_details).await
            .map_err(|e| shuttle_runtime::Error::Database(shuttle_runtime::CustomError::new(e).to_string()))
    } else {
        Err(shuttle_runtime::Error::Database("Not all database parameters were provided. The execution has been aborted!".to_string())) //В іншому випадку повернути у змінну db помилку
    };
    
    match db {
        Ok(db) => { //Якщо змінна db містить з'єднання
            Migrator::up(db.connection(), None)
                .await
                .map_err(|e| shuttle_runtime::Error::Database(shuttle_runtime::CustomError::new(e).to_string()))?;

            let state = AppState::new(db.clone(), secret_store)?;
            
            let app = api_routes(Arc::new(state.auth.clone()), Arc::new(db.clone()))
            .with(Cors::new().allow_origin_regex("*")) //Налаштування CORS політики
            .with(AddData::new(Arc::new(db.clone()))) //Передача посилання на з'єднання БД в аргументи функцій
            .with(AddData::new(Arc::new(state)));
            Ok(app.into()) //Завершення налаштування та передача Route в Shuttle Runtime.
        }
        Err(e) => {
            Err(e) //Інакше передати помилку.
        }
    }
}
