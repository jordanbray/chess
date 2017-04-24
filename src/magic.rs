use bitboard::{BitBoard, EMPTY};
use square::Square;
use color::Color;
use rank::Rank;
use file::File;

// Include the generated lookup tables
include!(concat!(env!("OUT_DIR"), "/magic_gen.rs"));

/// Get the rays for a bishop on a particular square.
pub fn get_bishop_rays(sq: Square) -> BitBoard {
    unsafe {
        *RAYS.get_unchecked(BISHOP).get_unchecked(sq.to_int() as usize)
    }
}

/// Get the rays for a rook on a particular square.
pub fn get_rook_rays(sq: Square) -> BitBoard {
    unsafe {
        *RAYS.get_unchecked(ROOK).get_unchecked(sq.to_int() as usize)
    }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
pub fn get_rook_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS.get_unchecked(ROOK).get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked((magic.offset as usize) + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift))
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
pub fn get_bishop_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS.get_unchecked(BISHOP).get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked((magic.offset as usize) + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift))
    }
}

/// Get the king moves for a particular square.
pub fn get_king_moves(sq: Square) -> BitBoard {
    unsafe {
        *KING_MOVES.get_unchecked(sq.to_index())
    }
}

/// Get the knight moves for a particular square.
pub fn get_knight_moves(sq: Square) -> BitBoard {
    unsafe {
        *KNIGHT_MOVES.get_unchecked(sq.to_index())
    }
}

/// Get the pawn capture move for a particular square, given the pawn's color and the potential
/// victims
pub fn get_pawn_attacks(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        *PAWN_ATTACKS.get_unchecked(color.to_index()).get_unchecked(sq.to_index()) & blockers
    }
}

/// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
/// the potential blocking pieces.
pub fn get_pawn_quiets(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        if (BitBoard::from_square(sq.uforward(color)) & blockers) != EMPTY {
            EMPTY
        } else {
            *PAWN_MOVES.get_unchecked(color.to_index()).get_unchecked(sq.to_index()) & !blockers
        }
    }
}

/// Get all the pawn moves for a particular square, given the pawn's color and the potential
/// blocking pieces and victims.
pub fn get_pawn_moves(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    get_pawn_attacks(sq, color, blockers) ^ get_pawn_quiets(sq, color, blockers)
}

/// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
/// This line does extend past the squares.
pub fn line(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *LINE.get_unchecked(sq1.to_index()).get_unchecked(sq2.to_index())
    }
}

/// Get a line between these two squares, not including the squares themselves.
pub fn between(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *BETWEEN.get_unchecked(sq1.to_index()).get_unchecked(sq2.to_index())
    }
}

/// Get a `BitBoard` that represents all the squares on a particular rank.
pub fn get_rank(rank: Rank) -> BitBoard {
    unsafe {
        *RANKS.get_unchecked(rank.to_index())
    }
}

/// Get a `BitBoard` that represents all the squares on a particular file.
pub fn get_file(file: File) -> BitBoard {
    unsafe {
        *FILES.get_unchecked(file.to_index())
    }
}

/// Get a `BitBoard` that represents the squares on the 1 or 2 files next to this file.
pub fn get_adjacent_files(file: File) -> BitBoard {
    unsafe {
        *ADJACENT_FILES.get_unchecked(file.to_index())
    }
}

