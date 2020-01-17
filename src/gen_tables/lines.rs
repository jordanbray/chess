use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::square::ALL_SQUARES;

// Given two squares, lookup a line going through those two squares, or return EMPTY.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut LINE: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];

// Generate the LINES array.
pub fn gen_lines() {
    for src in ALL_SQUARES.iter() {
        for dest in ALL_SQUARES.iter() {
            unsafe {
                LINE[src.to_index()][dest.to_index()] = ALL_SQUARES
                    .iter()
                    .filter(|test| {
                        let src_rank = src.get_rank().to_index() as i8;
                        let src_file = src.get_file().to_index() as i8;
                        let dest_rank = dest.get_rank().to_index() as i8;
                        let dest_file = dest.get_file().to_index() as i8;
                        let test_rank = test.get_rank().to_index() as i8;
                        let test_file = test.get_file().to_index() as i8;

                        // test diagonals first
                        if (src_rank - dest_rank).abs() == (src_file - dest_file).abs()
                            && *src != *dest
                        {
                            (src_rank - test_rank).abs() == (src_file - test_file).abs()
                                && (dest_rank - test_rank).abs() == (dest_file - test_file).abs()
                        // next, test rank/file lines
                        } else if (src_rank == dest_rank || src_file == dest_file) && *src != *dest
                        {
                            (src_rank == test_rank && dest_rank == test_rank)
                                || (src_file == test_file && dest_file == test_file)
                        // if src and dest don't line up, there is no line.  Return
                        // EMPTY
                        } else {
                            false
                        }
                    })
                    .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
            }
        }
    }
}

// Write the LINE array to the specified file.
pub fn write_lines(f: &mut File) {
    write!(f, "const LINE: [[BitBoard; 64]; 64] = [[\n").unwrap();
    for i in 0..64 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", LINE[i][j].0).unwrap() };
        }
        if i != 63 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();
}
