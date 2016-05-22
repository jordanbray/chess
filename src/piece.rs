/// Represent a chess piece as a very simple enum
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

/// How many piece types are there?
pub const NUM_PIECES: usize = 6;

/// An array representing each piece type, in order of ascending value.
pub const ALL_PIECES: [Piece; NUM_PIECES] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];

impl Piece {
    /// Convert the `Piece` to a `usize` for table lookups.
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

