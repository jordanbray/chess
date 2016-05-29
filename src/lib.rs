#![doc(html_root_url = "https://jordanbray.github.io/chess/")]

/// This is a rust chess move generator.
pub mod color;
pub mod piece;
pub mod square;
pub mod chess_move;
pub mod bitboard;
pub mod castle_rights;
pub mod board;
pub mod magic;
pub mod construct;
pub mod rank;
pub mod file;
pub mod zobrist;
pub mod cache_table;

extern crate rand;
#[macro_use]
extern crate lazy_static;

