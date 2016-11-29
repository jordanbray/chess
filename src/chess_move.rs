use piece::Piece;
use square::Square;
use std::fmt;
use std::cmp::Ordering;

/// Represent a ChessMove in memory
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq)]
pub struct ChessMove {
    source: Square,
    dest: Square,
    promotion: Option<Piece>
}

impl ChessMove {
    /// Create a new chess move, given a source `Square`, a destination `Square`, and an optional
    /// promotion `Piece`
    pub fn new(source: Square, dest: Square, promotion: Option<Piece>) -> ChessMove {
        ChessMove { source: source, dest: dest, promotion: promotion }
    }

    /// Get the source square (square the piece is currently on).
    pub fn get_source(&self) -> Square {
        self.source
    }

    /// Get the destination square (square the piece is going to).
    pub fn get_dest(&self) -> Square {
        self.dest
    }

    /// Get the promotion piece (maybe).
    pub fn get_promotion(&self) -> Option<Piece> {
        self.promotion
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            None => write!(f, "{}-{}", self.source, self.dest),
            Some(x) => write!(f, "{}-{}={}", self.source, self.dest, x)
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
                Some(x) => {
                    match other.promotion {
                        None => Ordering::Greater,
                        Some(y) => x.cmp(&y)
                    }
                }
            }
        } else {
            Ordering::Equal
        }
    }
}
 
