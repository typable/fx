use console::Color;
use console::Term;

use crate::color;
use crate::pad;
use crate::Result;

pub trait Widget {
    fn draw(&self, term: &Term) -> Result<()>;
}

type Pos = (usize, usize);

pub struct Window {
    pub pos: Pos,
    pub width: usize,
    pub height: usize,
}

impl Widget for Window {
    fn draw(&self, term: &Term) -> Result<()> {
        let (x, y) = self.pos;
        term.move_cursor_to(x, y)?;
        term.write_str(&color!(
            format!("╭{:─<width$}╮", "", width = self.width - 2),
            Color::Color256(240),
        ))?;
        for i in 1..self.height - 1 {
            term.move_cursor_to(x, y + i)?;
            term.write_str(&color!(
                if i == 2 {
                    format!("├{:─<width$}┤", "", width = self.width - 2)
                } else if i == self.height - 3 {
                    format!("├─────────┬{:─<width$}┤", "", width = self.width - 12)
                } else if i == self.height - 2 {
                    format!("│         │{: <width$}│", "", width = self.width - 12)
                } else {
                    format!("│{: <width$}│", "", width = self.width - 2)
                },
                Color::Color256(240),
            ))?;
        }
        term.move_cursor_to(x, self.height - 1)?;
        term.write_str(&color!(
            format!("╰─────────┴{:─<width$}╯", "", width = self.width - 12),
            Color::Color256(240),
        ))?;
        Ok(())
    }
}

pub struct List {
    pub pos: Pos,
    pub width: usize,
    pub height: usize,
    pub index: usize,
    pub items: Vec<Text>,
}

impl List {
    pub fn init(&mut self) {
        let (x, y) = self.pos;
        for i in 0..self.items.len() {
            self.items[i].pos = (x + 2, y + i);
            self.items[i].width = self.width - 2;
        }
    }
}

impl Widget for List {
    fn draw(&self, term: &Term) -> Result<()> {
        let (x, y) = self.pos;
        for i in 0..self.height {
            term.move_cursor_to(x, y + i)?;
            term.write_str(&format!("{: <width$}", "", width = self.width))?;
            if i < self.items.len() {
                self.items[i].draw(term)?;
            }
        }
        term.move_cursor_to(x, y + self.index)?;
        term.write_str("▶")?;
        Ok(())
    }
}

pub struct Text {
    pub pos: Pos,
    pub width: usize,
    pub text: String,
}

impl Widget for Text {
    fn draw(&self, term: &Term) -> Result<()> {
        let (x, y) = self.pos;
        term.move_cursor_to(x, y)?;
        term.write_str(&pad!(self.text, self.width, self.width))?;
        Ok(())
    }
}
