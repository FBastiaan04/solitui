use std::io;

use crossterm::{event::{self, EnableMouseCapture, Event, KeyCode, MouseEventKind}, execute};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, symbols::{self, border}, text::{Span, ToSpan}, widgets::{Block, Borders, Paragraph, Widget}, DefaultTerminal, Frame};

#[derive(Debug, Clone, Copy)]
struct Card {
    suit: u8,
    number: u8,
    hidden: bool,
    selected: bool
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
        "♠",
        "♥",
        "♣",
        "♦",
    ];

    const DECK: [Self; 52] = {
        let mut d = [const { Card {
            suit: 0,
            number: 0,
            hidden: true,
            selected: false
        } }; 52];
        let mut i = 0;
        while i < 52 {
            d[i].number = i as u8 / 4;
            d[i].suit = i as u8 % 4;
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
            , match (self.color() != 0, self.selected) {
                (true, true) => Style::new().red().on_white(),
                (true, false) => Style::new().red(),
                (false, true) => Style::new().black().on_white(),
                (false, false) => Style::new().white()
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

    const BLOCK_EMPTY: Block<'static> = {
        Block::bordered()
            .border_set(border::DOUBLE)
    };
}

struct App {
    rows: [Column; 8],
    stock: Pile,
    discard: Pile,
    suit_piles: [Pile; 4],
    selected_pos: SelectedPos,
    exit: bool,
    debug: String
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum SelectedPos {
    None,
    Discard,
    SuitPile(usize),
    Column(usize, usize)
}

impl App {
    fn init() -> Self {
        let mut res = Self {
            rows: [const { Column(Vec::new()) }; 8],
            stock: Pile(Vec::new()),
            discard: Pile(Vec::new()),
            suit_piles: [const { Pile(Vec::new()) }; 4],
            selected_pos: SelectedPos::None,
            exit: false,
            debug: "DEBUG STRING".to_string()
        };

        let mut rng = thread_rng();
        
        let mut deck = Card::DECK.choose_multiple(&mut rng, 52).map(|c| *c);

        for i in 0..8 {
            res.rows[i] = Column(deck.by_ref().take(i + 1).collect());
            res.rows[i].0[i].hidden = false;
        }

        res.stock = Pile(deck.collect());

        res
    }

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
        let ev = event::read()?;
        // self.debug = format!("{:#?}", ev);
        match ev {
            Event::Key(ev) => {
                match ev.code {
                    KeyCode::Esc => {self.exit = true}
                    KeyCode::Char('c') => {self.selected_pos = SelectedPos::None}
                    KeyCode::Char('d') => {
                        if let Some(mut card) = self.stock.0.pop() {
                            card.hidden = false;
                            self.discard.0.push(card);
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(ev) => {
                if ev.kind != MouseEventKind::Up(event::MouseButton::Left) {
                    return Ok(());
                }

                // self.debug = format!("{:#?}", ev);

                let new_pos = self.get_selected_pos(ev.column as usize, ev.row as usize);
                
                self.handle_move(new_pos);
                self.selected_pos = new_pos;
            }
            _ => {}
        }
        Ok(())
    }

    fn get_selected_pos(&mut self, x: usize, y: usize) -> SelectedPos {
        match x {
            0..=39 => {
                let x = x as usize / 5;
                let col = &self.rows[x];
                let y = y as usize / 2;
                if col.0.len() == 0 {
                    return SelectedPos::Column(x, 0)
                }
                if y >= col.0.len() {
                    let y = col.0.len() - 1;
                    return SelectedPos::Column(x, y)
                }
                if col.0[y].hidden {
                    return SelectedPos::Column(x, 0)
                }
                SelectedPos::Column(x, y)
            }
            41..46 => {
                match y {
                    0..5 => {
                        if let Some(mut card) = self.stock.0.pop() {
                            card.hidden = false;
                            self.discard.0.push(card);
                            SelectedPos::Discard
                        } else {SelectedPos::None}
                    }
                    5..10 => {
                        if self.discard.0.len() == 0 {
                            return SelectedPos::None
                        }
                        SelectedPos::Discard
                    }
                    10..30 => {
                        SelectedPos::SuitPile(y / 5 - 2)
                    }
                    _ => {
                        SelectedPos::None
                    }
                }
            }
            _ => {SelectedPos::None}
        }
    }

    fn handle_move(&mut self, dest: SelectedPos) {
        let src = &self.selected_pos;

        self.debug = format!("{:#?} -> {:#?}", src, dest);
        
        match dest {
            SelectedPos::None | SelectedPos::Discard => {}
            SelectedPos::SuitPile(n) => {
                if src == &SelectedPos::Discard {
                    let card = self.discard.0.last().unwrap();
                    if !self.validate_suit(n, card) {
                        return;
                    }
                    self.suit_piles[n].0.push(self.discard.0.pop().unwrap());
                    return;
                }

                if let SelectedPos::Column(x, y) = src {
                    if self.rows[*x].0.len() == 0 || self.rows[*x].0.len() > *y + 1 {
                        // only allow one card
                        return;
                    }
                    self.debug = "Here1".to_string();
                    if !self.validate_suit(n, &self.rows[*x].0[*y]) {
                        return;
                    }
                    self.debug = "Here2".to_string();
                    self.suit_piles[n].0.push(self.rows[*x].0.pop().unwrap());

                    if let Some(card) = self.rows[*x].0.last_mut() {
                        card.hidden = false;
                    }
                    return;
                }
            }
            SelectedPos::Column(x, _) => {
                match src {
                    SelectedPos::None => {},
                    SelectedPos::Discard => {
                        let card = self.discard.0.last().unwrap();
                        if !self.validate_col(x, card) {
                            return;
                        }
                        self.rows[x].0.push(self.discard.0.pop().unwrap());
                        return;
                    },
                    SelectedPos::SuitPile(n) => {
                        let card = match self.suit_piles[*n].0.last() {
                            Some(card) => card,
                            None => return
                        };
                        if !self.validate_col(x, card) {
                            return;
                        }
                        self.rows[x].0.push(self.suit_piles[*n].0.pop().unwrap());
                        return;
                    },
                    SelectedPos::Column(sx, sy) => {
                        if *sx == x {
                            return;
                        }
                        if self.rows[*sx].0.len() == 0 {
                            return;
                        }
                        let card = &self.rows[*sx].0[*sy];
                        if !self.validate_col(x, card) {
                            return;
                        }
                        let tmp: Vec<Card> = self.rows[*sx].0.drain(sy..).collect();
                        self.rows[x].0.extend(tmp);

                        if let Some(card) = self.rows[*sx].0.last_mut() {
                            card.hidden = false;
                        }
                        return;
                    },
                }
            },
        }
    }

    fn validate_suit(&self, pile_n: usize, card: &Card) -> bool {
        if let Some(last) = self.suit_piles[pile_n].0.last() {
            last.suit == card.suit
        } else {
            true
        }
    }

    fn validate_col(&self, col_n: usize, card: &Card) -> bool {
        if let Some(last) = self.rows[col_n].0.last() {
            last.color() != card.color() &&
            last.number == card.number + 1
        } else {
            card.number == 12 // King
        }
    }
}

struct Column(Vec<Card>);

struct Pile(Vec<Card>);

impl Widget for &Column {
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

impl Widget for &Pile {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Rect::new(area.x, area.y, 5, 5);
        if let Some(top) = self.0.last() {
            Paragraph::new(top.to_span())
                .block(Card::BLOCK_SINGLE)
                .render(area, buf);
            return
        }
        Card::BLOCK_EMPTY.render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 46 {
            Span::raw("Too small")
                .render(area, buf);
            return;
        }

        let mut x = area.x;
        let mut y = area.y;

        // columns
        for row in &self.rows {
            row.render(Rect::new(
                x,
                y,
                5,
                20
            ), buf);
            x += 5;
        }

        x += 1;
        // stock
        self.stock.render(Rect::new(
            x,
            y,
            5,
            5
        ), buf);
        y += 5;

        // discard
        self.discard.render(Rect::new(
            x,
            y,
            5,
            4
        ), buf);
        y += 5;

        // suit piles
        for i in 0..4 {
            self.suit_piles[i].render(Rect::new(
                x,
                y,
                5,
                5
            ), buf);
            y += 5;
        }

        x += 5;

        Paragraph::new(self.debug.clone())
            .render(Rect::new(
                x,
                0,
                area.width - x,
                area.height
            ), buf)
    }
}

fn main() -> io::Result<()> {
    let mut app = App::init();
    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture).unwrap();
    let res = app.run(&mut terminal);
    ratatui::restore();
    res
}

