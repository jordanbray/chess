use std::fmt;

/// Represent a chess piece as a very simple enum
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone)]
#[allow(dead_code)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

/// How many piece types are there?
#[allow(dead_code)]
pub const NUM_PIECES: usize = 6;

/// An array representing each piece type, in order of ascending value.
#[allow(dead_code)]
pub const ALL_PIECES: [Piece; NUM_PIECES] = [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King];

/// How many ways can I promote?
#[allow(dead_code)]
pub const NUM_PROMOTION_PIECES: usize = 4;

/// What pieces can I promote to?
#[allow(dead_code)]
pub const PROMOTION_PIECES: [Piece; 4] = [ Piece::Queen, Piece::Knight, Piece::Rook, Piece::Bishop ];

impl Piece {
    /// Convert the `Piece` to a `usize` for table lookups.
    #[allow(dead_code)]
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

impl fmt::Display for Piece {
    #[allow(dead_code)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Piece::Pawn => "P",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Rook => "R",
            Piece::Queen => "Q",
            Piece::King => "K"
        })
    }
}
