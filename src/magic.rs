use crate::bitboard::{BitBoard, EMPTY};
use crate::color::Color;
use crate::file::File;
use crate::rank::Rank;
use crate::square::Square;
#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

// Include the generated lookup tables
include!(concat!(env!("OUT_DIR"), "/magic_gen.rs"));

/// Get the rays for a bishop on a particular square.
#[inline]
pub fn get_bishop_rays(sq: Square) -> BitBoard {
    unsafe { *RAYS.get_unchecked(BISHOP).get_unchecked(sq.to_index()) }
}

/// Get the rays for a rook on a particular square.
#[inline]
pub fn get_rook_rays(sq: Square) -> BitBoard {
    unsafe { *RAYS.get_unchecked(ROOK).get_unchecked(sq.to_index()) }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
pub fn get_rook_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS
            .get_unchecked(ROOK)
            .get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked(
            (magic.offset as usize)
                + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift),
        ) & get_rook_rays(sq)
    }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[cfg(target_feature = "bmi2")]
#[inline]
pub fn get_rook_moves_bmi(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let bmi2_magic = *ROOK_BMI_MASK.get_unchecked(sq.to_int() as usize);
        let index = (_pext_u64(blockers.0, bmi2_magic.blockers_mask.0) as usize)
            + (bmi2_magic.offset as usize);
        let result = _pdep_u64(
            *BMI_MOVES.get_unchecked(index as usize) as u64,
            get_rook_rays(sq).0,
        );
        return BitBoard(result);
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
pub fn get_bishop_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS
            .get_unchecked(BISHOP)
            .get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked(
            (magic.offset as usize)
                + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift),
        ) & get_bishop_rays(sq)
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(target_feature = "bmi2")]
pub fn get_bishop_moves_bmi(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let bmi2_magic = *BISHOP_BMI_MASK.get_unchecked(sq.to_int() as usize);
        let index = (_pext_u64(blockers.0, bmi2_magic.blockers_mask.0) as usize)
            + (bmi2_magic.offset as usize);
        let result = _pdep_u64(
            *BMI_MOVES.get_unchecked(index as usize) as u64,
            get_bishop_rays(sq).0,
        );
        return BitBoard(result);
    }
}

/// Get the king moves for a particular square.
#[inline]
pub fn get_king_moves(sq: Square) -> BitBoard {
    unsafe { *KING_MOVES.get_unchecked(sq.to_index()) }
}

/// Get the knight moves for a particular square.
#[inline]
pub fn get_knight_moves(sq: Square) -> BitBoard {
    unsafe { *KNIGHT_MOVES.get_unchecked(sq.to_index()) }
}

/// Get the pawn capture move for a particular square, given the pawn's color and the potential
/// victims
#[inline]
pub fn get_pawn_attacks(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        *PAWN_ATTACKS
            .get_unchecked(color.to_index())
            .get_unchecked(sq.to_index())
            & blockers
    }
}
/// Get the legal destination castle squares for both players
#[inline]
pub fn get_castle_moves() -> BitBoard {
    CASTLE_MOVES
}

/// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
/// the potential blocking pieces.
#[inline]
pub fn get_pawn_quiets(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        if (BitBoard::from_square(sq.uforward(color)) & blockers) != EMPTY {
            EMPTY
        } else {
            *PAWN_MOVES
                .get_unchecked(color.to_index())
                .get_unchecked(sq.to_index())
                & !blockers
        }
    }
}

/// Get all the pawn moves for a particular square, given the pawn's color and the potential
/// blocking pieces and victims.
#[inline]
pub fn get_pawn_moves(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    get_pawn_attacks(sq, color, blockers) ^ get_pawn_quiets(sq, color, blockers)
}

/// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
/// This line does extend past the squares.
#[inline]
pub fn line(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *LINE
            .get_unchecked(sq1.to_index())
            .get_unchecked(sq2.to_index())
    }
}

/// Get a line between these two squares, not including the squares themselves.
#[inline]
pub fn between(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *BETWEEN
            .get_unchecked(sq1.to_index())
            .get_unchecked(sq2.to_index())
    }
}

/// Get a `BitBoard` that represents all the squares on a particular rank.
#[inline]
pub fn get_rank(rank: Rank) -> BitBoard {
    unsafe { *RANKS.get_unchecked(rank.to_index()) }
}

/// Get a `BitBoard` that represents all the squares on a particular file.
#[inline]
pub fn get_file(file: File) -> BitBoard {
    unsafe { *FILES.get_unchecked(file.to_index()) }
}

/// Get a `BitBoard` that represents the squares on the 1 or 2 files next to this file.
#[inline]
pub fn get_adjacent_files(file: File) -> BitBoard {
    unsafe { *ADJACENT_FILES.get_unchecked(file.to_index()) }
}

#[inline]
pub fn get_pawn_source_double_moves() -> BitBoard {
    PAWN_SOURCE_DOUBLE_MOVES
}

#[inline]
pub fn get_pawn_dest_double_moves() -> BitBoard {
    PAWN_DEST_DOUBLE_MOVES
}
