use regex::Regex;
use std::io;
use std::io::Write;

#[derive(Copy, Clone, PartialEq, Eq)]
struct Position {
    row: usize,
    col: usize,
}

#[derive(Copy, Clone)]
struct Move {
    src: Position,
    dst: Position,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Piece {
    Pawn = 1,
    Drone = 2,
    Queen = 3,
}

struct MartianChess {
    board: Vec<Option<Piece>>,
    turn: u32,
    score: (u8, u8),
    prev_move: Option<Move>,
    clock: i32,
}

impl MartianChess {
    fn new() -> Self {
        let mut game = MartianChess {
            board: vec![None; 32],
            turn: 0,
            score: (0, 0),
            prev_move: None,
            clock: -1,
        };

        for i in [6, 9, 10, 21, 22, 25] {
            game.board[i] = Some(Piece::Pawn);
        }
        for i in [2, 5, 8, 23, 26, 29] {
            game.board[i] = Some(Piece::Drone);
        }
        for i in [0, 1, 4, 27, 30, 31] {
            game.board[i] = Some(Piece::Queen);
        }

        return game;
    }

    fn position_to_index(&self, p: Position) -> usize {
        4 * p.row + p.col
    }

    fn get_piece(&self, Position { row, col }: Position) -> Option<Piece> {
        self.board[4 * row + col]
    }

    fn get_piece_mut(&mut self, Position { row, col }: Position) -> &mut Option<Piece> {
        &mut self.board[4 * row + col]
    }

    fn get_zone(&self, Position { row, .. }: Position) -> u32 {
        if row <= 3 {
            return 0_u32;
        } else {
            return 1_u32;
        }
    }

    fn move_piece(&mut self, m: &Move) -> Result<(), String> {
        if m.src.row == 32 {
            // Check if starting clock
            if self.clock > 0 {
                return Err("Clock already started".to_string());
            } else {
                self.clock = 7;
                return Ok(());
            }
        }

        if self.get_piece(m.src).is_none() {
            // Make sure a piece exists at src
            return Err(format!(
                "No piece at position {}{}",
                char::from((u32::from('A') + m.src.col as u32) as u8),
                m.src.row + 1
            ));
        }

        if self.turn % 2 != self.get_zone(m.src) {
            // Make sure the piece is in your zone of control
            return Err(format!(
                "Position {}{} is not in Player {} zone of control",
                char::from((u32::from('A') + m.src.col as u32) as u8),
                m.src.row + 1,
                (self.turn % 2) + 1
            ));
        }
        
        if m.src == m.dst {
            return Err(format!("Must move piece to location different from starting location"));
        }

        if let Some(p) = self.prev_move {
            if m.src == p.dst && m.dst == p.src {
                return Err(format!("Cannot use your move to undo previous move"));
            }
        }

        self.validate_move(&m)?;
        self.collision_check(&m)?;
        if self.turn % 2 != self.get_zone(m.dst) {
            // Moving
            if self.clock > 0 && !self.get_piece(m.dst).is_none() {
                self.clock = 7;
            }

            if self.turn % 2 == 0 {
                if let Some(p) = self.get_piece(m.dst) {
                    self.score.0 += p as u8;
                }
            } else {
                if let Some(p) = self.get_piece(m.dst) {
                    self.score.1 += p as u8;
                }
            }
            *self.get_piece_mut(m.dst) = self.get_piece(m.src);
        } else {
            self.can_promote(&m)?;
            match self.get_piece(m.src) {
                Some(Piece::Pawn) => {
                    match self.get_piece(m.dst) {
                        Some(Piece::Pawn) => *self.get_piece_mut(m.dst) = Some(Piece::Drone),
                        Some(Piece::Drone) => *self.get_piece_mut(m.dst) = Some(Piece::Queen),
                        Some(Piece::Queen) => panic!("Tried to promote Piece::Queen"),
                        None => *self.get_piece_mut(m.dst) = Some(Piece::Pawn),
                    };
                }
                Some(Piece::Drone) => {
                    match self.get_piece(m.dst) {
                        Some(Piece::Pawn) => *self.get_piece_mut(m.dst) = Some(Piece::Queen),
                        Some(Piece::Drone) => {
                            panic!("Tried to promote Piece::Drone with Piece::Drone")
                        }
                        Some(Piece::Queen) => panic!("Tried to promote Piece::Queen"),
                        None => *self.get_piece_mut(m.dst) = Some(Piece::Drone),
                    };
                }
                Some(Piece::Queen) => {
                    match self.get_piece(m.dst) {
                        Some(Piece::Pawn) => panic!("Tried to promote Piece::Queen"),
                        Some(Piece::Drone) => panic!("Tried to promote Piece::Queen"),
                        Some(Piece::Queen) => panic!("Tried to promote Piece::Queen"),
                        None => *self.get_piece_mut(m.dst) = Some(Piece::Queen),
                    };
                }
                None => (),
            };
        }
        *self.get_piece_mut(m.src) = None;
        self.turn += 1;

        self.prev_move = Some(*m);
        return Ok(());
    }

    fn validate_move(&self, m: &Move) -> Result<(), String> {
        match self.get_piece(m.src) {
            // Match different piece to check valid movement
            Some(Piece::Pawn) => {
                if (m.src.row as i32 - m.dst.row as i32).abs() != 1
                    || (m.src.col as i32 - m.dst.col as i32).abs() != 1
                {
                    return Err(format!("Pawns must move exactly one square diagonally"));
                }
            }
            Some(Piece::Drone) => {
                if (m.src.row as i32 - m.dst.row as i32).abs() == 0 {
                    if (m.src.col as i32 - m.dst.col as i32).abs() > 2 {
                        return Err(format!("Drones may move only 1 or 2 squares orthogonally"));
                    }
                } else if (m.src.col as i32 - m.dst.col as i32).abs() == 0 {
                    if (m.src.row as i32 - m.dst.row as i32).abs() > 2 {
                        return Err(format!("Drones may move only 1 or 2 squares orthogonally 1"));
                    }
                } else {
                    return Err(format!("Drones may move only 1 or 2 squares orthogonally"));
                }
            }
            Some(Piece::Queen) => {
                let dx = m.dst.row as i32 - m.src.row as i32;
                let dy = m.dst.col as i32 - m.src.col as i32;

                if !(dx == 0 || dy == 0 || dx.abs() == dy.abs()) {
                    return Err(format!("Queens may only move along a straight line"));
                }
            }
            _ => (), // Piece values can only by in range [1,3]
        }

        return Ok(());
    }

    fn collision_check(&self, m: &Move) -> Result<(), String> {
        let dx = (m.dst.row as i32 - m.src.row as i32).signum();
        let dy = (m.dst.col as i32 - m.src.col as i32).signum();

        let mut n = Position {
            row: (m.src.row as i32 + dx) as usize,
            col: (m.src.col as i32 + dy) as usize,
        };

        while n != m.dst {
            if !self.get_piece(n).is_none() {
                return Err(format!(
                    "Move blocked by {}{}",
                    char::from((u32::from('A') + n.col as u32) as u8),
                    n.row + 1
                ));
            } else {
                n.row = (n.row as i32 + dx) as usize;
                n.col = (n.col as i32 + dy) as usize;
            }
        }

        return Ok(());
    }

    fn can_promote(&self, m: &Move) -> Result<(), String> {
        if self.get_piece(m.src).unwrap() == Piece::Queen
            || self.get_piece(m.dst).unwrap() == Piece::Queen
        {
            return Err(format!("Cannot promote queen"));
        }

        let new_piece = match self.get_piece(m.src) {
            Some(Piece::Pawn) => match self.get_piece(m.dst) {
                Some(Piece::Pawn) => Piece::Drone,
                Some(Piece::Drone) => Piece::Queen,
                Some(Piece::Queen) => {
                    return Err(format!("Tried to promote Queen"));
                }
                None => {
                    return Err(format!("Tried to promote None piece"));
                }
            },
            Some(Piece::Drone) => match self.get_piece(m.dst) {
                Some(Piece::Pawn) => Piece::Queen,
                Some(Piece::Drone) => {
                    return Err(format!("Tried to promote Drone with Drone"));
                }
                Some(Piece::Queen) => {
                    return Err(format!("Tried to promote Queen"));
                }
                None => {
                    return Err(format!("Tried to promote None piece"));
                }
            },
            Some(Piece::Queen) => {
                return Err(format!("Tried to promote Queen"));
            }
            None => panic!("Well this shouldn't be possible..."),
        };

        let mut cnt = 0;
        for i in 0..32 {
            if let Some(p) = self.board[i] {
                if p == new_piece {
                    cnt += 1;
                }
            }
        }

        if cnt > 5 {
            return Err(format!(
                "Cannot have more than 6 {:?} on the board",
                new_piece
            ));
        }

        let mut cnt = 0;
        if self.turn % 2 == 0 {
            for i in 0..16 {
                if let Some(p) = self.board[i] {
                    if p == new_piece {
                        cnt += 1;
                    }
                }
            }
        } else {
            for i in 16..32 {
                if let Some(p) = self.board[i] {
                    if p == new_piece {
                        cnt += 1;
                    }
                }
            }
        }

        if cnt > 1 {
            return Err(format!(
                "Cannot promote if a {:?} is in your zone",
                new_piece
            ));
        }

        return Ok(());
    }

    fn can_play(&self) -> bool {
        let mut t1 = 0;
        for i in 0..16 {
            match self.board[i] {
                Some(_) => t1 += 1,
                None => (),
            }
        }

        let mut t2 = 0;
        for i in 16..32 {
            match self.board[i] {
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
                    match self.board[i * 4 + j] {
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

fn main() {
    let mut game = MartianChess::new();
    game.play();
}
