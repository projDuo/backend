use super::RoomId;
use random_string;

impl RoomId {
    pub fn generate() -> Self { //Метод генерування ідентифікатора, що складається з 6 чисел
        Self(random_string::generate(6, "0123456789"))
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}