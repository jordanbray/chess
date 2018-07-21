// This file generates 3 giant files, magic_gen.rs and zobrist_gen.rs
// The purpose of this file is to create lookup tables that can be used during move generation.
// This file has gotten pretty long and complicated, but hopefully the comments should allow

#![allow(dead_code)]

// it to be easily followed.
extern crate rand;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use rand::{Rng, thread_rng, weak_rng, SeedableRng};
use std::arch::x86_64:: _pext_u64;
// we use the same types as the rest of the library.
mod bitboard;
use bitboard::{BitBoard, EMPTY};

mod square;
use square::{Square, NUM_SQUARES, ALL_SQUARES};

mod rank;
use rank::Rank;

mod file;
use file::{File as ChessFile, NUM_FILES};

mod color;
pub use color::{Color, ALL_COLORS, NUM_COLORS};

mod piece;
use piece::NUM_PIECES;

mod castle_rights;
use castle_rights::NUM_CASTLE_RIGHTS;

// This structure is for the "Magic Bitboard" generation
#[derive(Copy, Clone)]
struct Magic {
    magic_number: BitBoard,
    mask: BitBoard,
    offset: u32,
    rightshift: u8,
}

#[derive(Copy, Clone)]
struct BmiMagic {
    blockers_mask: BitBoard,
    rays: BitBoard,
    offset: u32,
}

// These numbers allow you to hash a set of blocking pieces, and get an index in the MOVES
// array to return the valid moves, given a set of blocking pieces.
// This will be generated here, but then put into the magic_gen.rs as a const array.
static mut MAGIC_NUMBERS: [[Magic; NUM_SQUARES]; 2 ] =
        [[Magic { magic_number: EMPTY, mask: EMPTY, offset: 0, rightshift: 0 }; 64]; 2];

// How many squares can a blocking piece be on for the rook?
const ROOK_BITS: usize = 12;
// How many squares can a blocking piece be on for a bishop?
const BISHOP_BITS: usize = 9;
// How many different sets of moves for both rooks and bishops are there?
const NUM_MOVES: usize = 64 * (1<<ROOK_BITS) /* Rook Moves */ +
                         64 * (1<<BISHOP_BITS) /* Bishop Moves */;
static mut GENERATED_NUM_MOVES: usize = 0;

// This is the valid move lookup table.  This will be generated here, then put into
// the magic_gen.rs as a const array.
static mut MOVES: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

// When a MOVES bitboard is updated, update this with the rays that the MOVES bitboard
// may have set.  This helps with compressing the MOVES array.
static mut MOVE_RAYS: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

// An index for if we are looking at a rook or bishop.
const ROOK: usize = 0;
const BISHOP: usize = 1;

// Given a square, what are the valid king moves?
static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64];

// Given a square, what are the valid knight moves?
static mut KNIGHT_MOVES: [BitBoard; 64] = [EMPTY; 64];

// Given a square, what are the valid quiet pawn moves (non-captures)?
static mut PAWN_MOVES: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// Given a square, what are the pawn attacks (captures)?
static mut PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// Given two squares, lookup a line going through those two squares, or return EMPTY.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut LINE: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];

// Given two squares, lookup a line between those two squares, or return EMPTY.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut BETWEEN: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];

// Given a square and a piece type (rook or bishop only), what are the squares they
// would attack if no pieces were on the board?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut RAYS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// What are the EDGES of the board?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut EDGES: BitBoard = EMPTY;

// Given a rank, what squares are on that rank?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut RANKS: [BitBoard; 8] = [EMPTY; 8];

// Given a file, what squares are on that file?
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut FILES: [BitBoard; 8] = [EMPTY; 8];

// Given a file, what squares are adjacent to that file?  Useful for detecting passed pawns.
// This will be generated here, and then put into the magic_gen.rs as a const array.
static mut ADJACENT_FILES: [BitBoard; 8] = [EMPTY; 8];

static mut BISHOP_BMI_MASK: [BmiMagic; 64] =
    [BmiMagic { blockers_mask: EMPTY, rays: EMPTY, offset: 0 }; 64];

static mut ROOK_BMI_MASK: [BmiMagic; 64] = 
    [BmiMagic { blockers_mask: EMPTY, rays: EMPTY, offset: 0 }; 64];

static mut BMI_MOVES: [u16; NUM_MOVES] = [0; NUM_MOVES];
static mut GENERATED_BMI_MOVES: usize = 0;

// For each square, generate the RAYS for the bishop.
fn gen_bishop_rays() {
   for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[BISHOP][src.to_index()] =
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                (src_rank - dest_rank).abs() == (src_file - dest_file).abs() &&
                                    *src != **dest})
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// For each square, generate the RAYS for the rook.
fn gen_rook_rays() {
   for src in ALL_SQUARES.iter() {
        unsafe {
            RAYS[ROOK][src.to_index()] =
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index();
                                let src_file = src.get_file().to_index();
                                let dest_rank = dest.get_rank().to_index();
                                let dest_file = dest.get_file().to_index();

                                (src_rank == dest_rank || src_file == dest_file) &&
                                    *src != **dest})
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// Generate the edges of the board as a BitBoard
fn gen_edges() -> BitBoard {
    ALL_SQUARES.iter()
               .filter(|sq| sq.get_rank() == Rank::First ||
                            sq.get_rank() == Rank::Eighth ||
                            sq.get_file() == ChessFile::A ||
                            sq.get_file() == ChessFile::H)
               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
                            
}

// Generate the LINES array.
fn gen_lines() {
    for src in ALL_SQUARES.iter() {
        for dest in ALL_SQUARES.iter() {
            unsafe {
                LINE[src.to_index()][dest.to_index()] =
                    ALL_SQUARES.iter()
                               .filter(|test| {
                                    let src_rank = src.get_rank().to_index() as i8;
                                    let src_file = src.get_file().to_index() as i8;
                                    let dest_rank = dest.get_rank().to_index() as i8;
                                    let dest_file = dest.get_file().to_index() as i8;
                                    let test_rank = test.get_rank().to_index() as i8;
                                    let test_file = test.get_file().to_index() as i8;

                                    // test diagonals first
                                    if (src_rank - dest_rank).abs() ==
                                        (src_file - dest_file).abs() &&
                                        *src != *dest {
                                        (src_rank - test_rank).abs() ==
                                            (src_file - test_file).abs() &&
                                        (dest_rank - test_rank).abs() ==
                                            (dest_file - test_file).abs()
                                    // next, test rank/file lines
                                    } else if (src_rank == dest_rank || src_file == dest_file) &&
                                               *src != *dest {
                                        (src_rank == test_rank && dest_rank == test_rank) ||
                                        (src_file == test_file && dest_file == test_file)
                                    // if src and dest don't line up, there is no line.  Return
                                    // EMPTY
                                    } else {
                                        false
                                    }
                               })
                               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
            }
        }
    }
}

// Generate the PAWN_MOVES array.
fn gen_pawn_moves() {
    for color in ALL_COLORS.iter() {
        for src in ALL_SQUARES.iter() {
            unsafe {
                if src.get_rank() == color.to_second_rank() {
                    PAWN_MOVES[color.to_index()][src.to_index()] = BitBoard::from_square(src.uforward(*color)) ^ BitBoard::from_square(src.uforward(*color).uforward(*color));
                } else {
                    match src.forward(*color) {
                        None => PAWN_MOVES[color.to_index()][src.to_index()] = EMPTY,
                        Some(x) => PAWN_MOVES[color.to_index()][src.to_index()] = BitBoard::from_square(x)
                    };
                }
            }
        }
    }
}

// Generate the PAWN_ATTACKS array.
fn gen_pawn_attacks() {
    for color in ALL_COLORS.iter() {
        for src in ALL_SQUARES.iter() {
            unsafe {
                PAWN_ATTACKS[color.to_index()][src.to_index()] = EMPTY;
                match src.forward(*color) {
                    None => {},
                    Some(x) => {
                        match x.left() {
                            None => {},
                            Some(y) => PAWN_ATTACKS[color.to_index()][src.to_index()] ^= BitBoard::from_square(y)
                        };
                        match x.right() {
                            None => {},
                            Some(y) => PAWN_ATTACKS[color.to_index()][src.to_index()] ^= BitBoard::from_square(y)
                        };
                    }
                };
            }
        }
    }
}

// Is a number (t) between two numbers (a and b)?
fn between(a: i8, t: i8, b: i8) -> bool {
    if a < b {
        a < t && t < b
    } else {
        b < t && t < a
    }
}

// Generate the BETWEEN array.
fn gen_between() {
    for src in ALL_SQUARES.iter() {
        for dest in ALL_SQUARES.iter() {
            unsafe {
                BETWEEN[src.to_index()][dest.to_index()] =
                    ALL_SQUARES.iter()
                               .filter(|test| {
                                    let src_rank = src.get_rank().to_index() as i8;
                                    let src_file = src.get_file().to_index() as i8;
                                    let dest_rank = dest.get_rank().to_index() as i8;
                                    let dest_file = dest.get_file().to_index() as i8;
                                    let test_rank = test.get_rank().to_index() as i8;
                                    let test_file = test.get_file().to_index() as i8;

                                    // test diagonals first, as above
                                    if (src_rank - dest_rank).abs() ==
                                        (src_file - dest_file).abs() &&
                                        *src != *dest {
                                        (src_rank - test_rank).abs() ==
                                            (src_file - test_file).abs() &&
                                        (dest_rank - test_rank).abs() ==
                                            (dest_file - test_file).abs() &&
                                        between(src_rank, test_rank, dest_rank)
                                    } else if (src_rank == dest_rank || src_file == dest_file) &&
                                               *src != *dest {
                                        (src_rank == test_rank && dest_rank == test_rank && between(src_file, test_file, dest_file)) ||
                                        (src_file == test_file && dest_file == test_file && between(src_rank, test_rank, dest_rank))
                                    } else {
                                        false
                                    }
                               })
                               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
            }
        }
    }
}

// Generate the KNIGHT_MOVES array.
fn gen_knight_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KNIGHT_MOVES[src.to_index()] = 
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                ((src_rank - dest_rank).abs() == 2 &&
                                 (src_file - dest_file).abs() == 1) ||
                                ((src_rank - dest_rank).abs() == 1 &&
                                 (src_file - dest_file).abs() == 2)
                           })
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// Generate the KING_MOVES array.
fn gen_king_moves() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            KING_MOVES[src.to_index()] = 
                ALL_SQUARES.iter()
                           .filter(|dest| {
                                let src_rank = src.get_rank().to_index() as i8;
                                let src_file = src.get_file().to_index() as i8;
                                let dest_rank = dest.get_rank().to_index() as i8;
                                let dest_file = dest.get_file().to_index() as i8;

                                ((src_rank - dest_rank).abs() == 1 || (src_rank - dest_rank).abs() == 0) &&
                                ((src_file - dest_file).abs() == 1 || (src_file - dest_file).abs() == 0) &&
                                *src != **dest
                           })
                           .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}


// Given a bitboard, generate a list of every possible set of bitboards using those bits.
// AKA, if 'n' bits are set, generate 2^n bitboards where b1|b2|b3|...b(2^n) == mask
fn rays_to_questions(mask: BitBoard) -> Vec<BitBoard> {
    let mut result = vec!();
    let squares = mask.collect::<Vec<_>>();

    for i in 0..(1u64<<mask.popcnt()) {
        let mut current = EMPTY;
        for j in 0..mask.popcnt() {
            if (i & (1u64 << j)) == (1u64 << j) {
                current |= BitBoard::from_square(squares[j as usize]);
            }
        }
        result.push(current);
    }

    result
}

// Return a list of directions for the rook.
fn rook_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn left(sq: Square) -> Option<Square> { sq.left() }
    fn right(sq: Square) -> Option<Square> { sq.right() }
    fn up(sq: Square) -> Option<Square> { sq.up() }
    fn down(sq: Square) -> Option<Square> { sq.down() }

    vec![left, right, up, down]
}

// Return a list of directions for the bishop.
fn bishop_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn nw(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.up()) }
    fn ne(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.up()) }
    fn sw(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.down()) }
    fn se(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.down()) }

    vec![nw, ne, sw, se]
}

// Given a square and the type of piece, lookup the RAYS and remove the endpoint squares.
fn magic_mask(sq: Square, bishop_or_rook: usize) -> BitBoard {
    unsafe { RAYS[bishop_or_rook][sq.to_index()] & 
        if bishop_or_rook == BISHOP {
            !gen_edges()
        } else {
            !ALL_SQUARES.iter()
                        .filter(|edge| (sq.get_rank() == edge.get_rank() &&
                                        (edge.get_file() == ChessFile::A || edge.get_file() == ChessFile::H)) ||
                                       (sq.get_file() == edge.get_file() &&
                                        (edge.get_rank() == Rank::First || edge.get_rank() == Rank::Eighth)))
                        .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
        }
    }
}

// Generate all the possible combinations of blocking pieces for the rook/bishop, and then
// generate all possible moves for each set of blocking pieces.
fn questions_and_answers(sq: Square, bishop_or_rook: usize) -> (Vec<BitBoard>, Vec<BitBoard>) {
    let mask = magic_mask(sq, bishop_or_rook);
    let questions = rays_to_questions(mask);

    let mut answers = vec!();

    let movement = if bishop_or_rook == BISHOP { bishop_directions() } else { rook_directions() };

    for question in questions.iter() {
        let mut answer = EMPTY;
        for m in movement.iter() {
            let mut next = m(sq);
            while next != None {
                answer ^= BitBoard::from_square(next.unwrap());
                if (BitBoard::from_square(next.unwrap()) & *question) != EMPTY {
                    break;
                }
                next = m(next.unwrap());
            }
        }
        answers.push(answer);
    }

    (questions, answers)
}

// generate lookup tables for the pdep and pext bmi2 extensions
fn generate_bmis(sq: Square, bishop_or_rook: usize, cur_offset: usize) -> usize {
    let qa = questions_and_answers(sq, bishop_or_rook);
    let questions = qa.0;
    let answers = qa.1;

    let mask = magic_mask(sq, bishop_or_rook);
    let rays = unsafe { RAYS[bishop_or_rook][sq.to_index()] };

    let bmi = BmiMagic { blockers_mask: mask, rays: rays, offset: cur_offset as u32 };
    let result = cur_offset + questions.len();

    unsafe {
        if bishop_or_rook == ROOK {
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

fn gen_all_bmis() {
    let mut cur_offset = 0;
    for s in ALL_SQUARES.iter() {
        cur_offset = generate_bmis(*s, ROOK, cur_offset);
        cur_offset = generate_bmis(*s, BISHOP, cur_offset);
    }
    unsafe { GENERATED_BMI_MOVES = cur_offset; }
}

// Generate a random bitboard with a small number of bits.
fn random_bitboard<R: Rng>(rng: &mut R) -> BitBoard {
    BitBoard::new(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
}

// Find a perfect hashing function for the move generation for a particular square and piece type
// Store the resulting move array in MOVES[cur_offset...], and return the next offset
// to be used
fn generate_magic(sq: Square, bishop_or_rook: usize, cur_offset: usize) -> usize {
    let (questions, answers) = questions_and_answers(sq, bishop_or_rook);
    assert_eq!(questions.len().count_ones(), 1);
    assert_eq!(questions.len(), answers.len());
    let mask = magic_mask(sq, bishop_or_rook);

    assert_eq!(questions.iter().fold(EMPTY, |b, n| b | *n), mask);
    assert_eq!(answers.iter().fold(EMPTY, |b, n| b | *n), unsafe { RAYS[bishop_or_rook][sq.to_index()] });
    let mut new_offset = cur_offset;

    for i in 0..cur_offset {
        let mut found = true;
        for j in 0..answers.len() {
            unsafe {
                if MOVE_RAYS[i + j] & RAYS[bishop_or_rook][sq.to_index()] != EMPTY {
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

    let mut new_magic = Magic { magic_number: EMPTY, mask: mask, offset: new_offset as u32, rightshift: (questions.len().leading_zeros() + 1) as u8 };
    
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
        MAGIC_NUMBERS[bishop_or_rook][sq.to_index()] = new_magic;

        for i in 0..questions.len() {
            let j = (new_magic.magic_number * questions[i]).to_size(new_magic.rightshift);
            MOVES[(new_magic.offset as usize) + j] |= answers[i];
            MOVE_RAYS[(new_magic.offset as usize) + j] |= RAYS[bishop_or_rook][sq.to_index()]
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
fn gen_all_magic() {
    let mut cur_offset = 0;
    for bishop_or_rook in 0..2 {
        for sq in ALL_SQUARES.iter() {
            cur_offset = generate_magic(*sq, bishop_or_rook, cur_offset);
        }
    }
}

// Generate the EDGES, RANKS, FILES, and ADJACENT_FILES variables for storage in the
// magic_gen.rs file.
fn gen_bitboard_data() {
    unsafe {
        EDGES = ALL_SQUARES.iter()
                           .filter(|x| x.get_rank() == Rank::First ||
                                       x.get_rank() == Rank::Eighth ||
                                       x.get_file() == ChessFile::A ||
                                       x.get_file() == ChessFile::H)
                           .fold(EMPTY, |v, s| v | BitBoard::from_square(*s)); 
        for i in 0..8 {
            RANKS[i] = ALL_SQUARES.iter()
                                  .filter(|x| x.get_rank().to_index() == i)
                                  .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
            FILES[i] = ALL_SQUARES.iter()
                                  .filter(|x| x.get_file().to_index() == i)
                                  .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
            ADJACENT_FILES[i] = ALL_SQUARES.iter()
                                           .filter(|y| ((y.get_file().to_index() as i8) == (i as i8) - 1) ||
                                                       ((y.get_file().to_index() as i8) == (i as i8) + 1))
                                           .fold(EMPTY, |v, s| v | BitBoard::from_square(*s));
        }
    }
}

// Here is where I write all the data out to the files

// Write the PAWN_MOVES array to the specified file.
fn write_pawn_moves(f: &mut File) {
    write!(f, "const PAWN_MOVES: [[BitBoard; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", PAWN_MOVES[i][j].to_size(0)).unwrap() };
        }
        if i != 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();

}


// Write the PAWN_ATTACKS array to the specified file.
fn write_pawn_attacks(f: &mut File) {
    write!(f, "const PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", PAWN_ATTACKS[i][j].to_size(0)).unwrap() };
        }
        if i != 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();

}

// Write the LINE array to the specified file.
fn write_line(f: &mut File) {
    write!(f, "const LINE: [[BitBoard; 64]; 64] = [[\n").unwrap();
    for i in 0..64 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", LINE[i][j].to_size(0)).unwrap() };
        }
        if i != 63 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();

}

// Write the BETWEEN array to the specified file.
fn write_between(f: &mut File) {
    write!(f, "const BETWEEN: [[BitBoard; 64]; 64] = [[\n").unwrap();
    for i in 0..64 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", BETWEEN[i][j].to_size(0)).unwrap() };
        }
        if i != 63 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();
}

// Write the RAYS array to the specified file.
fn write_rays(f: &mut File) {
    write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
    write!(f, "const ROOK: usize = {};\n", ROOK).unwrap();
    write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
    write!(f, "const BISHOP: usize = {};\n", BISHOP).unwrap();
    write!(f, "#[cfg(not(target_feature=\"bmi2\"))]").unwrap();
    write!(f, "const RAYS: [[BitBoard; 64]; 2] = [[\n").unwrap();
    for i in 0..2 {
        for j in 0..64 {
            unsafe { write!(f, "    BitBoard({}),\n", RAYS[i][j].to_size(0)).unwrap() };
        }
        if i != 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n").unwrap();
}

// Write the KING_MOVES array to the specified file.
fn write_king_moves(f: &mut File) {
    write!(f, "const KING_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({}),\n", KING_MOVES[i].to_size(0)).unwrap() };
    }
    write!(f, "];\n").unwrap();
}

// Write the KNIGHT_MOVES array to the specified file.
fn write_knight_moves(f: &mut File) {
    write!(f, "const KNIGHT_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({}),\n", KNIGHT_MOVES[i].to_size(0)).unwrap() };
    }
    write!(f, "];\n").unwrap();
}

// Write the MAGIC_NUMBERS and MOVES arrays to the specified file.
fn write_magic(f: &mut File) {
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
        for j in 0 ..64 {
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

// Write the EDGES variable to a file.
fn write_edges(f: &mut File) {
    unsafe {
        write!(f, "/// What are all the edge squares on the `BitBoard`?\n").unwrap();
        write!(f, "pub const EDGES: BitBoard = BitBoard({});\n", EDGES.to_size(0)).unwrap();
    }
}

// Write the FILES array to the specified file.
fn write_files(f: &mut File) {
    unsafe {
        write!(f, "const FILES: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", FILES[i].to_size(0)).unwrap();
        }
        write!(f, "];\n").unwrap();
    }
}

// Write the ADJACENT_FILES array to the specified file.
fn write_adjacent_files(f: &mut File) {
    unsafe {
        write!(f, "const ADJACENT_FILES: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", ADJACENT_FILES[i].to_size(0)).unwrap();
        }
        write!(f, "];\n").unwrap();
    }
}

// Write the RANKS array to the specified file.
fn write_ranks(f: &mut File) {
    unsafe {
        write!(f, "const RANKS: [BitBoard; 8] = [\n").unwrap();
        for i in 0..8 {
            write!(f, "    BitBoard({}),\n", RANKS[i].to_size(0)).unwrap();
        }
        write!(f, "];\n").unwrap();
    }
}

// write the ZOBRIEST_* arrays to a file.  I don't generate it, because its just
// a bunch of random u64s
fn write_zobrist(f: &mut File) {
    let mut rng = weak_rng();
    rng.reseed([0xDEADBEEF, 0xBEEFDEAD, 0xABCDEFAB, 0x12345678]);

    write!(f, "const SIDE_TO_MOVE: u64 = {};\n\n", rng.next_u64()).unwrap();

    write!(f, "const ZOBRIST_PIECES: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[[\n").unwrap();
    for i in 0..NUM_COLORS {
        for j in 0..NUM_PIECES {
            for _ in 0..NUM_SQUARES {
                write!(f, "    {},\n", rng.next_u64()).unwrap();
            }
            if j != NUM_PIECES - 1 {
                write!(f, "   ], [\n").unwrap();
            }
        }
        if i != NUM_COLORS - 1 {
            write!(f, "  ]], [[\n").unwrap();
        }
    }
    write!(f, "]]];\n\n").unwrap();

    write!(f, "const ZOBRIST_CASTLES: [[u64; NUM_CASTLE_RIGHTS]; NUM_COLORS] = [[\n").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..NUM_CASTLE_RIGHTS {
            write!(f, "    {},\n", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            write!(f, "  ], [\n").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();

    write!(f, "const ZOBRIST_EP: [[u64; NUM_FILES]; NUM_COLORS] = [[\n").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..NUM_FILES {
            write!(f, "    {},\n", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            write!(f, "], [\n").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();
}

fn write_bmis(f: &mut File) {
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
        write!(f, "    BmiMagic {{ blockers_mask: BitBoard({}),\n", bmi.blockers_mask.0).unwrap();
        write!(f, "                rays: BitBoard({}),\n", bmi.rays.0).unwrap();
        write!(f, "                offset: {} }},\n", bmi.offset).unwrap();
    }
    write!(f, "];\n").unwrap();

    write!(f, "#[cfg(target_feature=\"bmi2\")]").unwrap();
    write!(f, "const BISHOP_BMI_MASK: [BmiMagic; 64] = [\n").unwrap();
    for i in 0..NUM_SQUARES {
        let bmi = unsafe { BISHOP_BMI_MASK[i] };
        write!(f, "    BmiMagic {{ blockers_mask: BitBoard({}),\n", bmi.blockers_mask.0).unwrap();
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

// Generate everything.
fn main() {
    gen_lines();
    gen_between();
    gen_bishop_rays();
    gen_rook_rays();
    gen_knight_moves();
    gen_king_moves();
    gen_pawn_attacks();
    gen_pawn_moves();
    gen_all_magic();
    gen_bitboard_data();
    gen_all_bmis();

    let out_dir = env::var("OUT_DIR").unwrap();
    let magic_path = Path::new(&out_dir).join("magic_gen.rs");
    let mut f = File::create(&magic_path).unwrap();

    write_king_moves(&mut f);
    write_knight_moves(&mut f);
    write_rays(&mut f);
    write_between(&mut f);
    write_line(&mut f);
    write_pawn_attacks(&mut f);
    write_pawn_moves(&mut f);
    write_magic(&mut f);
    write_edges(&mut f);
    write_ranks(&mut f);
    write_files(&mut f);
    write_adjacent_files(&mut f);
    write_bmis(&mut f);

    let zobrist_path = Path::new(&out_dir).join("zobrist_gen.rs");
    let mut z = File::create(&zobrist_path).unwrap();

    write_zobrist(&mut z);    
}

