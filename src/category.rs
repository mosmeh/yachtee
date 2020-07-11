use std::fmt;

pub const CATEGORIES: [Category; 13] = [
    Category::Ones,
    Category::Twos,
    Category::Threes,
    Category::Fours,
    Category::Fives,
    Category::Sixes,
    Category::ThreeOfAKind,
    Category::FourOfAKind,
    Category::FullHouse,
    Category::SmallStraight,
    Category::LargeStraight,
    Category::FiveOfAKind,
    Category::Chance,
];

pub const UPPER_SECTION: [Category; 6] = [
    Category::Ones,
    Category::Twos,
    Category::Threes,
    Category::Fours,
    Category::Fives,
    Category::Sixes,
];

pub const LOWER_SECTION: [Category; 7] = [
    Category::ThreeOfAKind,
    Category::FourOfAKind,
    Category::FullHouse,
    Category::SmallStraight,
    Category::LargeStraight,
    Category::FiveOfAKind,
    Category::Chance,
];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Ones,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    ThreeOfAKind,
    FourOfAKind,
    FullHouse,
    SmallStraight,
    LargeStraight,
    FiveOfAKind,
    Chance,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Category::Ones => f.write_str("⚀ 1s"),
            Category::Twos => f.write_str("⚁ 2s"),
            Category::Threes => f.write_str("⚂ 3s"),
            Category::Fours => f.write_str("⚃ 4s"),
            Category::Fives => f.write_str("⚄ 5s"),
            Category::Sixes => f.write_str("⚅ 6s"),
            Category::ThreeOfAKind => f.write_str("3 of a Kind"),
            Category::FourOfAKind => f.write_str("4 of a Kind"),
            Category::FullHouse => f.write_str("Full House"),
            Category::SmallStraight => f.write_str("Small Straight"),
            Category::LargeStraight => f.write_str("Large Straight"),
            Category::FiveOfAKind => f.write_str("5 of a Kind"),
            Category::Chance => f.write_str("Chance"),
        }
    }
}
