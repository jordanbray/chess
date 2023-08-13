use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::square::ALL_SQUARES;

// Given a square, what are the valid knight moves?
static mut KNIGHT_MOVES: [BitBoard; 64] = [EMPTY; 64];

// Generate the KNIGHT_MOVES array.
pub fn gen_knight_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KNIGHT_MOVES[src.to_index()] = ALL_SQUARES
                .iter()
                .filter(|dest| {
                    let src_rank = src.get_rank().to_index() as i8;
                    let src_file = src.get_file().to_index() as i8;
                    let dest_rank = dest.get_rank().to_index() as i8;
                    let dest_file = dest.get_file().to_index() as i8;

                    ((src_rank - dest_rank).abs() == 2 && (src_file - dest_file).abs() == 1)
                        || ((src_rank - dest_rank).abs() == 1 && (src_file - dest_file).abs() == 2)
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// Write the KNIGHT_MOVES array to the specified file.
#[allow(clippy::needless_range_loop)]
pub fn write_knight_moves(f: &mut File) {
    writeln!(f, "const KNIGHT_MOVES: [BitBoard; 64] = [").unwrap();
    for i in 0..64 {
        unsafe { writeln!(f, "    BitBoard({}),", KNIGHT_MOVES[i].0).unwrap() };
    }
    writeln!(f, "];").unwrap();
}
