use crate::category::{self, Category};
use crate::dice::DiceSet;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Scoreboard(HashMap<Category, u32>);

impl Scoreboard {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn game_is_finished(&self) -> bool {
        category::CATEGORIES
            .iter()
            .all(|category| self.0.contains_key(category))
    }

    pub fn category_is_available(&self, category: Category, dice_set: &DiceSet) -> bool {
        self.0
            .get(&category)
            .map(|score| {
                category == Category::FiveOfAKind && *score > 0 && dice_set.score(category) > 0
            })
            .unwrap_or(true)
    }

    pub fn choose_category(&mut self, category: Category, dice_set: &DiceSet) {
        let score = dice_set.score(category);

        match self.0.entry(category) {
            Entry::Occupied(mut o) if category == Category::FiveOfAKind && *o.get() > 0 => {
                *o.get_mut() += score
            }
            Entry::Vacant(v) => {
                v.insert(score);
            }
            _ => panic!("Unavailable category"),
        }
    }

    pub fn category_score(&self, category: Category) -> Option<u32> {
        self.0.get(&category).copied()
    }

    pub fn upper_section_bonus(&self) -> u32 {
        if self.basic_total() > 62 {
            35
        } else {
            0
        }
    }

    pub fn upper_total(&self) -> u32 {
        self.basic_total() + self.upper_section_bonus()
    }

    pub fn lower_total(&self) -> u32 {
        category::LOWER_SECTION
            .iter()
            .filter_map(|category| self.0.get(category))
            .sum()
    }

    pub fn grand_total(&self) -> u32 {
        self.upper_total() + self.lower_total()
    }

    fn basic_total(&self) -> u32 {
        category::UPPER_SECTION
            .iter()
            .filter_map(|category| self.0.get(category))
            .sum()
    }
}
