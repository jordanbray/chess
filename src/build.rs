extern crate rand;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod bitboard;
#[allow(unused_imports)]
use bitboard::{BitBoard, EMPTY};

mod square;
#[allow(unused_imports)]
use square::{Square, NUM_SQUARES, ALL_SQUARES};

mod rank;
#[allow(unused_imports)]
use rank::Rank;

mod file;
#[allow(unused_imports)]
use file::File as ChessFile;

mod piece;
#[allow(unused_imports)]
use piece::Piece;

mod color;
#[allow(unused_imports)]
pub use color::Color;

// the following things are just for move generation

#[derive(Copy, Clone)]
struct Magic {
    magic_number: BitBoard,
    mask: BitBoard,
    offset: u32,
    rightshift: u8,
}


const ROOK: usize = 0;
const BISHOP: usize = 1;

static mut MAGIC_NUMBERS: [[Magic; NUM_SQUARES]; 2 ] =
        [[Magic { magic_number: EMPTY, mask: EMPTY, offset: 0, rightshift: 0 }; 64]; 2];

const ROOK_BITS: usize = 12;
const BISHOP_BITS: usize = 9;
const NUM_MOVES: usize = 64 * (1<<ROOK_BITS) /* Rook Moves */ +
                         64 * (1<<BISHOP_BITS) /* Bishop Moves */;

static mut MOVES: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64];
static mut KNIGHT_MOVES: [BitBoard; 64] = [EMPTY; 64];
static mut PAWN_MOVES: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];
static mut PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// the following are helper variables to cache regularly-used values
static mut LINE: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];
static mut BETWEEN: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];
static mut RAYS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

fn gen_bishop_rays() {
   for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[BISHOP][src.to_index()] =
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                (src_rank - dest_rank).abs() == (src_file - dest_file).abs() &&
                                    *src != **dest})
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

fn gen_rook_rays() {
   for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[ROOK][src.to_index()] =
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index();
                                let src_file = src.get_file().to_index();
                                let dest_rank = dest.get_rank().to_index();
                                let dest_file = dest.get_file().to_index();

                                (src_rank == dest_rank || src_file == dest_file) &&
                                    *src != **dest})
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

fn gen_edges() -> BitBoard {
    ALL_SQUARES.iter()
               .filter(|sq| sq.get_rank() == Rank::First ||
                            sq.get_rank() == Rank::Eighth ||
                            sq.get_file() == ChessFile::A ||
                            sq.get_file() == ChessFile::H)
               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
                            
}

fn gen_lines() {
    for src in ALL_SQUARES.iter() {
        for dest in ALL_SQUARES.iter() {
            unsafe {
                LINE[src.to_index()][dest.to_index()] =
                    ALL_SQUARES.iter()
                               .filter(|test| {
                                    let src_rank = src.get_rank().to_index() as i8;
                                    let src_file = src.get_file().to_index() as i8;
                                    let dest_rank = dest.get_rank().to_index() as i8;
                                    let dest_file = dest.get_file().to_index() as i8;
                                    let test_rank = test.get_rank().to_index() as i8;
                                    let test_file = test.get_file().to_index() as i8;

                                    // test diagonals first
                                    if ((src_rank - dest_rank).abs() ==
                                        (src_file - dest_file).abs() &&
                                        *src != *dest) {
                                        (src_rank - test_rank).abs() ==
                                            (src_file - test_file).abs() &&
                                        (dest_rank - test_rank).abs() ==
                                            (dest_file - test_file).abs()
                                    // next, test rank/file lines
                                    } else if ((src_rank == dest_rank || src_file == dest_file) &&
                                               *src != *dest) {
                                        (src_rank == test_rank &&
                                            dest_rank == test_rank &&
                                            src_rank == dest_rank) ||
                                        (src_file == test_file &&
                                            dest_file == test_file &&
                                            src_file == dest_file)
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

fn gen_knight_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KNIGHT_MOVES[src.to_index()] = 
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                ((src_rank - dest_rank).abs() == 2 &&
                                 (src_file - dest_file).abs() == 1) ||
                                ((src_rank - dest_rank).abs() == 1 &&
                                 (src_file - dest_file).abs() == 2)
                           })
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

fn gen_king_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KING_MOVES[src.to_index()] = 
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                (src_rank - dest_rank).abs() == 1 ||
                                (src_file - dest_file).abs() == 1
                           })
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let magic_path = Path::new(&out_dir).join("magic_gen.rs");
    let mut f = File::create(&magic_path).unwrap();

    f.write_all(b"
        pub fn message() -> &'static str {
            \"Hello World\"
            }").unwrap();
}

