extern crate rand;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use rand::{Rng, thread_rng};

mod bitboard;
use bitboard::{BitBoard, EMPTY};

mod square;
use square::{Square, NUM_SQUARES, ALL_SQUARES};

mod rank;
use rank::Rank;

mod file;
use file::File as ChessFile;

mod piece;
use piece::Piece;

mod color;
pub use color::{Color, ALL_COLORS};

const ROOK: usize = 0;
const BISHOP: usize = 1;

static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64]; // DONE
static mut KNIGHT_MOVES: [BitBoard; 64] = [EMPTY; 64]; // DONE
static mut PAWN_MOVES: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2]; // DONE
static mut PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2]; // DONE

// the following are helper variables to cache regularly-used values
static mut LINE: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64]; // DONE
static mut BETWEEN: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64]; // DONE
static mut RAYS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2]; // DONE

fn write_king_moves(f: &mut File) {
    write!(f, "const KING_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({})", KING_MOVES[i].to_size(0)).unwrap() };
        if i != 63 {
            write!(f, ",").unwrap();
        }
        write!(f, "\n").unwrap();
    }
    write!(f, "];\n").unwrap();
}

fn write_knight_moves(f: &mut File) {
    write!(f, "const KNIGHT_MOVES: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({})", KNIGHT_MOVES[i].to_size(0)).unwrap() };
        if i != 63 {
            write!(f, ",").unwrap();
        }
        write!(f, "\n").unwrap();
    }
    write!(f, "];\n").unwrap();
}

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

fn gen_edges() -> BitBoard {
    ALL_SQUARES.iter()
               .filter(|sq| sq.get_rank() == Rank::First ||
                            sq.get_rank() == Rank::Eighth ||
                            sq.get_file() == ChessFile::A ||
                            sq.get_file() == ChessFile::H)
               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
                            
}

fn gen_corners() -> BitBoard {
    ALL_SQUARES.iter()
               .filter(|sq| (sq.get_rank() == Rank::First ||
                             sq.get_rank() == Rank::Eighth) &&
                            (sq.get_file() == ChessFile::A ||
                             sq.get_file() == ChessFile::H))
               .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
}

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
                                    if ((src_rank - dest_rank).abs() ==
                                        (src_file - dest_file).abs() &&
                                        *src != *dest) {
                                        (src_rank - test_rank).abs() ==
                                            (src_file - test_file).abs() &&
                                        (dest_rank - test_rank).abs() ==
                                            (dest_file - test_file).abs()
                                    // next, test rank/file lines
                                    } else if ((src_rank == dest_rank || src_file == dest_file) &&
                                               *src != *dest) {
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

fn between(a: i8, t: i8, b: i8) -> bool {
    if (a < b) {
        a < t && t < b
    } else {
        b < t && t < a
    }
}

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
                                    if ((src_rank - dest_rank).abs() ==
                                        (src_file - dest_file).abs() &&
                                        *src != *dest) {
                                        (src_rank - test_rank).abs() ==
                                            (src_file - test_file).abs() &&
                                        (dest_rank - test_rank).abs() ==
                                            (dest_file - test_file).abs() &&
                                        between(src_rank, test_rank, dest_rank)
                                    } else if ((src_rank == dest_rank || src_file == dest_file) &&
                                               *src != *dest) {
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
            println!("{}", KING_MOVES[src.to_index()]);
        }
    }
}

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

/// Return a list of directions for the rook.
fn rook_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn left(sq: Square) -> Option<Square> { sq.left() }
    fn right(sq: Square) -> Option<Square> { sq.right() }
    fn up(sq: Square) -> Option<Square> { sq.up() }
    fn down(sq: Square) -> Option<Square> { sq.down() }

    vec![left, right, up, down]
}

/// Return a list of directions for the bishop.
fn bishop_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn nw(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.up()) }
    fn ne(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.up()) }
    fn sw(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.down()) }
    fn se(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.down()) }

    vec![nw, ne, sw, se]
}

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

// the following things are just for move generation
#[derive(Copy, Clone)]
struct Magic {
    magic_number: BitBoard,
    mask: BitBoard,
    offset: u32,
    rightshift: u8,
}

static mut MAGIC_NUMBERS: [[Magic; NUM_SQUARES]; 2 ] =
        [[Magic { magic_number: EMPTY, mask: EMPTY, offset: 0, rightshift: 0 }; 64]; 2];

const ROOK_BITS: usize = 12;
const BISHOP_BITS: usize = 9;
const NUM_MOVES: usize = 64 * (1<<ROOK_BITS) /* Rook Moves */ +
                         64 * (1<<BISHOP_BITS) /* Bishop Moves */;

static mut MOVES: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

/// generate a random bitboard with few bits
fn random_bitboard<R: Rng>(rng: &mut R) -> BitBoard {
    BitBoard::new(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
}


fn generate_magic(sq: Square, bishop_or_rook: usize, cur_offset: usize) -> usize {
    let (questions, answers) = questions_and_answers(sq, bishop_or_rook);
    assert_eq!(questions.len().count_ones(), 1);
    assert_eq!(questions.len(), answers.len());
    let mask = magic_mask(sq, bishop_or_rook);

    assert_eq!(questions.iter().fold(EMPTY, |b, n| b | *n), mask);
    assert_eq!(answers.iter().fold(EMPTY, |b, n| b | *n), unsafe { RAYS[bishop_or_rook][sq.to_index()] });

    let mut new_magic = Magic { magic_number: EMPTY, mask: mask, offset: cur_offset as u32, rightshift: (questions.len().leading_zeros() + 1) as u8 };
    
    let mut done = false;
    let mut rng = thread_rng();

    let mut num_tries = 1000000000;

    while !done && num_tries > 0 {
        num_tries -= 1;
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

    assert!(num_tries != 0);

    unsafe {
        MAGIC_NUMBERS[bishop_or_rook][sq.to_index()] = new_magic;

        for i in 0..questions.len() {
            let j = (new_magic.magic_number * questions[i]).to_size(new_magic.rightshift);
            MOVES[(new_magic.offset as usize) + j] = answers[i];
        }
        (new_magic.offset as usize) + questions.len()
    }
}

fn gen_all_magic() {
    let mut cur_offset = 0;
    for bishop_or_rook in 0..2 {
        for sq in ALL_SQUARES.iter() {
            cur_offset = generate_magic(*sq, bishop_or_rook, cur_offset);
            println!("Generated {}", sq.to_index() * (bishop_or_rook + 1));
        }
    }
}

fn main() {
    println!("Generating lookup tables...");
    gen_lines();
    gen_between();
    gen_bishop_rays();
    gen_rook_rays();
    gen_knight_moves();
    gen_king_moves();
    gen_pawn_attacks();
    gen_pawn_moves();
    println!("Generating magics.  This may take a while...");
    gen_all_magic();

    let out_dir = env::var("OUT_DIR").unwrap();
    let magic_path = Path::new(&out_dir).join("magic_gen.rs");
    let mut f = File::create(&magic_path).unwrap();

    write_king_moves(&mut f);
    write_knight_moves(&mut f);
}

