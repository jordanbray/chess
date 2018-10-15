use castle_rights::{CastleRights, NUM_CASTLE_RIGHTS};
use color::{Color, NUM_COLORS};
use file::{File, NUM_FILES};
use piece::{Piece, NUM_PIECES};
use square::{Square, NUM_SQUARES};

/// Create a completely blank type.  This allows all the functions to be part of this type, which I
/// think is a bit cleaner than bare functions everywhere.
pub struct Zobrist;

// Include the generated lookup tables
include!(concat!(env!("OUT_DIR"), "/zobrist_gen.rs"));

impl Zobrist {
    /// Get the value for a particular piece
    pub fn piece(piece: Piece, square: Square, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_PIECES
                .get_unchecked(color.to_index())
                .get_unchecked(piece.to_index())
                .get_unchecked(square.to_index())
        }
    }

    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_CASTLES
                .get_unchecked(color.to_index())
                .get_unchecked(castle_rights.to_index())
        }
    }

    pub fn en_passant(file: File, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_EP
                .get_unchecked(color.to_index())
                .get_unchecked(file.to_index())
        }
    }

    pub fn color() -> u64 {
        SIDE_TO_MOVE
    }
}
