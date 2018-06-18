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
//! use chess::{Board, ChessMove};
//!
//! let board = Board::default();
//! let mut moves = [ChessMove::default(); 256];
//! let count = board.enumerate_moves(&mut moves);
//! assert_eq!(count, 20);
//! ```
//!

mod board;
pub use board::*;

mod bitboard;
pub use bitboard::{BitBoard, EMPTY};

mod cache_table;
pub use cache_table::*;

mod castle_rights;
pub use castle_rights::*;

mod chess_move;
pub use chess_move::*;

mod color;
pub use color::*;

mod construct;
pub use construct::*;

mod file;
pub use file::*;

mod magic;
pub use magic::{get_bishop_rays, get_rook_rays, get_rook_moves, get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_quiets, get_pawn_moves, line, between, get_rank, get_file, get_adjacent_files, EDGES};

mod piece;
pub use piece::*;

mod rank;
pub use rank::*;

mod square;
pub use square::*;

mod movegen;
pub use movegen::*;

mod zobrist;

