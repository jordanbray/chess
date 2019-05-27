#![doc(html_root_url = "https://jordanbray.github.io/chess/")]
//! # Rust Chess Library
//! This is a chess move generation library for rust.  It is designed to be fast, so that it can be
//! used in a chess engine or UI without performance issues.
//!
//! ## Example
//!
//! This generates all the moves on the starting chess position, and checks that the number of
//! moves is correct.
//!
//! ```
//!
//! use chess::{Board, MoveGen};
//!
//! let board = Board::default();
//! let movegen = MoveGen::new_legal(&board);
//! assert_eq!(movegen.len(), 20);
//! ```
//!

mod board;
pub use crate::board::*;

mod bitboard;
pub use crate::bitboard::{BitBoard, EMPTY};

mod cache_table;
pub use crate::cache_table::*;

mod castle_rights;
pub use crate::castle_rights::*;

mod chess_move;
pub use crate::chess_move::*;

mod color;
pub use crate::color::*;

mod construct;
pub use crate::construct::*;

mod file;
pub use crate::file::*;

mod magic;
pub use crate::magic::{
    between, get_adjacent_files, get_bishop_moves, get_bishop_rays, get_file, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_moves, get_pawn_quiets, get_rank, get_rook_moves,
    get_rook_rays, line, EDGES,
};

#[cfg(target_feature = "bmi2")]
pub use crate::magic::{get_bishop_moves_bmi, get_rook_moves_bmi};

mod piece;
pub use crate::piece::*;

mod rank;
pub use crate::rank::*;

mod square;
pub use crate::square::*;

mod movegen;
pub use crate::movegen::MoveGen;

mod zobrist;

mod game;
pub use crate::game::{Action, Game, GameResult};

mod board_builder;
pub use crate::board_builder::BoardBuilder;

mod error;
pub use crate::error::Error;
