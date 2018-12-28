// This file generates 3 giant files, magic_gen.rs and zobrist_gen.rs
// The purpose of this file is to create lookup tables that can be used during move generation.
// This file has gotten pretty long and complicated, but hopefully the comments should allow

#![allow(dead_code)]

// it to be easily followed.
extern crate rand;

mod between;
mod bmis;
mod generate_all_tables;
mod king;
mod knights;
mod lines;
mod magic;
mod magic_helpers;
mod pawns;
mod ranks_files;
mod rays;
mod zobrist;

pub use self::generate_all_tables::generate_all_tables;
