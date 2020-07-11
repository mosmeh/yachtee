use crate::category::Category;

use anyhow::Result;
use crossterm::style::{ContentStyle, StyledContent};
use crossterm::{cursor, style};
use itertools::Itertools;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::io;

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Dice(u8);

impl From<u8> for Dice {
    fn from(x: u8) -> Dice {
        Dice(x)
    }
}

impl Distribution<Dice> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dice {
        Dice(rng.gen_range(0, Dice::NUM_FACES as u8))
    }
}

impl Dice {
    pub const NUM_FACES: usize = 6;
    pub const WIDTH: u16 = 9;
    pub const HEIGHT: u16 = 5;

    pub fn draw<W: io::Write>(&self, w: &mut W, x: u16, y: u16) -> Result<()> {
        self.draw_styled(w, x, y, ContentStyle::new())
    }

    pub fn draw_styled<W: io::Write>(
        &self,
        w: &mut W,
        x: u16,
        y: u16,
        style: ContentStyle,
    ) -> Result<()> {
        for (line, y) in DICE_FACES[self.0 as usize]
            .split('\n')
            .zip(y..Dice::HEIGHT + y)
        {
            crossterm::queue!(
                w,
                cursor::MoveTo(x, y),
                style::PrintStyledContent(StyledContent::new(style.clone(), line))
            )?;
        }

        Ok(())
    }
}

pub struct DiceSet(pub [Dice; Self::NUM_DICE]);

impl Distribution<DiceSet> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DiceSet {
        DiceSet(rng.gen())
    }
}

impl DiceSet {
    pub const NUM_DICE: usize = 5;

    pub fn score(&self, category: Category) -> u32 {
        match category {
            Category::Ones => self.count(0) as u32,
            Category::Twos => self.count(1) as u32 * 2,
            Category::Threes => self.count(2) as u32 * 3,
            Category::Fours => self.count(3) as u32 * 4,
            Category::Fives => self.count(4) as u32 * 5,
            Category::Sixes => self.count(5) as u32 * 6,
            Category::ThreeOfAKind if self.matches_n_of_a_kind(3) => self.sum(),
            Category::FourOfAKind if self.matches_n_of_a_kind(4) => self.sum(),
            Category::FullHouse
                if {
                    let mut counts = [0; Dice::NUM_FACES];
                    for x in &self.0 {
                        counts[x.0 as usize] += 1;
                    }
                    counts.iter().any(|x| *x == 2) && counts.iter().any(|x| *x == 3)
                } =>
            {
                25
            }
            Category::SmallStraight if self.matches_straight(4) => 30,
            Category::LargeStraight if self.matches_straight(5) => 40,
            Category::FiveOfAKind if self.0.iter().all_equal() => 50,
            Category::Chance => self.sum(),
            _ => 0,
        }
    }

    fn count(&self, i: u8) -> usize {
        self.0.iter().filter(|x| x.0 == i).count()
    }

    fn sum(&self) -> u32 {
        self.0.iter().map(|x| x.0 as u32 + 1).sum()
    }

    fn matches_n_of_a_kind(&self, n: u8) -> bool {
        let mut counts = [0; Dice::NUM_FACES];
        for x in &self.0 {
            counts[x.0 as usize] += 1;
            if counts[x.0 as usize] >= n {
                return true;
            }
        }
        false
    }

    fn matches_straight(&self, n: u8) -> bool {
        for start in 0..=Dice::NUM_FACES as u8 - n {
            if (start..start + n).all(|x| self.0.contains(&Dice::from(x))) {
                return true;
            }
        }
        false
    }
}

const DICE_FACES: [&str; Dice::NUM_FACES] = [
    "╭───────╮\n\
     │       │\n\
     │   ●   │\n\
     │       │\n\
     ╰───────╯",
    "╭───────╮\n\
     │ ●     │\n\
     │       │\n\
     │     ● │\n\
     ╰───────╯",
    "╭───────╮\n\
     │ ●     │\n\
     │   ●   │\n\
     │     ● │\n\
     ╰───────╯",
    "╭───────╮\n\
     │ ●   ● │\n\
     │       │\n\
     │ ●   ● │\n\
     ╰───────╯",
    "╭───────╮\n\
     │ ●   ● │\n\
     │   ●   │\n\
     │ ●   ● │\n\
     ╰───────╯",
    "╭───────╮\n\
     │ ●   ● │\n\
     │ ●   ● │\n\
     │ ●   ● │\n\
     ╰───────╯",
];
