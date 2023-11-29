use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::square::ALL_SQUARES;

// Given two squares, lookup a line between those two squares, or return EMPTY.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut BETWEEN: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];

// Is a number (t) between two numbers (a and b)?
fn between(a: i8, t: i8, b: i8) -> bool {
    if a < b {
        a < t && t < b
    } else {
        b < t && t < a
    }
}

// Generate the BETWEEN array.
pub fn gen_between() {
    for src in ALL_SQUARES.iter() {
        for dest in ALL_SQUARES.iter() {
            unsafe {
                BETWEEN[src.to_index()][dest.to_index()] = ALL_SQUARES
                    .iter()
                    .filter(|test| {
                        let src_rank = src.get_rank().to_index() as i8;
                        let src_file = src.get_file().to_index() as i8;
                        let dest_rank = dest.get_rank().to_index() as i8;
                        let dest_file = dest.get_file().to_index() as i8;
                        let test_rank = test.get_rank().to_index() as i8;
                        let test_file = test.get_file().to_index() as i8;

                        // test diagonals first, as above
                        if (src_rank - dest_rank).abs() == (src_file - dest_file).abs()
                            && *src != *dest
                        {
                            (src_rank - test_rank).abs() == (src_file - test_file).abs()
                                && (dest_rank - test_rank).abs() == (dest_file - test_file).abs()
                                && between(src_rank, test_rank, dest_rank)
                        } else if (src_rank == dest_rank || src_file == dest_file) && *src != *dest
                        {
                            (src_rank == test_rank
                                && dest_rank == test_rank
                                && between(src_file, test_file, dest_file))
                                || (src_file == test_file
                                    && dest_file == test_file
                                    && between(src_rank, test_rank, dest_rank))
                        } else {
                            false
                        }
                    })
                    .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
            }
        }
    }
}

// Write the BETWEEN array to the specified file.
#[allow(clippy::needless_range_loop)]
pub fn write_between(f: &mut File) {
    writeln!(f, "const BETWEEN: [[BitBoard; 64]; 64] = [[").unwrap();
    for i in 0..64 {
        for j in 0..64 {
            unsafe { writeln!(f, "    BitBoard({}),", BETWEEN[i][j].0).unwrap() };
        }
        if i != 63 {
            writeln!(f, "  ], [").unwrap();
        }
    }
    writeln!(f, "]];").unwrap();
}
