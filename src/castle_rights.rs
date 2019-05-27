use std::hint::unreachable_unchecked;

use crate::bitboard::{BitBoard, EMPTY};
use crate::color::Color;
use crate::file::File;
use crate::square::Square;

use crate::magic::{KINGSIDE_CASTLE_SQUARES, QUEENSIDE_CASTLE_SQUARES};

/// What castle rights does a particular player have?
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Hash)]
pub enum CastleRights {
    NoRights,
    KingSide,
    QueenSide,
    Both,
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
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, // 1
        0, 0, 0, 0, 0, 0, 0, 0, // 2
        0, 0, 0, 0, 0, 0, 0, 0, // 3
        0, 0, 0, 0, 0, 0, 0, 0, // 4
        0, 0, 0, 0, 0, 0, 0, 0, // 5
        0, 0, 0, 0, 0, 0, 0, 0, // 6
        0, 0, 0, 0, 0, 0, 0, 0, // 7
        2, 0, 0, 0, 3, 0, 0, 1,
    ],
];

impl CastleRights {
    /// Can I castle kingside?
    pub fn has_kingside(&self) -> bool {
        self.to_index() & 1 == 1
    }

    /// Can I castle queenside?
    pub fn has_queenside(&self) -> bool {
        self.to_index() & 2 == 2
    }

    pub fn square_to_castle_rights(color: Color, sq: Square) -> CastleRights {
        CastleRights::from_index(unsafe {
            *CASTLES_PER_SQUARE
                .get_unchecked(color.to_index())
                .get_unchecked(sq.to_index())
        } as usize)
    }

    /// What squares need to be empty to castle kingside?
    pub fn kingside_squares(&self, color: Color) -> BitBoard {
        unsafe { *KINGSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    }

    /// What squares need to be empty to castle queenside?
    pub fn queenside_squares(&self, color: Color) -> BitBoard {
        unsafe { *QUEENSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    }

    /// Remove castle rights, and return a new `CastleRights`.
    pub fn remove(&self, remove: CastleRights) -> CastleRights {
        CastleRights::from_index(self.to_index() & !remove.to_index())
    }

    /// Add some castle rights, and return a new `CastleRights`.
    pub fn add(&self, add: CastleRights) -> CastleRights {
        CastleRights::from_index(self.to_index() | add.to_index())
    }

    /// Convert `CastleRights` to `usize` for table lookups
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    pub fn from_index(i: usize) -> CastleRights {
        match i {
            0 => CastleRights::NoRights,
            1 => CastleRights::KingSide,
            2 => CastleRights::QueenSide,
            3 => CastleRights::Both,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    pub fn unmoved_rooks(&self, color: Color) -> BitBoard {
        match *self {
            CastleRights::NoRights => EMPTY,
            CastleRights::KingSide => BitBoard::set(color.to_my_backrank(), File::H),
            CastleRights::QueenSide => BitBoard::set(color.to_my_backrank(), File::A),
            CastleRights::Both => {
                BitBoard::set(color.to_my_backrank(), File::A)
                    ^ BitBoard::set(color.to_my_backrank(), File::H)
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
    /// Note: It is invalid to pass in a non-rook square.  The code may panic.
    pub fn rook_square_to_castle_rights(square: Square) -> CastleRights {
        match square.get_file() {
            File::A => CastleRights::QueenSide,
            File::H => CastleRights::KingSide,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
