use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use super::*;
impl Card { //Методи структури Card
    pub fn new(element: Element, effect: Effect) -> Self { //Конструктор
        Self {
            element,
            effect,
        }
    }

    pub fn play(&self, card: &Self) -> Result<Effect, ()> { //Метод для битви двох карт
        let coef = self.element.coefficient(card.element); //визначення коефіцієнту
        let other_power = match card.effect { //Визначення базової атаки попередньої карти
            Effect::Atk(power) => power, //Якщо ефект Atk то це є значенням цього ефекту
            _ => 1, //інакше 1
        };
        match self.effect { //Обробка ефектів щойно зіграної карти
            Effect::Atk(power) => { //Якщо ефект Atk
                if (power as f32 *coef).round() < other_power as f32 { return Err(()) } //То порівняти базову атаку помножену на коефіцієнт з атакою попередньої карти
                //повернути помилку якщо менше
                Ok(Effect::Atk(power))
            },
            effect => { //Якщо будь-який інший ефект
                if coef < 1.0 { return Err(()) } //повернути помилку якщо коефіцієнт менший за 1
                Ok(effect)
            },
        }
        //при успіху функція повертає ефект
    }
}

impl Distribution<Card> for Standard { //Генератор рандомних значень для Card
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Card {
        Card {
            element: rand::random(),
            effect: rand::random(), 
        }

    }
}