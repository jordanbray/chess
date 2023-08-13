use crate::bitboard::{BitBoard, EMPTY};
use crate::file::File;
use crate::gen_tables::rays::get_rays;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::{Square, ALL_SQUARES};
use rand::Rng;

// How many squares can a blocking piece be on for the rook?
const ROOK_BITS: usize = 12;
// How many squares can a blocking piece be on for a bishop?
const BISHOP_BITS: usize = 9;
// How many different sets of moves for both rooks and bishops are there?
pub const NUM_MOVES: usize = 64 * (1<<ROOK_BITS) /* Rook Moves */ +
                         64 * (1<<BISHOP_BITS) /* Bishop Moves */;

// Return a list of directions for the rook.
fn rook_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn left(sq: Square) -> Option<Square> {
        sq.left()
    }
    fn right(sq: Square) -> Option<Square> {
        sq.right()
    }
    fn up(sq: Square) -> Option<Square> {
        sq.up()
    }
    fn down(sq: Square) -> Option<Square> {
        sq.down()
    }

    vec![left, right, up, down]
}

// Return a list of directions for the bishop.
fn bishop_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn nw(sq: Square) -> Option<Square> {
        sq.left().and_then(|s| s.up())
    }
    fn ne(sq: Square) -> Option<Square> {
        sq.right().and_then(|s| s.up())
    }
    fn sw(sq: Square) -> Option<Square> {
        sq.left().and_then(|s| s.down())
    }
    fn se(sq: Square) -> Option<Square> {
        sq.right().and_then(|s| s.down())
    }

    vec![nw, ne, sw, se]
}

// Generate a random bitboard with a small number of bits.
pub fn random_bitboard<R: Rng>(rng: &mut R) -> BitBoard {
    BitBoard::new(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
}

// Given a square and the type of piece, lookup the RAYS and remove the endpoint squares.
pub fn magic_mask(sq: Square, piece: Piece) -> BitBoard {
    get_rays(sq, piece)
        & if piece == Piece::Bishop {
            !gen_edges()
        } else {
            !ALL_SQUARES
                .iter()
                .filter(|edge| {
                    (sq.get_rank() == edge.get_rank()
                        && (edge.get_file() == File::A || edge.get_file() == File::H))
                        || (sq.get_file() == edge.get_file()
                            && (edge.get_rank() == Rank::First || edge.get_rank() == Rank::Eighth))
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
        }
}

// Given a bitboard, generate a list of every possible set of bitboards using those bits.
// AKA, if 'n' bits are set, generate 2^n bitboards where b1|b2|b3|...b(2^n) == mask
fn rays_to_questions(mask: BitBoard) -> Vec<BitBoard> {
    let mut result = vec![];
    let squares = mask.collect::<Vec<_>>();

    for i in 0..(1u64 << mask.popcnt()) {
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

// Generate all the possible combinations of blocking pieces for the rook/bishop, and then
// generate all possible moves for each set of blocking pieces.
pub fn questions_and_answers(sq: Square, piece: Piece) -> (Vec<BitBoard>, Vec<BitBoard>) {
    let mask = magic_mask(sq, piece);
    let questions = rays_to_questions(mask);

    let mut answers = vec![];

    let movement = if piece == Piece::Bishop {
        bishop_directions()
    } else {
        rook_directions()
    };

    for question in questions.iter() {
        let mut answer = EMPTY;
        for m in movement.iter() {
            let mut next = m(sq);
            while next.is_some() {
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

// Generate the edges of the board as a BitBoard
fn gen_edges() -> BitBoard {
    ALL_SQUARES
        .iter()
        .filter(|sq| {
            sq.get_rank() == Rank::First
                || sq.get_rank() == Rank::Eighth
                || sq.get_file() == File::A
                || sq.get_file() == File::H
        })
        .fold(EMPTY, |b, s| b | BitBoard::from_square(*s))
}
