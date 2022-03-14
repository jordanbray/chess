use crate::between;
use crate::bitboard::{BitBoard, EMPTY};
use crate::color::Color;
use crate::file::File;
use crate::square::Square;

/// Represents the possible castle types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CastleType {
    Kingside,
    Queenside
}

impl CastleType {
    /// The file the king ends up on after this type of castling
    #[inline(always)]
    #[allow(dead_code)]
    pub fn king_dest_file(&self) -> File {
        match self {
            CastleType::Kingside => File::G,
            CastleType::Queenside => File::C,
        }
    }

    /// The square the king ends up on after this type of castling
    #[inline(always)]
    #[allow(dead_code)]
    pub fn king_dest(&self, color: Color) -> Square {
        Square::make_square(color.to_my_backrank(), self.king_dest_file())
    }

    /// The file the rook ends up on after this type of castling
    #[inline(always)]
    pub fn rook_dest_file(&self) -> File {
        match self {
            CastleType::Kingside => File::F,
            CastleType::Queenside => File::D,
        }
    }

    /// The square the rook ends up on after this type of castling
    #[inline(always)]
    pub fn rook_dest(&self, color: Color) -> Square {
        Square::make_square(color.to_my_backrank(), self.rook_dest_file())
    }


    /// What squares need to be empty to castle kingside?
    fn kingside_squares(king_square: Square) -> BitBoard {
        let dest = Square::make_square(king_square.get_rank(), File::G);
        between(king_square, dest) | BitBoard::from_square(dest)
    }

    /// What squares need to be empty to castle queenside?
    fn queenside_squares(king_square: Square) -> BitBoard {
        let dest = Square::make_square(king_square.get_rank(), File::C);
        between(king_square, dest) | BitBoard::from_square(dest)
    }

    /// The squares that the must be clear and not under attack for the king
    /// to be able to castle
    #[inline(always)]
    pub fn king_journey_squares(&self, king_square: Square) -> BitBoard {
        match self {
            CastleType::Kingside => Self::kingside_squares(king_square),
            CastleType::Queenside => Self::queenside_squares(king_square),
        }
    }
}

/// What castle rights does a particular player have?
/// 
/// `CastleRights` is a pair of `Option<File>` values, one for kingside castling,
/// and one for queenside castling.
/// 
/// If the value is `None`, we don't have that castling right. If it is `Some(file)`, we can
/// castle with our rook positioned on the given `file`.
/// 
/// This allows supporting chess 960 positions in addition to normal chess positions.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct CastleRights {
    pub(crate) kingside: Option<File>,
    pub(crate) queenside: Option<File>
}

impl Default for CastleRights {
    #[inline]
    fn default() -> Self {
        Self { queenside: None, kingside: None }
    }
}

impl CastleRights {

    /// Crate a new `CastleRights` value given the kingside and queenside castle rights.
    pub fn new(kingside: Option<File>, queenside: Option<File>) -> CastleRights {
        CastleRights{kingside, queenside}
    }

    /// Can I castle kingside?
    #[inline(always)]
    pub fn has_kingside(&self) -> bool {
        self.kingside.is_some()
    }

    /// Can I castle queenside?
    #[inline(always)]
    pub fn has_queenside(&self) -> bool {
        self.queenside.is_some()
    }

    /// Can I castle both sides?
    #[inline(always)]
    pub fn has_both(&self) -> bool {
        self.has_kingside() & self.has_queenside()
    }

    /// Can I castle either side?
    #[inline(always)]
    pub fn has_any(&self) -> bool {
        self.has_kingside() | self.has_queenside()
    }

    /// Do I have the given castle right?
    #[inline(always)]
    pub fn has(&self, castle_type: CastleType) -> bool {
        match castle_type {
            CastleType::Kingside => self.kingside.is_some(),
            CastleType::Queenside => self.queenside.is_some(),
        }
    }

    /// Return the value for the given castle type
    #[inline(always)]
    pub fn get(&self, castle_type: CastleType) -> Option<File> {
        match castle_type {
            CastleType::Kingside => self.kingside,
            CastleType::Queenside => self.queenside,
        }
    }

    pub fn to_string(&self, color: Color) -> String {
        let queenside = match self.queenside {
            Some(file) => file.to_string(),
            None => "".to_string(),
        };
        let kingside = match self.kingside {
            Some(file) => file.to_string(),
            None => "".to_string()
        };
        let result = format!("{}{}", queenside, kingside);

        if color == Color::Black {
            result.to_lowercase()
        } else {
            result
        }
    }

    /// Remove castle rights, and return a new `CastleRights`.
    pub fn remove(&self, remove: File) -> CastleRights {
        let mut res = *self;
        if res.kingside == Some(remove) { res.kingside = None; }
        else if res.queenside == Some(remove) {res.queenside = None;}
        res
    }

    #[allow(non_upper_case_globals)]
    pub const NoRights: Self = CastleRights{ kingside: None, queenside: None};

    /// What rooks are guaranteed to not have moved given this castle rights?
    pub fn unmoved_rooks(&self, color: Color) -> BitBoard {
        let mut res = EMPTY;
        if let Some(file) = self.kingside {
            res |= BitBoard::set(color.to_my_backrank(), file);
        }
        if let Some(file) = self.queenside {
            res |= BitBoard::set(color.to_my_backrank(), file);
        }
        res
    }

    /// for hashing purposes only
    pub(crate) fn to_index(&self) -> usize {
        self.kingside.is_some() as usize + self.queenside.is_some() as usize * 2
    }
}
