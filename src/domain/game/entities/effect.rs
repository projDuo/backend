use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use super::Effect;
impl Distribution<Effect> for Standard { //Генератор рандомних значень для списку ефектів
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Effect {
        match rng.gen_range(0..72) {
            0..=59 => Effect::Atk(rng.gen_range(1..=12)), //~83%
            60..=63 => Effect::Flow, //~5,5%
            64..=67 => Effect::Stun, //~5,5%
            _ => Effect::Add(rng.gen_range(1..=4)), //~5,5%
        }
    }
}