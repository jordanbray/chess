use std::fs::File;
use std::io::Write;

use bitboard::{BitBoard, EMPTY};
use color::ALL_COLORS;
use square::ALL_SQUARES;

// Given a square, what are the valid quiet pawn moves (non-captures)?
static mut PAWN_MOVES: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// Given a square, what are the pawn attacks (captures)?
static mut PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// Generate the PAWN_MOVES array.
pub fn gen_pawn_moves() {
    for color in ALL_COLORS.iter() {
        for src in ALL_SQUARES.iter() {
            unsafe {
                if src.get_rank() == color.to_second_rank() {
                    PAWN_MOVES[color.to_index()][src.to_index()] =
                        BitBoard::from_square(src.uforward(*color))
                            ^ BitBoard::from_square(src.uforward(*color).uforward(*color));
                } else {
                    match src.forward(*color) {
                        None => PAWN_MOVES[color.to_index()][src.to_index()] = EMPTY,
                        Some(x) => {
                            PAWN_MOVES[color.to_index()][src.to_index()] = BitBoard::from_square(x)
                        }
                    };
                }
            }
        }
    }
}

// Generate the PAWN_ATTACKS array.
pub fn gen_pawn_attacks() {
    for color in ALL_COLORS.iter() {
        for src in ALL_SQUARES.iter() {
            unsafe {
                PAWN_ATTACKS[color.to_index()][src.to_index()] = EMPTY;
                match src.forward(*color) {
                    None => {}
                    Some(x) => {
                        match x.left() {
                            None => {}
                            Some(y) => {
                                PAWN_ATTACKS[color.to_index()][src.to_index()] ^=
                                    BitBoard::from_square(y)
                            }
                        };
                        match x.right() {
                            None => {}
                            Some(y) => {
                                PAWN_ATTACKS[color.to_index()][src.to_index()] ^=
                                    BitBoard::from_square(y)
                            }
                        };
                    }
                };
            }
        }
    }
}

// Write the PAWN_MOVES array to the specified file.
pub fn write_pawn_moves(f: &mut File) {
    write!(f, "const PAWN_MOVES: [[BitBoard; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", PAWN_MOVES[i][j].to_size(0)).unwrap() };
        }
        if i != 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();
}

// Write the PAWN_ATTACKS array to the specified file.
pub fn write_pawn_attacks(f: &mut File) {
    write!(f, "const PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", PAWN_ATTACKS[i][j].to_size(0)).unwrap() };
        }
        if i != 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();
}
