use std::error::Error;
use std::io;
use std::io::Write;

use crate::pieces::Piece;
use crate::board::Board;
use crate::board::Position;

use regex::Regex;

pub mod pieces;
pub mod board;

pub struct Config {
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        Ok(Config {})
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut game = MartianChess::new(config);
    game.play();

    Ok(())
}

#[derive(Copy, Clone)]
struct Move {
    src: Position,
    dst: Position,
}

struct MartianChess {
    board: Board,
    turn: u32,
    score: (u8, u8),
    prev_move: Option<Move>,
    clock: i32,
}

impl MartianChess {
    fn new(config: Config) -> Self {
        let mut game = MartianChess {
            board: Board::new(8, 4),
            turn: 0,
            score: (0, 0),
            prev_move: None,
            clock: -1,
        };

        *game.board.get_piece_mut(Position{row: 1, col: 2}) = Some(Piece::Pawn);
        *game.board.get_piece_mut(Position{row: 2, col: 1}) = Some(Piece::Pawn);
        *game.board.get_piece_mut(Position{row: 2, col: 2}) = Some(Piece::Pawn);
        *game.board.get_piece_mut(Position{row: 5, col: 1}) = Some(Piece::Pawn);
        *game.board.get_piece_mut(Position{row: 5, col: 2}) = Some(Piece::Pawn);
        *game.board.get_piece_mut(Position{row: 6, col: 1}) = Some(Piece::Pawn);

        *game.board.get_piece_mut(Position{row: 0, col: 2}) = Some(Piece::Drone);
        *game.board.get_piece_mut(Position{row: 1, col: 1}) = Some(Piece::Drone);
        *game.board.get_piece_mut(Position{row: 2, col: 0}) = Some(Piece::Drone);
        *game.board.get_piece_mut(Position{row: 5, col: 3}) = Some(Piece::Drone);
        *game.board.get_piece_mut(Position{row: 6, col: 2}) = Some(Piece::Drone);
        *game.board.get_piece_mut(Position{row: 7, col: 1}) = Some(Piece::Drone);
        
        *game.board.get_piece_mut(Position{row: 0, col: 0}) = Some(Piece::Queen);
        *game.board.get_piece_mut(Position{row: 0, col: 1}) = Some(Piece::Queen);
        *game.board.get_piece_mut(Position{row: 1, col: 0}) = Some(Piece::Queen);
        *game.board.get_piece_mut(Position{row: 6, col: 3}) = Some(Piece::Queen);
        *game.board.get_piece_mut(Position{row: 7, col: 2}) = Some(Piece::Queen);
        *game.board.get_piece_mut(Position{row: 7, col: 3}) = Some(Piece::Queen);

        return game;
    }


    fn get_zone(&self, Position { row, .. }: Position) -> u32 {
        if row <= 3 {
            return 0_u32;
        } else {
            return 1_u32;
        }
    }

    fn move_piece(&mut self, m: &Move) -> Result<(), &'static str> {
        if m.src.row == 32 {
            // Check if starting clock
            if self.clock > 0 {
                return Err("Clock already started");
            } else {
                self.clock = 8;
                return Ok(());
            }
        }

        let Some(src_piece) = self.board.get_piece(m.src) else {
            // Make sure a piece exists at src
            return Err("No piece at source position");
        };

        if self.turn % 2 != self.get_zone(m.src) {
            // Make sure the piece is in your zone of control
            return Err("Source Position is not in Player zone of control");
        }
        
        if m.src == m.dst {
            return Err("Must move piece to location different from starting location");
        }

        if let Some(p) = self.prev_move {
            if m.src == p.dst && m.dst == p.src {
                return Err("Cannot use your move to undo previous move");
            }
        }

        src_piece.validate_move(&m.src, &m.dst)?;
        self.collision_check(&m)?;
        if self.turn % 2 != self.get_zone(m.dst) {
            // Moving
            if self.clock > 0 && !self.board.get_piece(m.dst).is_none() {
                self.clock = 8;
            }

            if self.turn % 2 == 0 {
                if let Some(p) = self.board.get_piece(m.dst) {
                    self.score.0 += p.points();
                }
            } else {
                if let Some(p) = self.board.get_piece(m.dst) {
                    self.score.1 += p.points();
                }
            }
            *self.board.get_piece_mut(m.dst) = self.board.get_piece(m.src);
        } else {
            if self.board.get_piece(m.dst).is_none() {
                *self.board.get_piece_mut(m.dst) = self.board.get_piece(m.src);
            } else {
                let new_piece = self.can_promote(&m)?;
                *self.board.get_piece_mut(m.dst) = Some(new_piece);
            }
        }
        *self.board.get_piece_mut(m.src) = None;
        self.turn += 1;

        self.prev_move = Some(*m);
        return Ok(());
    }

    fn collision_check(&self, m: &Move) -> Result<(), &'static str> {
        let dx = (m.dst.row as i32 - m.src.row as i32).signum();
        let dy = (m.dst.col as i32 - m.src.col as i32).signum();

        let mut n = Position {
            row: (m.src.row as i32 + dx) as usize,
            col: (m.src.col as i32 + dy) as usize,
        };

        while n != m.dst {
            if !self.board.get_piece(n).is_none() {
                return Err("Move blocked by");
            } else {
                n.row = (n.row as i32 + dx) as usize;
                n.col = (n.col as i32 + dy) as usize;
            }
        }

        return Ok(());
    }

    fn can_promote(&self, m: &Move) -> Result<Piece, &'static str> {
        let Some(src_piece) = self.board.get_piece(m.src) else {
            return Err("No source piece to promote");
        };
        
        let Some(dst_piece) = self.board.get_piece(m.dst) else {
            return Err("No destination piece to promote");
        };

        let new_piece = src_piece.promote(dst_piece)?;

        let mut cnt = 0;
        for i in 0..32 {
            if let Some(p) = self.board.data[i] {
                if p == new_piece {
                    cnt += 1;
                }
            }
        }

        if cnt > 5 {
            return Err("Cannot have more than 6 of one piece on the board");
        }

        let mut cnt = 0;
        if self.turn % 2 == 0 {
            for i in 0..16 {
                if let Some(p) = self.board.data[i] {
                    if p == new_piece {
                        cnt += 1;
                    }
                }
            }
        } else {
            for i in 16..32 {
                if let Some(p) = self.board.data[i] {
                    if p == new_piece {
                        cnt += 1;
                    }
                }
            }
        }

        if cnt > 1 {
            return Err("Cannot promote if piece type already in your zone");
        }

        return Ok(new_piece);
    }

    fn can_play(&self) -> bool {
        let mut t1 = 0;
        for i in 0..16 {
            match self.board.data[i] {
                Some(_) => t1 += 1,
                None => (),
            }
        }

        let mut t2 = 0;
        for i in 16..32 {
            match self.board.data[i] {
                Some(_) => t2 += 1,
                None => (),
            }
        }

        if t1 == 0 {
            println!("Game Over: No pieces in zone 1");
            return false;
        }

        if t2 == 0 {
            println!("Game Over: No pieces in zone 2");
            return false;
        }

        if self.clock == 0 {
            println!("Game Over: Clock has reached 0");
            return false;
        }

        return true;
    }

    fn print(&self) {
        println!(" |ABCD");
        println!("-+----");

        for i in 0..8 {
            print!("{}:", i + 1);
            for j in 0..4 {
                print!(
                    "{}",
                    match self.board.get_piece(Position{row: i, col: j}) {
                        None => '.',
                        Some(Piece::Pawn) => 'P',
                        Some(Piece::Drone) => 'D',
                        Some(Piece::Queen) => 'Q',
                    }
                );
            }
            if i == 3 {
                println!("\n-+----");
            } else {
                println!("");
            }
        }
    }

    fn get_move(&self) -> Option<Move> {
        if self.clock > -1 {
            print!("Player {} ({})> ", (self.turn % 2) + 1, self.clock);
        } else {
            print!("Player {}> ", (self.turn % 2) + 1);
        }
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("");

        if self.clock == -1 {
            let re = Regex::new("\\s*clk\\s*").unwrap();
            if re.is_match(&input) {
                return Some(Move {
                    src: Position { row: 32, col: 32 },
                    dst: Position { row: 32, col: 32 },
                });
            }
        }

        let re = Regex::new("([a-dA-D])([1-8])([a-dA-D])([1-8])").unwrap();
        match re.captures(&input) {
            Some(caps) => Some(Move {
                src: Position {
                    row: caps[2].parse::<usize>().unwrap() - 1,
                    col: ((u32::from(caps[1].chars().nth(0).unwrap()) | 32) - u32::from('a'))
                        as usize,
                },
                dst: Position {
                    row: caps[4].parse::<usize>().unwrap() - 1,
                    col: ((u32::from(caps[3].chars().nth(0).unwrap()) | 32) - u32::from('a'))
                        as usize,
                },
            }),
            None => None,
        }
    }

    fn play(&mut self) {
        while self.can_play() {
            self.print();
            if let Some(m) = self.get_move() {
                match self.move_piece(&m) {
                    Ok(_) => {
                        if self.clock > 0 {
                            self.clock -= 1;
                        }
                    }
                    Err(s) => println!("{}", s),
                };
                println!("");
            } else {
                println!("Failed to parse move");
            }
        }
        println!("Player 1: {}, Player 2: {}", self.score.0, self.score.1);
        if self.score.0 > self.score.1 {
            println!("Player 1 wins!!!");
        } else if self.score.0 < self.score.1 {
            println!("Player 2 wins!!!");
        } else {
            if self.turn % 2 == 0 {
                println!("Player 2 wins!!!");
            } else {
                println!("Player 1 wins!!!");
            }
        }
    }
}
