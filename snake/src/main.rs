use rand::seq::SliceRandom;
use rustvision::vec::Vec2;
use std::{thread, time, io};
use std::collections::VecDeque;
use std::io::*;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Clone, PartialEq)]
enum BoardField {
    Empty,
    Fruit,
    SnakeHead,
    SnakeTail
}

struct GameBoard {
    board: Vec<Vec<BoardField>>,
    snake: VecDeque<Vec2<u8>>,
    size: Vec2<u8>
}

impl GameBoard {
    fn new(size: Vec2<u8>) -> GameBoard {

        assert!(size.y >= 2);
        assert!(size.x >= 2);
        let mut game_board = GameBoard {
            board: vec![vec![BoardField::Empty; size.x as usize]; size.y as usize],
            snake: VecDeque::from([Vec2 {x: 0u8, y: 0u8}]),
            size: Vec2{x: size.x, y: size.y}
        };
        
        game_board.board[0][0] = BoardField::SnakeHead;
        game_board.place_new_fruit();

        game_board
    }

    fn place_new_fruit(&mut self) -> bool {
        let empty_fields: Vec<Vec2<u8>> = self.board.iter()
            .flatten()
            .enumerate()
            .filter(|(_, field)| **field == BoardField::Empty)
            .map(|(index, _)| Vec2 {x: index as u8 % self.size.x, y: index as u8 / self.size.x })
            .collect();

        match empty_fields.choose(&mut rand::thread_rng()){
            Some(fruit_pos) => { 
                self.board[fruit_pos.y as usize][fruit_pos.x as usize] = BoardField::Fruit; 
                true 
            }
            None => false
        }
    }

    fn update(&mut self, direction: &Direction) -> Option<u16> {
        let head_pos = self.snake.get(0).expect("GameBoard is not initialized").clone();
        let tail_end_pos = self.snake.get(self.snake.len() - 1).unwrap();
        let next_head_pos = match direction {
            Direction::Up if head_pos.y == 0 => Vec2 {x: head_pos.x, y: self.size.y - 1 },
            Direction::Up => head_pos - Vec2 {x: 0u8, y: 1u8},
            Direction::Down if head_pos.y >= self.size.y - 1 => Vec2 {x: head_pos.x, y: 0u8},
            Direction::Down => head_pos + Vec2 {x: 0u8, y: 1u8},
            Direction::Left if head_pos.x == 0 => Vec2 {x: self.size.x - 1, y: head_pos.y },
            Direction::Left => head_pos - Vec2 {x: 1u8, y: 0u8},
            Direction::Right if head_pos.x >= self.size.x - 1 => Vec2 {x: 0u8, y: head_pos.y},
            Direction::Right => head_pos + Vec2 {x: 1u8, y: 0u8},
        };

        self.board[head_pos.y as usize][head_pos.x as usize] = BoardField::SnakeTail;
        
        if self.board[next_head_pos.y as usize][next_head_pos.x as usize] == BoardField::Fruit {
            if self.place_new_fruit() == false {
                return Some(self.snake.len() as u16)
            }
        }
        else {
            self.board[tail_end_pos.y as usize][tail_end_pos.x as usize] = BoardField::Empty;
            self.snake.pop_back();
        }
        
        if self.board[next_head_pos.y as usize][next_head_pos.x as usize] == BoardField::SnakeTail {
            return Some(self.snake.len() as u16);
        }

        self.board[next_head_pos.y as usize][next_head_pos.x as usize] = BoardField::SnakeHead;
        
        self.snake.push_front(next_head_pos);

        None
    }
}

impl std::fmt::Display for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        output.reserve((self.size.x + 2) as usize * self.size.y as usize);
        for row in &self.board {
            for field in row {
                match field {
                    BoardField::Empty => output.push('.'),
                    BoardField::SnakeTail => output.push('o'),
                    BoardField::SnakeHead => output.push('O'),
                    BoardField::Fruit => output.push('@'),
                }
            }
            output.push('\r');
            output.push('\n');
        }
        write!(f, "{}", output)
    }
}

fn read_from_terminal(tx: Sender<Direction>) {
    
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    for c in stdin.keys() {
        let direction = match c.unwrap() {
            Key::Char('w') => Direction::Up,
            Key::Char('a') => Direction::Left,
            Key::Char('s') => Direction::Down,
            Key::Char('d') => Direction::Right,
            _ => continue,
        };
        stdout.flush().unwrap();

        match tx.send(direction) {
            Ok(_) => (),
            Err(err) => print!("Cannot send data, error: {}", err)
        }
    }
}

fn show_tutorial() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "To change snake movement direction use keys 'w', 'a', 's' and 'd'.\r\n\
                    Try to collect as many fruits as possible '@'.\r\n\
                    Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    show_tutorial();

    let (tx, rx): (Sender<Direction>, Receiver<Direction>) = mpsc::channel();
    thread::spawn(move || {
        read_from_terminal(tx);
    });

    let mut game_board = GameBoard::new(Vec2 {x: 16u8, y: 16u8});
    let mut last_dir = Direction::Right;
    
    print!("{}", game_board);
    loop {
        thread::sleep(time::Duration::from_millis(100));
        
        match rx.try_recv() {
            Ok(direction) => last_dir = direction,
            Err(_) => ()
        } 
        
        match game_board.update(&last_dir) {
            Some(n) => { print!("Game ended, your score: {}\n", n); break; },
            None => ()
        }

        print!("{}[2J", 27 as char);
        print!("{}", game_board);
        io::stdout().flush().unwrap();
    }
}
