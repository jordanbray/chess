use std::arch::x86_64::_pext_u64;
use std::fs::File;
use std::io::Write;

use bitboard::{BitBoard, EMPTY};
use gen_tables::magic_helpers::{magic_mask, questions_and_answers, NUM_MOVES};
use gen_tables::rays::get_rays;
use piece::Piece;
use square::{Square, ALL_SQUARES, NUM_SQUARES};

#[derive(Copy, Clone)]
struct BmiMagic {
    blockers_mask: BitBoard,
    rays: BitBoard,
    offset: u32,
}

static mut BISHOP_BMI_MASK: [BmiMagic; 64] = [BmiMagic {
    blockers_mask: EMPTY,
    rays: EMPTY,
    offset: 0,
}; 64];

static mut ROOK_BMI_MASK: [BmiMagic; 64] = [BmiMagic {
    blockers_mask: EMPTY,
    rays: EMPTY,
    offset: 0,
}; 64];

static mut BMI_MOVES: [u16; NUM_MOVES] = [0; NUM_MOVES];

static mut GENERATED_BMI_MOVES: usize = 0;

// generate lookup tables for the pdep and pext bmi2 extensions
fn generate_bmis(sq: Square, piece: Piece, cur_offset: usize) -> usize {
    let qa = questions_and_answers(sq, piece);
    let questions = qa.0;
    let answers = qa.1;

    let mask = magic_mask(sq, piece);
    let rays = get_rays(sq, piece);

    let bmi = BmiMagic {
        blockers_mask: mask,
        rays: rays,
        offset: cur_offset as u32,
    };
    let result = cur_offset + questions.len();

    unsafe {
        if piece == Piece::Rook {
            ROOK_BMI_MASK[sq.to_index()] = bmi;
        } else {
            BISHOP_BMI_MASK[sq.to_index()] = bmi;
        }
    }

    for i in 0..questions.len() {
        let question = unsafe { _pext_u64(questions[i].0, mask.0) as usize };
        let answer = unsafe { _pext_u64(answers[i].0, rays.0) as u16 };
        unsafe {
            BMI_MOVES[cur_offset + question] = answer;
        }
    }

    return result;
}

pub fn gen_all_magic() {
    let mut cur_offset = 0;
    for s in ALL_SQUARES.iter() {
        cur_offset = generate_bmis(*s, Piece::Rook, cur_offset);
        cur_offset = generate_bmis(*s, Piece::Bishop, cur_offset);
    }
    unsafe {
        GENERATED_BMI_MOVES = cur_offset;
    }
}

pub fn write_magic(f: &mut File) {
    write!(f, "#[cfg(target_feature=\"bmi2\")]").unwrap();
    write!(f, "#[derive(Copy, Clone)]\n").unwrap();
    write!(f, "struct BmiMagic {{\n").unwrap();
    write!(f, "    blockers_mask: BitBoard,\n").unwrap();
    write!(f, "    rays: BitBoard,\n").unwrap();
    write!(f, "    offset: u32,\n").unwrap();
    write!(f, "}}\n\n").unwrap();

    write!(f, "#[cfg(target_feature=\"bmi2\")]").unwrap();
    write!(f, "const ROOK_BMI_MASK: [BmiMagic; 64] = [\n").unwrap();
    for i in 0..NUM_SQUARES {
        let bmi = unsafe { ROOK_BMI_MASK[i] };
        write!(
            f,
            "    BmiMagic {{ blockers_mask: BitBoard({}),\n",
            bmi.blockers_mask.0
        ).unwrap();
        write!(f, "                rays: BitBoard({}),\n", bmi.rays.0).unwrap();
        write!(f, "                offset: {} }},\n", bmi.offset).unwrap();
    }
    write!(f, "];\n").unwrap();

    write!(f, "#[cfg(target_feature=\"bmi2\")]").unwrap();
    write!(f, "const BISHOP_BMI_MASK: [BmiMagic; 64] = [\n").unwrap();
    for i in 0..NUM_SQUARES {
        let bmi = unsafe { BISHOP_BMI_MASK[i] };
        write!(
            f,
            "    BmiMagic {{ blockers_mask: BitBoard({}),\n",
            bmi.blockers_mask.0
        ).unwrap();
        write!(f, "                rays: BitBoard({}),\n", bmi.rays.0).unwrap();
        write!(f, "                offset: {} }},\n", bmi.offset).unwrap();
    }
    write!(f, "];\n").unwrap();

    let moves = unsafe { GENERATED_BMI_MOVES };
    write!(f, "#[cfg(target_feature=\"bmi2\")]").unwrap();
    write!(f, "const BMI_MOVES: [u16; {}] = [\n", moves).unwrap();

    for i in 0..moves {
        write!(f, "    {},\n", unsafe { BMI_MOVES[i] }).unwrap();
    }
    write!(f, "];\n\n").unwrap();
}
