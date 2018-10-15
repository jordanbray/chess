use std::fs::File;
use std::io::Write;

use bitboard::{BitBoard, EMPTY};
use square::ALL_SQUARES;

// Given a square, what are the valid king moves?
static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64];

// Generate the KING_MOVES array.
pub fn gen_king_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KING_MOVES[src.to_index()] = 
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                ((src_rank - dest_rank).abs() == 1 || (src_rank - dest_rank).abs() == 0) &&
                                ((src_file - dest_file).abs() == 1 || (src_file - dest_file).abs() == 0) &&
                                *src != **dest
                           })
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}


// Write the KING_MOVES array to the specified file.
pub fn write_king_moves(f: &mut File) {
    write!(f, "const KING_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({}),\n", KING_MOVES[i].to_size(0)).unwrap() };
    }
    write!(f, "];\n").unwrap();
}


