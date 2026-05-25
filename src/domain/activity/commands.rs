use uuid::Uuid;

use crate::domain::activity::Activity;


#[derive(Debug, Clone)]
pub struct MarkActivityCommand {
    pub id: Uuid,
    pub room: Option<Option<String>>,
    pub game: Option<Option<Uuid>>,
}

impl MarkActivityCommand {
    pub fn new(
        id: Uuid,
    ) -> Self {
        Self { id, room: None, game: None }
    }

    pub fn room(&mut self, room: Option<String>) -> &mut Self {
        self.room = Some(room);
        self
    }

    pub fn game(&mut self, game: Option<Uuid>) -> &mut Self {
        self.game = Some(game);
        self
    }
}

impl From<Uuid> for MarkActivityCommand {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl Into<Activity> for MarkActivityCommand {
    fn into(self) -> Activity {
        let room = self.room.unwrap_or_default();
        let game = self.game.unwrap_or_default();
        Activity::new(self.id, room, game)
    }
}

impl From<Activity> for MarkActivityCommand {
    fn from(value: Activity) -> Self {
        Self::new(*value.id())
            .room(value.room)
            .game(value.game)
            .to_owned()
    }
}