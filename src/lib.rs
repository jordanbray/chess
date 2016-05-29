#![doc(html_root_url = "https://jordanbray.github.io/chess/")]
//! # Rust Chess Library
//! This is a chess move generation library for rust.  It is designed to be fast, so that it can be
//! used in a chess engine or UI without performance issues.

mod board;
pub use board::*;

mod bitboard;
pub use bitboard::*;

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
pub use magic::*;

mod piece;
pub use piece::*;

mod rank;
pub use rank::*;

mod square;
pub use square::*;

mod zobrist;

extern crate rand;
#[macro_use]
extern crate lazy_static;

