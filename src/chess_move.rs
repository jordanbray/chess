use piece::Piece;
use square::Square;

/// Represent a ChessMove in memory
#[derive(Clone, Copy, PartialOrd, PartialEq)]
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
