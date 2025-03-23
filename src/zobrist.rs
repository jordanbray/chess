use crate::castle_rights::CastleRights;
use crate::color::{Color, NUM_COLORS};
use crate::file::{File, NUM_FILES};
use crate::piece::{Piece, NUM_PIECES};
use crate::square::{Square, NUM_SQUARES};

/// Create a completely blank type.  This allows all the functions to be part of this type, which I
/// think is a bit cleaner than bare functions everywhere.
pub struct Zobrist;

// Include the generated lookup tables
include!(concat!(env!("OUT_DIR"), "/zobrist_gen.rs"));

impl Zobrist {
    /// Get the value for a particular piece
    #[inline(always)]
    pub fn piece(piece: Piece, square: Square, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_PIECES
                .get_unchecked(color.into_index())
                .get_unchecked(piece.into_index())
                .get_unchecked(square.into_index())
        }
    }

    #[inline(always)]
    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_CASTLES
                .get_unchecked(color.into_index())
                .get_unchecked(castle_rights.into_index())
        }
    }

    #[inline(always)]
    pub fn en_passant(file: File, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_EP
                .get_unchecked(color.into_index())
                .get_unchecked(file.into_index())
        }
    }

    #[inline(always)]
    pub fn color(color: Color) -> u64 {
        if (!color).into() {
            SIDE_TO_MOVE
        } else {
            0
        }
    }
}
