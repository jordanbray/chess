// This file generates 3 giant files, magic_gen.rs and zobrist_gen.rs
// The purpose of this file is to create lookup tables that can be used during move generation.
// This file has gotten pretty long and complicated, but hopefully the comments should allow

#![allow(dead_code)]

// it to be easily followed.
extern crate rand;
mod bitboard;
mod color;
mod file;
mod gen_tables;
mod piece;
mod rank;
mod square;

use gen_tables::generate_all_tables;
// Generate everything.
fn main() {
    generate_all_tables();
}
