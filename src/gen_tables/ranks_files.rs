use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::file::File as ChessFile;
use crate::rank::Rank;
use crate::square::ALL_SQUARES;

// Given a rank, what squares are on that rank?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut RANKS: [BitBoard; 8] = [EMPTY; 8];

// Given a file, what squares are on that file?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut FILES: [BitBoard; 8] = [EMPTY; 8];

// Given a file, what squares are adjacent to that file?  Useful for detecting passed pawns.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut ADJACENT_FILES: [BitBoard; 8] = [EMPTY; 8];

// What are the EDGES of the board?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut EDGES: BitBoard = EMPTY;

// Generate the EDGES, RANKS, FILES, and ADJACENT_FILES variables for storage in the
pub fn gen_bitboard_data() {
    unsafe {
        EDGES = ALL_SQUARES
            .iter()
            .filter(|x| {
                x.get_rank() == Rank::First
                    || x.get_rank() == Rank::Eighth
                    || x.get_file() == ChessFile::A
                    || x.get_file() == ChessFile::H
            })
            .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
        for i in 0..8 {
            RANKS[i] = ALL_SQUARES
                .iter()
                .filter(|x| x.get_rank().to_index() == i)
                .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
            FILES[i] = ALL_SQUARES
                .iter()
                .filter(|x| x.get_file().to_index() == i)
                .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
            ADJACENT_FILES[i] = ALL_SQUARES
                .iter()
                .filter(|y| {
                    ((y.get_file().to_index() as i8) == (i as i8) - 1)
                        || ((y.get_file().to_index() as i8) == (i as i8) + 1)
                })
                .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
        }
    }
}

// Write the FILES array to the specified file.
pub fn write_bitboard_data(f: &mut File) {
    unsafe {
        write!(f, "const FILES: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", FILES[i].0).unwrap();
        }
        write!(f, "];\n").unwrap();
        write!(f, "const ADJACENT_FILES: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", ADJACENT_FILES[i].0).unwrap();
        }
        write!(f, "];\n").unwrap();
        write!(f, "const RANKS: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", RANKS[i].0).unwrap();
        }
        write!(f, "];\n").unwrap();
        write!(f, "/// What are all the edge squares on the `BitBoard`?\n").unwrap();
        write!(
            f,
            "pub const EDGES: BitBoard = BitBoard({});\n",
            EDGES.0
        )
        .unwrap();
    }
}
