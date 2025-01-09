use crate::board::Position;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Drone,
    Queen,
}

impl Piece {
    pub fn validate_move(&self, src: &Position, dst: &Position) -> Result<(), &'static str> {
        match self {
            // Match different piece to check valid movement
            Piece::Pawn => {
                if (src.row as i32 - dst.row as i32).abs() != 1
                    || (src.col as i32 - dst.col as i32).abs() != 1
                {
                    return Err("Pawns must move exactly one square diagonally");
                }
            },
            Piece::Drone => {
                if (src.row as i32 - dst.row as i32).abs() == 0 {
                    if (src.col as i32 - dst.col as i32).abs() > 2 {
                        return Err("Drones may move only 1 or 2 squares orthogonally");
                    }
                } else if (src.col as i32 - dst.col as i32).abs() == 0 {
                    if (src.row as i32 - dst.row as i32).abs() > 2 {
                        return Err("Drones may move only 1 or 2 squares orthogonally 1");
                    }
                } else {
                    return Err("Drones may move only 1 or 2 squares orthogonally");
                }
            },
            Piece::Queen => {
                let dx = dst.row as i32 - src.row as i32;
                let dy = dst.col as i32 - src.col as i32;

                if !(dx == 0 || dy == 0 || dx.abs() == dy.abs()) {
                    return Err("Queens may only move along a straight line");
                }
            },
        }

        return Ok(());
    }

    pub fn promote(&self, other: Piece) -> Result<Piece, &'static str> {
        match self {
            Piece::Pawn => {
                match other {
                    Piece::Pawn => Ok(Piece::Pawn),
                    Piece::Drone => Ok(Piece::Queen),
                    Piece::Queen => Err("Cannot promote Queen."),
                }
            },
            Piece::Drone => {
                match other {
                    Piece::Pawn => Ok(Piece::Queen),
                    Piece::Drone => Err("Cannot promote Drone with a Drone"),
                    Piece::Queen => Err("Cannot promote Queen."),
                }
            },
            Piece::Queen => Err("Cannot promote Queen."),
        }
    }

    pub fn points(&self) -> u8 {
        match self {
            Piece::Pawn => 1,
            Piece::Drone => 2,
            Piece::Queen => 3,
        }
    }
}
