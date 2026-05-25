use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Identify { //Структура, що описує подію ідентифікації
    pub token: String,
}