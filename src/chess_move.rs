use crate::piece::Piece;
use crate::square::Square;
use std::cmp::Ordering;
use std::fmt;

/// Represent a ChessMove in memory
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Default, Debug, Hash)]
pub struct ChessMove {
    source: Square,
    dest: Square,
    promotion: Option<Piece>,
}

impl ChessMove {
    /// Create a new chess move, given a source `Square`, a destination `Square`, and an optional
    /// promotion `Piece`
    #[inline]
    pub fn new(source: Square, dest: Square, promotion: Option<Piece>) -> ChessMove {
        ChessMove {
            source: source,
            dest: dest,
            promotion: promotion,
        }
    }

    /// Get the source square (square the piece is currently on).
    #[inline]
    pub fn get_source(&self) -> Square {
        self.source
    }

    /// Get the destination square (square the piece is going to).
    #[inline]
    pub fn get_dest(&self) -> Square {
        self.dest
    }

    /// Get the promotion piece (maybe).
    #[inline]
    pub fn get_promotion(&self) -> Option<Piece> {
        self.promotion
    }

    /// Convert a UCI `String` to a move. If invalid, return `None`
    /// ```
    /// use chess::{ChessMove, Square, Piece};
    ///
    /// let mv = ChessMove::new(Square::E7, Square::E8, Some(Piece::Queen));
    ///
    /// assert_eq!(ChessMove::from_string("e7e8q".to_owned()).expect("Valid Move"), mv);
    /// ```
    #[inline]
    pub fn from_string(s: String) -> Option<ChessMove> {
        let source = Square::from_string(s.get(0..2)?.to_string())?;
        let dest = Square::from_string(s.get(2..4)?.to_string())?;

        let mut promo = None;
        if s.len() == 5 {
            promo = Some(match s.chars().last()? {
                'q' => Piece::Queen,
                'r' => Piece::Rook,
                'n' => Piece::Knight,
                'b' => Piece::Bishop,
                _ => return None,
            });
        }

        Some(ChessMove::new(source, dest, promo))
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            None => write!(f, "{}{}", self.source, self.dest),
            Some(x) => write!(f, "{}{}{}", self.source, self.dest, x),
        }
    }
}

impl Ord for ChessMove {
    fn cmp(&self, other: &ChessMove) -> Ordering {
        if self.source != other.source {
            self.source.cmp(&other.source)
        } else if self.dest != other.dest {
            self.dest.cmp(&other.dest)
        } else if self.promotion != other.promotion {
            match self.promotion {
                None => Ordering::Less,
                Some(x) => match other.promotion {
                    None => Ordering::Greater,
                    Some(y) => x.cmp(&y),
                },
            }
        } else {
            Ordering::Equal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_uci_moves() {
        assert_eq!(ChessMove::from_string("e2e-".to_owned()), None);
        assert_eq!(ChessMove::from_string("".to_owned()), None);
        assert_eq!(ChessMove::from_string("e7e8p".to_owned()), None);
        assert_eq!(ChessMove::from_string("e7e8z".to_owned()), None);
    }

    #[test]
    fn valid_uci_moves() {
        assert_eq!(
            ChessMove::from_string("e2e4".to_owned()),
            Some(ChessMove::new(Square::E2, Square::E4, None))
        );
        assert_eq!(
            ChessMove::from_string("g1f3".to_owned()),
            Some(ChessMove::new(Square::G1, Square::F3, None))
        );
        assert_eq!(
            ChessMove::from_string("a2a4".to_owned()),
            Some(ChessMove::new(Square::A2, Square::A4, None))
        );
        assert_eq!(
            ChessMove::from_string("h2h4".to_owned()),
            Some(ChessMove::new(Square::H2, Square::H4, None))
        );
    }
}
