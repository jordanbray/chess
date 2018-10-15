use rand::thread_rng;
use std::fs::File;
use std::io::Write;

use bitboard::{BitBoard, EMPTY};
use gen_tables::magic_helpers::{magic_mask, questions_and_answers, random_bitboard, NUM_MOVES};
use gen_tables::rays::get_rays;
use piece::Piece;
use square::{Square, ALL_SQUARES, NUM_SQUARES};

// This structure is for the "Magic Bitboard" generation
#[derive(Copy, Clone)]
struct Magic {
    magic_number: BitBoard,
    mask: BitBoard,
    offset: u32,
    rightshift: u8,
}

// These numbers allow you to hash a set of blocking pieces, and get an index in the MOVES
// array to return the valid moves, given a set of blocking pieces.
// This will be generated here, but then put into the magic_gen.rs as a const array.
static mut MAGIC_NUMBERS: [[Magic; NUM_SQUARES]; 2] = [[Magic {
    magic_number: EMPTY,
    mask: EMPTY,
    offset: 0,
    rightshift: 0,
}; 64]; 2];

// How many squares can a blocking piece be on for the rook?
static mut GENERATED_NUM_MOVES: usize = 0;

// This is the valid move lookup table.  This will be generated here, then put into
// the magic_gen.rs as a const array.
static mut MOVES: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

// When a MOVES bitboard is updated, update this with the rays that the MOVES bitboard
// may have set.  This helps with compressing the MOVES array.
static mut MOVE_RAYS: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

// Find a perfect hashing function for the move generation for a particular square and piece type
// Store the resulting move array in MOVES[cur_offset...], and return the next offset
// to be used
fn generate_magic(sq: Square, piece: Piece, cur_offset: usize) -> usize {
    let (questions, answers) = questions_and_answers(sq, piece);
    assert_eq!(questions.len().count_ones(), 1);
    assert_eq!(questions.len(), answers.len());
    let mask = magic_mask(sq, piece);

    assert_eq!(questions.iter().fold(EMPTY, |b, n| b | *n), mask);
    assert_eq!(
        answers.iter().fold(EMPTY, |b, n| b | *n),
        get_rays(sq, piece)
    );
    let mut new_offset = cur_offset;

    for i in 0..cur_offset {
        let mut found = true;
        for j in 0..answers.len() {
            unsafe {
                if MOVE_RAYS[i + j] & get_rays(sq, piece) != EMPTY {
                    found = false;
                    break;
                }
            }
        }
        if found {
            new_offset = i;
            break;
        }
    }

    let mut new_magic = Magic {
        magic_number: EMPTY,
        mask: mask,
        offset: new_offset as u32,
        rightshift: (questions.len().leading_zeros() + 1) as u8,
    };

    let mut done = false;
    let mut rng = thread_rng();

    while !done {
        let magic_bitboard = random_bitboard(&mut rng);

        if (mask * magic_bitboard).popcnt() < 6 {
            continue;
        }

        let mut new_answers = vec![EMPTY; questions.len()];
        done = true;
        for i in 0..questions.len() {
            let j = (magic_bitboard * questions[i]).to_size(new_magic.rightshift);
            if new_answers[j] == EMPTY || new_answers[j] == answers[i] {
                new_answers[j] = answers[i];
            } else {
                done = false;
                break;
            }
        }
        if done {
            new_magic.magic_number = magic_bitboard;
        }
    }

    unsafe {
        MAGIC_NUMBERS[if piece == Piece::Rook { 0 } else { 1 }][sq.to_index()] = new_magic;

        for i in 0..questions.len() {
            let j = (new_magic.magic_number * questions[i]).to_size(new_magic.rightshift);
            MOVES[(new_magic.offset as usize) + j] |= answers[i];
            MOVE_RAYS[(new_magic.offset as usize) + j] |= get_rays(sq, piece);
        }
        if new_offset + questions.len() < cur_offset {
            new_offset = cur_offset;
        } else {
            new_offset += questions.len();
        }
        GENERATED_NUM_MOVES = new_offset;
        new_offset
    }
}

// Generate the magic each square for both rooks and bishops.
pub fn gen_all_magic() {
    let mut cur_offset = 0;
    for piece in [Piece::Bishop, Piece::Rook].iter() {
        for sq in ALL_SQUARES.iter() {
            cur_offset = generate_magic(*sq, *piece, cur_offset);
        }
    }
}

// Write the MAGIC_NUMBERS and MOVES arrays to the specified file.
pub fn write_magic(f: &mut File) {
    write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
    write!(f, "#[derive(Copy, Clone)]\n").unwrap();
    write!(f, "struct Magic {{\n").unwrap();
    write!(f, "    magic_number: BitBoard,\n").unwrap();
    write!(f, "    mask: BitBoard,\n").unwrap();
    write!(f, "    offset: u32,\n").unwrap();
    write!(f, "    rightshift: u8\n").unwrap();
    write!(f, "}}\n\n").unwrap();

    write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
    write!(f, "const MAGIC_NUMBERS: [[Magic; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe {
                write!(f, "    Magic {{ magic_number: BitBoard({}), mask: BitBoard({}), offset: {}, rightshift: {} }},\n",
                    MAGIC_NUMBERS[i][j].magic_number.to_size(0),
                    MAGIC_NUMBERS[i][j].mask.to_size(0),
                    MAGIC_NUMBERS[i][j].offset,
                    MAGIC_NUMBERS[i][j].rightshift).unwrap();
            }
        }
        if i != 1 {
            write!(f, "], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();

    unsafe {
        write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
        write!(f, "const MOVES: [BitBoard; {}] = [\n", GENERATED_NUM_MOVES).unwrap();
        for i in 0..GENERATED_NUM_MOVES {
            write!(f, "    BitBoard({}),\n", MOVES[i].to_size(0)).unwrap();
        }
    }
    write!(f, "];\n").unwrap();
}
