use piece::{Piece, NUM_PIECES, ALL_PIECES};
use square::{Square, NUM_SQUARES, ALL_SQUARES};
use color::{Color, NUM_COLORS, ALL_COLORS};
use castle_rights::{CastleRights, NUM_CASTLE_RIGHTS, ALL_CASTLE_RIGHTS};
use file::{File, NUM_FILES, ALL_FILES};
use rand::{weak_rng, Rng, SeedableRng};
use std::sync::{Once, ONCE_INIT};

// these variables represent xorable values that are added/removed from a hash as the board gets
// updated
static mut ZOBRIST_PIECES: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[[0; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS];
static mut ZOBRIST_CASTLES: [[u64; NUM_CASTLE_RIGHTS]; NUM_COLORS] = [[0; NUM_CASTLE_RIGHTS]; NUM_COLORS];
static mut ZOBRIST_EP: [[u64; NUM_FILES]; NUM_COLORS] = [[0; NUM_FILES]; NUM_COLORS];
static mut SIDE_TO_MOVE: u64 = 0;

/// have these variables been set up yet?
static SETUP: Once = ONCE_INIT;

/// Create a completely blank type.  This allows all the functions to be part of this type, which I
/// think is a bit cleaner than bare functions everywhere.
pub struct Zobrist;

impl Zobrist {
    /// Initialized zobrist numbers for incremental update hashing.  Must be called before hashes can be
    /// used for board objects.
    pub fn construct() {
        SETUP.call_once(|| {
            let mut rng = weak_rng();
            rng.reseed([0xDEADBEEF, 0xBEEFDEAD, 0xABCDEFAB, 0x12345678]);

            unsafe {
                SIDE_TO_MOVE = rng.next_u64();
            }

            for color in ALL_COLORS.iter() {
                for sq in ALL_SQUARES.iter() {
                    for piece in ALL_PIECES.iter() {
                        unsafe {
                            ZOBRIST_PIECES[color.to_index()][piece.to_index()][sq.to_index()] = rng.next_u64();
                        }
                    }
                }
                for cr in ALL_CASTLE_RIGHTS.iter() {
                    unsafe {
                        ZOBRIST_CASTLES[color.to_index()][cr.to_index()] = rng.next_u64();
                    }
                }
                for file in ALL_FILES.iter() {
                    unsafe {
                        ZOBRIST_EP[color.to_index()][file.to_index()] = rng.next_u64();
                    }
                }
            }
        });
    }

    /// Get the value for a particular piece
    pub fn piece(piece: Piece, square: Square, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_PIECES.get_unchecked(color.to_index()).get_unchecked(piece.to_index()).get_unchecked(square.to_index())
        }
    }

    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_CASTLES.get_unchecked(color.to_index()).get_unchecked(castle_rights.to_index())
        }
    }

    pub fn en_passant(file: File, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_EP.get_unchecked(color.to_index()).get_unchecked(file.to_index())
        }
    }

    pub fn color() -> u64 {
        unsafe {
            SIDE_TO_MOVE
        }
    }
}
