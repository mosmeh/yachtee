mod category;
mod dice;
mod scoreboard;
mod table_printer;

use dice::{Dice, DiceSet};
use scoreboard::Scoreboard;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, style, terminal};
use rand::Rng;
use std::io::{self, Write};

const MAX_ROLLS: usize = 3;
const MAX_MESSAGE_LINES: usize = 3;

fn main() -> Result<()> {
    Game::new().run()?;
    Ok(())
}

struct Game {
    dice_set: DiceSet,
    scoreboard: Scoreboard,
    roll_count: usize,
    selected_category_index: Option<usize>,
    dice_selected: [bool; DiceSet::NUM_DICE],
}

impl Game {
    fn new() -> Self {
        Self {
            dice_set: rand::thread_rng().gen(),
            scoreboard: Scoreboard::new(),
            roll_count: 1,
            selected_category_index: Some(0),
            dice_selected: [false; DiceSet::NUM_DICE],
        }
    }

    fn run(mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();
        std::thread::spawn(move || loop {
            if let Ok(event) = event::read() {
                let _ = tx.send(event);
            }
        });

        let mut stdout = setup_terminal()?;

        loop {
            self.draw(&mut stdout)?;
            stdout.flush()?;

            if let Event::Key(key) = rx.recv()? {
                match (key.modifiers, key.code) {
                    (_, KeyCode::Esc)
                    | (KeyModifiers::CONTROL, KeyCode::Char('c'))
                    | (_, KeyCode::Char('q')) => break,
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) | (_, KeyCode::Char('w')) => {
                        self.on_up();
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) | (_, KeyCode::Char('s')) => {
                        self.on_down();
                    }
                    (_, KeyCode::Home) => self.on_home(),
                    (_, KeyCode::End) => self.on_end(),
                    (_, KeyCode::Enter) | (_, KeyCode::Char(' ')) => self.on_enter(),
                    (_, KeyCode::Char('r')) => self.on_r_key(),
                    (_, KeyCode::Char(c)) if c.is_digit(10) => self.on_number_key(c),
                    _ => (),
                }
            }

            if self.scoreboard.game_is_finished() {
                break;
            }
        }

        self.draw(&mut stdout)?;
        crossterm::queue!(
            stdout,
            cursor::MoveTo(
                0,
                // roll count
                1
                // margin
                + 1
                // main content
                + Dice::HEIGHT * DiceSet::NUM_DICE as u16
                // margin
                + 1
                // message
                + MAX_MESSAGE_LINES as u16
                // margin
                + 1
            )
        )?;
        stdout.flush()?;
        cleanup_terminal(stdout)?;

        Ok(())
    }

    fn on_up(&mut self) {
        const N: usize = category::CATEGORIES.len();
        let i = (self.selected_category_index.unwrap() + N - 1) % N;
        self.selected_category_index = Some(i);
        self.select_prev_available_category();
    }

    fn on_down(&mut self) {
        let i = (self.selected_category_index.unwrap() + 1) % category::CATEGORIES.len();
        self.selected_category_index = Some(i);
        self.select_next_available_category();
    }

    fn on_home(&mut self) {
        self.selected_category_index = Some(0);
        self.select_next_available_category();
    }

    fn on_end(&mut self) {
        self.selected_category_index = Some(category::CATEGORIES.len() - 1);
        self.select_prev_available_category();
    }

    fn select_prev_available_category(&mut self) {
        const N: usize = category::CATEGORIES.len();

        while !self.scoreboard.category_is_available(
            category::CATEGORIES[self.selected_category_index.unwrap()],
            &self.dice_set,
        ) {
            let i = (self.selected_category_index.unwrap() + N - 1) % N;
            self.selected_category_index = Some(i);
        }
    }

    fn select_next_available_category(&mut self) {
        while !self.scoreboard.category_is_available(
            category::CATEGORIES[self.selected_category_index.unwrap()],
            &self.dice_set,
        ) {
            let i = (self.selected_category_index.unwrap() + 1) % category::CATEGORIES.len();
            self.selected_category_index = Some(i);
        }
    }

    fn on_enter(&mut self) {
        let category = category::CATEGORIES[self.selected_category_index.unwrap()];
        self.scoreboard.choose_category(category, &self.dice_set);

        self.dice_selected = [false; DiceSet::NUM_DICE];

        if self.scoreboard.game_is_finished() {
            self.selected_category_index = None;
        } else {
            self.dice_set = rand::thread_rng().gen();
            self.roll_count = 1;
            self.select_next_available_category();
        }
    }

    fn on_r_key(&mut self) {
        if self.roll_count >= MAX_ROLLS || self.dice_selected.iter().all(|x| !x) {
            return;
        }

        for (dice, selected) in self.dice_set.0.iter_mut().zip(self.dice_selected.iter()) {
            if *selected {
                *dice = rand::thread_rng().gen();
            }
        }

        self.roll_count += 1;
        self.dice_selected = [false; DiceSet::NUM_DICE];
    }

    fn on_number_key(&mut self, c: char) {
        if self.roll_count >= MAX_ROLLS {
            return;
        }

        if let Some(d) = parse_dice_number(c) {
            self.dice_selected[d] ^= true;
        }
    }

    fn draw<W: io::Write>(&self, w: &mut W) -> Result<()> {
        let text = format!("Roll {} / {}", self.roll_count, MAX_ROLLS);
        crossterm::queue!(w, cursor::MoveTo(0, 0), style::Print(text))?;

        self.draw_content(w, 0, 2)?;

        const MESSAGE_Y: u16 = 3 + Dice::HEIGHT * DiceSet::NUM_DICE as u16;
        crossterm::queue!(w, cursor::MoveTo(0, MESSAGE_Y),)?;

        let mut text = vec!["Enter:       choose a scoring category"];
        if self.roll_count < MAX_ROLLS {
            text.push("Number keys: mark dice to be re-rolled");
            if self.dice_selected.iter().any(|x| *x) {
                text.push("R:           roll marked dice");
            }
        }

        for (line, y) in text
            .iter()
            .chain(std::iter::repeat(&""))
            .take(MAX_MESSAGE_LINES)
            .zip(3 + Dice::HEIGHT * DiceSet::NUM_DICE as u16..)
        {
            crossterm::queue!(
                w,
                cursor::MoveTo(0, y),
                terminal::Clear(terminal::ClearType::CurrentLine),
                style::Print(line)
            )?;
        }

        Ok(())
    }

    fn draw_content<W: io::Write>(&self, w: &mut W, x: u16, y: u16) -> Result<()> {
        let dice_num_x = x
            // left margin
            +2;
        let dice_x = dice_num_x
            // width of dice number
            + 1
            // margin
            + 2;

        for (i, dice) in self.dice_set.0.iter().enumerate() {
            crossterm::queue!(
                w,
                cursor::MoveTo(dice_num_x, y + Dice::HEIGHT / 2 + Dice::HEIGHT * i as u16),
                style::Print(i + 1)
            )?;

            if self.dice_selected[i] {
                dice.draw_styled(
                    w,
                    dice_x,
                    y + Dice::HEIGHT * i as u16,
                    style::ContentStyle::new()
                        .foreground(style::Color::Black)
                        .background(style::Color::Yellow),
                )?;
            } else {
                dice.draw(w, dice_x, y + Dice::HEIGHT * i as u16)?;
            }
        }

        let table_x = dice_x
            + Dice::WIDTH
            // margin
            + 4;
        self.draw_table(w, table_x, y)?;

        Ok(())
    }

    fn draw_table<W: io::Write>(&self, w: W, x: u16, y: u16) -> Result<()> {
        use table_printer::{TablePrinter, TablePrinterBuilder};

        let mut printer = TablePrinterBuilder::new(w)
            .x(x)
            .y(y)
            .width(24)
            .key_width(19)
            .value_width(3)
            .build();

        let print_section =
            |printer: &mut TablePrinter<W>, section: &[category::Category], offset| -> Result<()> {
                for (i, category) in section.iter().enumerate() {
                    let score = self.scoreboard.category_score(*category).unwrap_or(0)
                        + if self
                            .scoreboard
                            .category_is_available(*category, &self.dice_set)
                        {
                            self.dice_set.score(*category)
                        } else {
                            0
                        };

                    if self
                        .selected_category_index
                        .map(|selected| i + offset == selected)
                        .unwrap_or(false)
                    {
                        printer.selected_item(category, score)?;
                    } else if self
                        .scoreboard
                        .category_is_available(*category, &self.dice_set)
                    {
                        printer.highlighted_item(category, score)?;
                    } else {
                        printer.item(category, score)?;
                    }
                }

                Ok(())
            };

        printer.section("Upper Section")?.indent()?;
        print_section(&mut printer, &category::UPPER_SECTION, 0)?;
        printer
            .separator()?
            .item("Bonus if > 62", self.scoreboard.upper_section_bonus())?
            .item("Total", self.scoreboard.upper_total())?
            .unindent()?
            .blank()?;

        printer.section("Lower Section")?.indent()?;
        print_section(
            &mut printer,
            &category::LOWER_SECTION,
            category::UPPER_SECTION.len(),
        )?;
        printer
            .separator()?
            .item("Total", self.scoreboard.lower_total())?
            .unindent()?
            .blank()?;

        printer.item("Grand Total", self.scoreboard.grand_total())?;

        Ok(())
    }
}

fn setup_terminal() -> Result<io::Stdout> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    crossterm::queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    Ok(stdout)
}

fn cleanup_terminal<W: io::Write>(mut w: W) -> Result<()> {
    crossterm::queue!(w, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn parse_dice_number(c: char) -> Option<usize> {
    if let Some(d) = c.to_digit(10) {
        let d = d as usize;
        if 0 < d && d <= DiceSet::NUM_DICE {
            return Some(d - 1);
        }
    }
    None
}
