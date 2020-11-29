use crate::color::Color;
use std::fmt;
use serde::{Serialize, Deserialize};

/// Represent a chess piece as a very simple enum
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// How many piece types are there?
pub const NUM_PIECES: usize = 6;

/// An array representing each piece type, in order of ascending value.
pub const ALL_PIECES: [Piece; NUM_PIECES] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

/// How many ways can I promote?
pub const NUM_PROMOTION_PIECES: usize = 4;

/// What pieces can I promote to?
pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Knight, Piece::Rook, Piece::Bishop];

impl Piece {
    /// Convert the `Piece` to a `usize` for table lookups.
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert a piece with a color to a string.  White pieces are uppercase, black pieces are
    /// lowercase.
    ///
    /// ```
    /// use chess::{Piece, Color};
    ///
    /// assert_eq!(Piece::King.to_string(Color::White), "K");
    /// assert_eq!(Piece::Knight.to_string(Color::Black), "n");
    /// ```
    #[inline]
    pub fn to_string(&self, color: Color) -> String {
        let piece = format!("{}", self);
        if color == Color::White {
            piece.to_uppercase()
        } else {
            piece
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Piece::Pawn => "p",
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Rook => "r",
                Piece::Queen => "q",
                Piece::King => "k",
            }
        )
    }
}
