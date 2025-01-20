use std::{io, mem::{self, MaybeUninit}};

use crossterm::event::{self, Event, KeyCode};
use rand::seq::SliceRandom;
use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, symbols::{self, border}, text::{Span, ToSpan}, widgets::{Block, Borders, Paragraph, Widget}, DefaultTerminal, Frame};

#[derive(Debug, Clone, Copy)]
struct Card {
    suit: u8,
    number: u8,
    hidden: bool
}

impl Card {
    const NUMBERS: [&'static str; 13] = [
        "A",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "10",
        "J",
        "Q",
        "K",
    ];

    const SUITS: [&'static str; 4] = [
        "♤",
        "♡",
        "♧",
        "◇",
    ];

    const DECK: [Self; 52] = {
        let mut d = [const { Card {
            suit: 0,
            number: 0,
            hidden: true,
        } }; 52];
        let mut i = 0;
        while i < 52 {
            let c = &mut d[i];
            c.number = i as u8 / 4;
            c.suit = i as u8 % 4;
            i += 1;
        }
        d
    };

    fn color(&self) -> u8 {
        self.suit % 2
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        if self.hidden {
            return String::new();
        }
        format!(
            "{}{}",
            Card::NUMBERS[self.number as usize],
            Card::SUITS[self.suit as usize]
        )
    }
}

impl ToSpan for Card {
    fn to_span(&self) -> Span<'_> {
        Span::styled(
            self.to_string()
            , if self.color() == 0 {
                Style::default()
            } else {
                Style::new().red()
            }
        )
    }
}

impl Card {
    const BLOCK_SINGLE: Block<'static> = {
       Block::bordered().border_set(border::ROUNDED)
    };

    const BLOCK_FIRST: Block<'static> = {
        Block::bordered()
            .border_set(border::ROUNDED)
            .borders(Borders::TOP.union(Borders::LEFT).union(Borders::RIGHT))
    };

    const BLOCK_MIDDLE: Block<'static> = {
        Block::bordered()
            .border_set(symbols::border::Set {
                bottom_left: symbols::line::ROUNDED.vertical_right,
                bottom_right: symbols::line::ROUNDED.vertical_left,
                top_left: symbols::line::ROUNDED.vertical_right,
                top_right: symbols::line::ROUNDED.vertical_left,
                ..symbols::border::ROUNDED
            })
            .borders(Borders::TOP.union(Borders::LEFT).union(Borders::RIGHT))
    };

    const BLOCK_LAST: Block<'static> = {
        Block::bordered()
            .border_set(symbols::border::Set {
                top_left: symbols::line::ROUNDED.vertical_right,
                top_right: symbols::line::ROUNDED.vertical_left,
                ..symbols::border::ROUNDED
            })
    };
}

#[derive(Default)]
struct App {
    rows: [Vec<Card>; 8],
    stock: Vec<Card>,
    discard: Vec<Card>,
    suit_piles: [Vec<Card>; 4],
    exit: bool
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }
    
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(ev) if ev.code == KeyCode::Esc => {self.exit = true}
            _ => {}
        }
        Ok(())
    }
}

struct Column<'a>(&'a [Card]);

impl<'a> Widget for Column<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.0.len() == 0 {return}
        let x = area.x;
        let mut y = area.y;
        let first = &self.0[0];
        if self.0.len() == 1 {
            Paragraph::new(first.to_span())
                .block(Card::BLOCK_SINGLE)
                .render(Rect::new(x, y, 5, 5), buf);
            return
        }
        Paragraph::new(first.to_span())
            .block(Card::BLOCK_FIRST)
            .render(Rect::new(x, y, 5, 2), buf);
        y += 2;
        for i in 1..(self.0.len() - 1) {
            Paragraph::new(self.0[i].to_span())
                .block(Card::BLOCK_MIDDLE)
                .render(Rect::new(x, y, 5, 2), buf);
            y += 2;
        }

        Paragraph::new(self.0.last().unwrap().to_span())
            .block(Card::BLOCK_LAST)
            .render(Rect::new(x, y, 5, 5), buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, row) in self.rows.iter().enumerate() {
            let row = Column(row.as_slice());
            row.render(Rect::new(
                area.x + i as u16 * 5,
                area.y,
                5, 20
            ), buf);
        }
        /*
        for i in 1..3 {
        Paragraph::new(self.rows[1][0].to_span())
            .block(Card::BLOCK_FIRST)
            .render(Rect::new(area.x, area.y + 2 * i as u16, 5, 2), buf);
        }
        */
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    
    app.stock = Card::DECK.to_vec();
    let mut rng = rand::thread_rng();
    app.stock.shuffle(&mut rng);

    for i in 0..8 {
        let row = &mut app.rows[i];
        for _ in 0..i {
            row.push(app.stock.pop().unwrap());
        }
        let mut last = app.stock.pop().unwrap();
        last.hidden = false;
        row.push(last);
    }
    
    let res = app.run(&mut terminal);
    ratatui::restore();
    res
}

