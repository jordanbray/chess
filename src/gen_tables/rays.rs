use crate::bitboard::{BitBoard, EMPTY};
use crate::piece::Piece;
use crate::square::{Square, ALL_SQUARES};
use std::fs::File;
use std::io::Write;

// Given a square and a piece type (rook or bishop only), what are the squares they
// would attack if no pieces were on the board?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut RAYS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// For each square, generate the RAYS for the bishop.
pub fn gen_bishop_rays() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[1][src.to_index()] = ALL_SQUARES
                .iter()
                .filter(|dest| {
                    let src_rank = src.get_rank().to_index() as i8;
                    let src_file = src.get_file().to_index() as i8;
                    let dest_rank = dest.get_rank().to_index() as i8;
                    let dest_file = dest.get_file().to_index() as i8;

                    (src_rank - dest_rank).abs() == (src_file - dest_file).abs() && *src != **dest
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// For each square, generate the RAYS for the rook.
pub fn gen_rook_rays() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[0][src.to_index()] = ALL_SQUARES
                .iter()
                .filter(|dest| {
                    let src_rank = src.get_rank().to_index();
                    let src_file = src.get_file().to_index();
                    let dest_rank = dest.get_rank().to_index();
                    let dest_file = dest.get_file().to_index();

                    (src_rank == dest_rank || src_file == dest_file) && *src != **dest
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

pub fn get_rays(sq: Square, piece: Piece) -> BitBoard {
    unsafe { RAYS[if piece == Piece::Rook { 0 } else { 1 }][sq.to_index()] }
}

// Write the RAYS array to the specified file.
pub fn write_rays(f: &mut File) {
    writeln!(f, "const ROOK: usize = {};", 0).unwrap();
    writeln!(f, "const BISHOP: usize = {};", 1).unwrap();
    writeln!(f, "const RAYS: [[BitBoard; 64]; 2] = [[").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { writeln!(f, "    BitBoard({}),", RAYS[i][j].0).unwrap() };
        }
        if i != 1 {
            writeln!(f, "  ], [").unwrap();
        }
    }
    writeln!(f, "]];").unwrap();
}
