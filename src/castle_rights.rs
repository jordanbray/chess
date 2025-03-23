use std::fmt;
use std::hint::unreachable_unchecked;

use crate::bitboard::{BitBoard, EMPTY};
use crate::color::Color;
use crate::file::File;
use crate::square::Square;

use crate::magic::{KINGSIDE_CASTLE_SQUARES, QUEENSIDE_CASTLE_SQUARES};

/// What castle rights does a particular player have?
#[repr(u8)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum CastleRights {
    NoRights = 0b00,
    Both = 0b11,
    KingSide = 0b01,
    QueenSide = 0b10,
}

/// How many different types of `CastleRights` are there?
pub const NUM_CASTLE_RIGHTS: usize = 4;

/// Enumerate all castle rights.
pub const ALL_CASTLE_RIGHTS: [CastleRights; NUM_CASTLE_RIGHTS] = [
    CastleRights::NoRights,
    CastleRights::KingSide,
    CastleRights::QueenSide,
    CastleRights::Both,
];

//? Could this be turned into a logic function? Or does this simply increase complexity...
/*
fn square_to_castle_rights(color: Color, square: Square) -> CastleRights {
    let rank = if color.into() { Rank::1 } else { Rank::8 };
    if square.get_rank() != rank.into_index() {
        CastleRights::NoRights
    } else {
        match square.get_file() {
            File::A => CastleRights::QueenSide,
            File::E => CastleRights::Both,
            File::H => CastleRights::KingSide,
            _ => CastleRights::None,
        }
    }
}
*/
const CASTLES_PER_SQUARE: [[u8; 64]; 2] = [
    [
        2, 0, 0, 0, 3, 0, 0, 1, // 1
        0, 0, 0, 0, 0, 0, 0, 0, // 2
        0, 0, 0, 0, 0, 0, 0, 0, // 3
        0, 0, 0, 0, 0, 0, 0, 0, // 4
        0, 0, 0, 0, 0, 0, 0, 0, // 5
        0, 0, 0, 0, 0, 0, 0, 0, // 6
        0, 0, 0, 0, 0, 0, 0, 0, // 7
        0, 0, 0, 0, 0, 0, 0, 0, // 8
    ], // white
    [
        0, 0, 0, 0, 0, 0, 0, 0, // 1
        0, 0, 0, 0, 0, 0, 0, 0, // 2
        0, 0, 0, 0, 0, 0, 0, 0, // 3
        0, 0, 0, 0, 0, 0, 0, 0, // 4
        0, 0, 0, 0, 0, 0, 0, 0, // 5
        0, 0, 0, 0, 0, 0, 0, 0, // 6
        0, 0, 0, 0, 0, 0, 0, 0, // 7
        2, 0, 0, 0, 3, 0, 0, 1, // 8
    ], // black
];

impl CastleRights {
    /// Can I castle kingside?
    pub fn has_kingside(&self) -> bool {
        // Self::Both == 3 -> 0b11 & 0b01 == 0b01 👍
        self.into_index() & 1 == 1
    }

    /// Can I castle queenside?
    pub fn has_queenside(&self) -> bool {
        // Self::Both == 3 -> 0b11 & 0b10 == 0b10 👍
        self.into_index() & 2 == 2
    }

    /// What rights does this square enable?
    pub fn square_to_castle_rights(color: Color, sq: Square) -> CastleRights {
        CastleRights::from_index(unsafe {
            *CASTLES_PER_SQUARE
                .get_unchecked(color.into_index())
                .get_unchecked(sq.into_index())
        } as usize)
    }

    /// What squares need to be empty to castle kingside?
    pub fn kingside_squares(&self, color: Color) -> BitBoard {
        unsafe { *KINGSIDE_CASTLE_SQUARES.get_unchecked(color.into_index()) }
    }

    /// What squares need to be empty to castle queenside?
    pub fn queenside_squares(&self, color: Color) -> BitBoard {
        unsafe { *QUEENSIDE_CASTLE_SQUARES.get_unchecked(color.into_index()) }
    }

    /// Remove castle rights, and return a new `CastleRights`.
    pub fn remove(&self, remove: CastleRights) -> CastleRights {
        CastleRights::from_index(self.into_index() & !remove.into_index())
    }

    /// Add some castle rights, and return a new `CastleRights`.
    pub fn add(&self, add: CastleRights) -> CastleRights {
        CastleRights::from_index(self.into_index() | add.into_index())
    }

    /// Convert `CastleRights` to `usize` for table lookups
    pub fn into_index(&self) -> usize {
        *self as usize
    }

    /// Convert this into a `&'static str` (for displaying)
    fn to_str(&self) -> &'static str {
        match *self {
            CastleRights::NoRights => "",
            CastleRights::KingSide => "k",
            CastleRights::QueenSide => "q",
            CastleRights::Both => "kq",
        }
    }

    /// Convert `usize` to `CastleRights`.
    pub fn from_index(i: usize) -> CastleRights {
        match i & 3 {
            0 => CastleRights::NoRights,
            1 => CastleRights::KingSide,
            2 => CastleRights::QueenSide,
            3 => CastleRights::Both,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    pub fn unmoved_rooks(&self, color: Color) -> BitBoard {
        let my_backrank = color.to_my_backrank();
        match *self {
            CastleRights::NoRights => EMPTY,
            CastleRights::KingSide => BitBoard::set(my_backrank, File::H),
            CastleRights::QueenSide => BitBoard::set(my_backrank, File::A),
            CastleRights::Both => {
                BitBoard::set(my_backrank, File::A)
                    //? Why is this a carrot (^) and not a pipe (|)
                    ^ BitBoard::set(my_backrank, File::H)
            }
        }
    }

    /// Convert the castle rights to an FEN compatible string.
    ///
    /// ```
    /// use chess::{CastleRights, Color};
    ///
    /// assert_eq!(CastleRights::NoRights.to_string(Color::White), "");
    /// assert_eq!(CastleRights::Both.to_string(Color::Black), "kq");
    /// assert_eq!(CastleRights::KingSide.to_string(Color::White), "K");
    /// assert_eq!(CastleRights::QueenSide.to_string(Color::Black), "q");
    /// ```
    #[cfg(feature="std")]
    pub fn to_string(&self, color: Color) -> String {
        let result = match *self {
            CastleRights::NoRights => "",
            CastleRights::KingSide => "k",
            CastleRights::QueenSide => "q",
            CastleRights::Both => "kq",
        };

        if color == Color::White {
            result.to_uppercase()
        } else {
            result.to_string()
        }
    }

    /// Given a square of a rook, which side is it on?
    pub fn rook_square_to_castle_rights(square: Square) -> CastleRights {
        match square.get_file() {
            File::A => CastleRights::QueenSide,
            File::H => CastleRights::KingSide,
            _ => CastleRights::NoRights,
        }
    }

    /// Combine this `CastleRights` with a `Color` (to display)
    pub fn with_color(&self, color: Color) -> CastleRightsWithColor {
        CastleRightsWithColor { castle_rights: *self, color }
    }
}

pub struct CastleRightsWithColor {
    castle_rights: CastleRights,
    color: Color,
}

impl fmt::Display for CastleRightsWithColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.castle_rights.to_str();

        if self.color == Color::White {
            for c in s.chars() {
                write!(f, "{}", c.to_uppercase())?
            }
            Ok(())
        } else {
            write!(f, "{}", s)
        }
    }
}