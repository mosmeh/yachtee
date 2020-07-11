use anyhow::Result;
use crossterm::style::{Color, ContentStyle, StyledContent};
use crossterm::{cursor, style};
use std::fmt;
use std::io;

const SELECT_SYMBOL: &str = "> ";
const HIGHLIGHT_COLOR: Color = Color::Yellow;

pub struct TablePrinter<W: io::Write> {
    writer: W,
    x: u16,
    y: u16,
    width: usize,
    key_width: usize,
    value_width: usize,
    indent: u16,
}

impl<W: io::Write> TablePrinter<W> {
    pub fn item<DK, DV>(&mut self, key: DK, value: DV) -> Result<&mut Self>
    where
        DK: fmt::Display,
        DV: fmt::Display,
    {
        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.x, self.y),
            style::Print(self.format(key, value, ""))
        )?;

        self.y += 1;
        Ok(self)
    }

    pub fn highlighted_item<DK, DV>(&mut self, key: DK, value: DV) -> Result<&mut Self>
    where
        DK: fmt::Display,
        DV: fmt::Display,
    {
        let style = ContentStyle::new().foreground(HIGHLIGHT_COLOR);
        let styled = StyledContent::new(style, self.format(key, value, ""));

        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.x, self.y),
            style::PrintStyledContent(styled)
        )?;

        self.y += 1;
        Ok(self)
    }

    pub fn selected_item<DK, DV>(&mut self, key: DK, value: DV) -> Result<&mut Self>
    where
        DK: fmt::Display,
        DV: fmt::Display,
    {
        let style = ContentStyle::new()
            .foreground(Color::Black)
            .background(HIGHLIGHT_COLOR);
        let styled = StyledContent::new(style, self.format(key, value, SELECT_SYMBOL));

        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.x, self.y),
            style::PrintStyledContent(styled)
        )?;

        self.y += 1;
        Ok(self)
    }

    pub fn section<D>(&mut self, text: D) -> Result<&mut Self>
    where
        D: fmt::Display + Clone,
    {
        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.x, self.y),
            style::Print(text)
        )?;

        self.y += 1;
        Ok(self)
    }

    pub fn blank(&mut self) -> Result<&mut Self> {
        self.y += 1;
        Ok(self)
    }

    pub fn separator(&mut self) -> Result<&mut Self> {
        let separator = "─".repeat(self.width);

        crossterm::queue!(
            self.writer,
            cursor::MoveTo(self.x, self.y),
            style::Print(separator)
        )?;

        self.y += 1;
        Ok(self)
    }

    pub fn indent(&mut self) -> Result<&mut Self> {
        self.indent += SELECT_SYMBOL.len() as u16;
        Ok(self)
    }

    pub fn unindent(&mut self) -> Result<&mut Self> {
        self.indent -= SELECT_SYMBOL.len() as u16;
        Ok(self)
    }

    fn format<DK, DV>(&self, key: DK, value: DV, symbol: &str) -> String
    where
        DK: fmt::Display,
        DV: fmt::Display,
    {
        let left_margin = " ".repeat(self.indent as usize - symbol.len());
        let content = format!(
            "{:key_width$}{:>value_width$}",
            format!("{}", key),
            format!("{}", value),
            key_width = self.key_width - self.indent as usize,
            value_width = self.value_width
        );
        let right_margin = " ".repeat(self.width - self.key_width - self.value_width);

        left_margin + symbol + &content + &right_margin
    }
}

pub struct TablePrinterBuilder<W: io::Write> {
    writer: W,
    x: u16,
    y: u16,
    width: usize,
    key_width: usize,
    value_width: usize,
}

impl<W: io::Write> TablePrinterBuilder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            x: 0,
            y: 0,
            width: 10,
            key_width: 5,
            value_width: 5,
        }
    }

    pub fn build(self) -> TablePrinter<W> {
        TablePrinter {
            writer: self.writer,
            x: self.x,
            y: self.y,
            width: self.width,
            key_width: self.key_width,
            value_width: self.value_width,
            indent: 0,
        }
    }

    pub fn x(mut self, x: u16) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: u16) -> Self {
        self.y = y;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn key_width(mut self, key_width: usize) -> Self {
        self.key_width = key_width;
        self
    }

    pub fn value_width(mut self, value_width: usize) -> Self {
        self.value_width = value_width;
        self
    }
}
