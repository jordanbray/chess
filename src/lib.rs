#![doc(html_root_url = "https://jordanbray.github.io/chess/")]
//! This is a rust chess move generator

/// Manage side-to-move information.
pub mod color;
/// Enumerate the piece types (colorless)
pub mod piece;
/// Manage a structure for a square
pub mod square;
/// Manage a structure for a chess move with an optional promotion
pub mod chess_move;
/// Use bitboards to manage piece locations
pub mod bitboard;
/// Keep track of castle rights information, and generate some information related to castling
pub mod castle_rights;
/// Manage an entire chess board
pub mod board;
/// Magic bitboards are the heart of any good chess algorithm.  This module manages lookup tables
/// for move generation.
pub mod magic;
/// Note: You must call construct::construct() before using this library.
pub mod construct;
/// Handle a chess rank, or row.
pub mod rank;
/// Handle a chess file, or column.
pub mod file;
/// Incremental updating hashing algorithm for chess boards.
pub mod zobrist;
/// Cache a particular value for use later, using a u64 as the key.
pub mod cache_table;

extern crate rand;
#[macro_use]
extern crate lazy_static;

