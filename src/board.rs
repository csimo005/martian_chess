use crate::pieces::Piece;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Board {
    pub data: Vec<Option<Piece>>,
    pub rows: usize,
    pub cols: usize,
}

impl Board {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self{rows, cols, data: vec![None; rows*cols]}
    }

    pub fn position_to_index(&self, p: Position) -> usize {
        self.cols * p.row + p.col
    }

    pub fn get_piece(&self, Position { row, col }: Position) -> Option<Piece> {
        self.data[self.cols * row + col]
    }

    pub fn get_piece_mut(&mut self, Position { row, col }: Position) -> &mut Option<Piece> {
        &mut self.data[self.cols * row + col]
    }
}
