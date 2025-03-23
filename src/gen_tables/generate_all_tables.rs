// This file generates 3 giant files, magic_gen.rs and zobrist_gen.rs
// The purpose of this file is to create lookup tables that can be used during move generation.
// This file has gotten pretty long and complicated, but hopefully the comments should allow
// it to be easily followed.

use std::env;
use std::fs::File;
use std::path::Path;

use crate::gen_tables::between::*;
use crate::gen_tables::king::*;
use crate::gen_tables::knights::*;
use crate::gen_tables::lines::*;
use crate::gen_tables::pawns::*;
use crate::gen_tables::ranks_files::*;
use crate::gen_tables::rays::*;
use crate::gen_tables::zobrist::*;

#[cfg(target_feature = "bmi2")]
use crate::gen_tables::bmis::*;
use crate::gen_tables::magic::*;

pub fn generate_all_tables() {
    gen_lines(); // LINE
    gen_between(); // BETWEEN
    gen_bishop_rays(); // RAYS (ind of below)
    gen_rook_rays(); // RAYS (ind of above)
    gen_knight_moves(); // KNIGHT_MOVES
    gen_king_moves(); // KING_MOVES
    gen_pawn_attacks(); // PAWN_ATTACKS
    gen_pawn_moves(); // PAWN_MOVES
    gen_all_magic(); // MOVE_RAYS, MAGIC_NUMBERS, MOVES, GENERATED_NUM_MOVES
    gen_bitboard_data(); // EDGES, RANKS, ADJACENT_FILES, FILES
    #[cfg(target_feature = "bmi2")]
    gen_all_bmis(); // BISHOP_BMI_MASK, ROOK_BMI_MASK, BMI_MOVES, GENERATED_BMI_MOVES

    let out_dir = env::var("OUT_DIR").unwrap();
    let magic_path = Path::new(&out_dir).join("magic_gen.rs");
    let mut f = File::create(magic_path).unwrap();

    write_king_moves(&mut f);
    write_knight_moves(&mut f);
    write_rays(&mut f);
    write_between(&mut f);
    write_lines(&mut f);
    write_pawn_attacks(&mut f);
    write_pawn_moves(&mut f);
    write_magic(&mut f);
    #[cfg(target_feature = "bmi2")]
    write_bmis(&mut f);
    write_bitboard_data(&mut f);

    let zobrist_path = Path::new(&out_dir).join("zobrist_gen.rs");
    let mut z = File::create(zobrist_path).unwrap();

    write_zobrist(&mut z);
}
