use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use super::Element;
impl Element {
    pub fn index(&self) -> usize { //Геттер позиції наданого елементу
        *self as usize
    }

    //self - карта зіграна щойно
    //other - карта зіграна попереднім гравцем
    pub fn coefficient(&self, other: Self) -> f32 {
        if *self == Element::Energy || other == Element::Energy { return 1.0 }; //Якщо елемент однієї з двох карток, то повернути коефіцієнт 1.0 
        //Визначення позицій карток
        let pos = self.index() as isize + 1;
        let other_pos = other.index() as isize + 1;
        //Визначення середньої позиції списку
        let half = (Element::Energy as isize - 1)/2;
        let mut distance = if pos <= other_pos { //якщо елемент карти зіграної щойно в списку стоїть до елементу карти зіграної попереднім гравцем
            other_pos - pos //то більше відняти за менше
        } else {
            Element::Energy.index() as isize + other_pos - pos //інакше зробити теж саме, але також додати індекс останнього елементу для забезпечення циклічності елементів
            //це необхідно так як елемент карти зіграної щойно розташований після елементу карти зіграної п.г і тому йому необхідно пройти до кінця списку,
            //почати з початку і тільки потім дійти  
        };
        if distance == 0 { return 1.0 } // якщо елементи однакові то повернути коефіцієнт 1.0
        if distance > half { distance += 1 } //ребалансування після втручення в формулу коефіцієнту в попередньому рядку
        else {
            distance -= 1;
        }
        0.50 + (Element::Energy as isize - distance) as f32 / 4_f32 //Повернення значення обчисленого за формулою для визначення коефіцієнту
    }
}

impl Distribution<Element> for Standard { //Релізація генератора рандомних значень для списку елементів
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Element {
        match rng.gen_range(0..=5) {
            0 => Element::Water,
            1 => Element::Fire,
            2 => Element::Wood,
            3 => Element::Earth,
            4 => Element::Air,
            _ => Element::Energy,
        }
    }
}