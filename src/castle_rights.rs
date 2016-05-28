use bitboard::{BitBoard, EMPTY};
use color::Color;
use square::Square;
use file::File;
/// What castle rights does a particular player have?
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum CastleRights {
    NoRights,
    KingSide,
    QueenSide,
    Both,
}

pub const NUM_CASTLE_RIGHTS: usize = 4;
pub const ALL_CASTLE_RIGHTS: [CastleRights; NUM_CASTLE_RIGHTS] = [CastleRights::NoRights, CastleRights::KingSide, CastleRights::QueenSide, CastleRights::Both];

impl CastleRights {
    /// Can I castle kingside?
    pub fn has_kingside(&self) -> bool {
        self.to_index() & 1 == 1
    }

    /// Can I castle queenside?
    pub fn has_queenside(&self) -> bool {
        self.to_index() & 2 == 2
    }

    /// What squares need to be empty to castle kingside?
    pub fn kingside_squares(&self, color: Color) -> BitBoard {
        BitBoard::set(color.to_my_backrank(), File::F) ^
        BitBoard::set(color.to_my_backrank(), File::G)
    }

    /// What squares need to be empty to castle queenside?
    pub fn queenside_squares(&self, color: Color) -> BitBoard {
        BitBoard::set(color.to_my_backrank(), File::B) ^
        BitBoard::set(color.to_my_backrank(), File::C) ^
        BitBoard::set(color.to_my_backrank(), File::D)
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
            _ => unreachable!()
        }
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    pub fn unmoved_rooks(&self, color: Color) -> BitBoard {
        match *self {
            CastleRights::NoRights => EMPTY,
            CastleRights::KingSide => BitBoard::set(color.to_my_backrank(), File::H),
            CastleRights::QueenSide => BitBoard::set(color.to_my_backrank(), File::A),
            CastleRights::Both => BitBoard::set(color.to_my_backrank(), File::A) ^
                                  BitBoard::set(color.to_my_backrank(), File::H)
        }
    }

    /// Given a square of a rook, which side is it on?
    /// Note: It is invalid to pass in a non-rook square.  The code may panic.
    pub fn rook_square_to_castle_rights(square: Square) -> CastleRights {
        match square.file() {
            File::A => CastleRights::QueenSide,
            File::H => CastleRights::KingSide,
            _       => unreachable!()
        }
    }
}

