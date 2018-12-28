use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::color::ALL_COLORS;
use crate::file::File as ChessFile;
use crate::rank::Rank;
use crate::square::{Square, ALL_SQUARES};

// Given a square, what are the valid king moves?
static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64];
static mut KINGSIDE_CASTLE_SQUARES: [BitBoard; 2] = [EMPTY; 2];
static mut QUEENSIDE_CASTLE_SQUARES: [BitBoard; 2] = [EMPTY; 2];

// Generate the KING_MOVES array.
pub fn gen_king_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KING_MOVES[src.to_index()] = ALL_SQUARES
                .iter()
                .filter(|dest| {
                    let src_rank = src.get_rank().to_index() as i8;
                    let src_file = src.get_file().to_index() as i8;
                    let dest_rank = dest.get_rank().to_index() as i8;
                    let dest_file = dest.get_file().to_index() as i8;

                    ((src_rank - dest_rank).abs() == 1 || (src_rank - dest_rank).abs() == 0)
                        && ((src_file - dest_file).abs() == 1 || (src_file - dest_file).abs() == 0)
                        && *src != **dest
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }

    gen_kingside_castle_squares();
    gen_queenside_castle_squares();
}

fn gen_kingside_castle_squares() {
    for color in ALL_COLORS.iter() {
        unsafe {
            KINGSIDE_CASTLE_SQUARES[color.to_index()] =
                BitBoard::set(color.to_my_backrank(), ChessFile::F)
                    ^ BitBoard::set(color.to_my_backrank(), ChessFile::G);
        }
    }
}

fn gen_queenside_castle_squares() {
    for color in ALL_COLORS.iter() {
        unsafe {
            QUEENSIDE_CASTLE_SQUARES[color.to_index()] =
                BitBoard::set(color.to_my_backrank(), ChessFile::B)
                    ^ BitBoard::set(color.to_my_backrank(), ChessFile::C)
                    ^ BitBoard::set(color.to_my_backrank(), ChessFile::D);
        }
    }
}

fn gen_castle_moves() -> BitBoard {
    let c1 = Square::make_square(Rank::First, ChessFile::C);
    let c8 = Square::make_square(Rank::Eighth, ChessFile::C);
    let e1 = Square::make_square(Rank::First, ChessFile::E);
    let e8 = Square::make_square(Rank::Eighth, ChessFile::E);
    let g1 = Square::make_square(Rank::First, ChessFile::G);
    let g8 = Square::make_square(Rank::Eighth, ChessFile::G);

    BitBoard::from_square(c1)
        ^ BitBoard::from_square(c8)
        ^ BitBoard::from_square(e1)
        ^ BitBoard::from_square(e8)
        ^ BitBoard::from_square(g1)
        ^ BitBoard::from_square(g8)
}

// Write the KING_MOVES array to the specified file.
pub fn write_king_moves(f: &mut File) {
    write!(f, "const KING_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({}),\n", KING_MOVES[i].to_size(0)).unwrap() };
    }
    write!(f, "];\n").unwrap();

    write!(f, "pub const KINGSIDE_CASTLE_SQUARES: [BitBoard; 2] = [\n").unwrap();
    unsafe {
        write!(
            f,
            " BitBoard({}), BitBoard({})];\n",
            KINGSIDE_CASTLE_SQUARES[0].to_size(0),
            KINGSIDE_CASTLE_SQUARES[1].to_size(0)
        )
        .unwrap()
    };

    write!(f, "pub const QUEENSIDE_CASTLE_SQUARES: [BitBoard; 2] = [\n").unwrap();
    unsafe {
        write!(
            f,
            " BitBoard({}), BitBoard({})];\n",
            QUEENSIDE_CASTLE_SQUARES[0].to_size(0),
            QUEENSIDE_CASTLE_SQUARES[1].to_size(0)
        )
        .unwrap()
    };

    write!(
        f,
        "const CASTLE_MOVES: BitBoard = BitBoard({});\n",
        gen_castle_moves().to_size(0)
    )
    .unwrap();
}
