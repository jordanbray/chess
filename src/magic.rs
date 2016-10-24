use bitboard::{BitBoard, EMPTY, get_rank, get_adjacent_files};
use square::{Square, NUM_SQUARES, ALL_SQUARES};
use color::{Color, ALL_COLORS};
use rand::{Rng, thread_rng};
use std::sync::{Once, ONCE_INIT};

#[derive(Copy, Clone)]
struct Magic {
    magic_number: BitBoard,
    mask: BitBoard,
    offset: u32,
    rightshift: u8,
}

// the following things are just for move generation
const ROOK: usize = 0;
const BISHOP: usize = 1;

static mut MAGIC_NUMBERS: [[Magic; NUM_SQUARES]; 2 ] =
        [[Magic { magic_number: EMPTY, mask: EMPTY, offset: 0, rightshift: 0 }; 64]; 2];

const ROOK_BITS: usize = 12;
const BISHOP_BITS: usize = 9;
const NUM_MOVES: usize = 64 * (1<<ROOK_BITS) /* Rook Moves */ +
                         64 * (1<<BISHOP_BITS) /* Bishop Moves */;

static mut MOVES: [BitBoard; NUM_MOVES] = [EMPTY; NUM_MOVES];

static mut KING_MOVES: [BitBoard; 64] = [EMPTY; 64];
static mut KNIGHT_MOVES: [BitBoard; 64] = [EMPTY; 64];
static mut PAWN_MOVES: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];
static mut PAWN_ATTACKS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

// the following are helper variables to cache regularly-used values
static mut LINE: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];
static mut BETWEEN: [[BitBoard; 64]; 64] = [[EMPTY; 64]; 64];
static mut RAYS: [[BitBoard; 64]; 2] = [[EMPTY; 64]; 2];

static SETUP: Once = ONCE_INIT;

/// Initialize all the magic numbers and lookup tables.
/// Note: You want to call construct::construct() instead.  It's easier, and you must call
/// BitBoard::construct() before calling this, so just rely on the other one.
pub fn construct() {
    SETUP.call_once(|| {
        let mut index: usize = 0;
        for sq in ALL_SQUARES.iter() {
            index = gen_rooks(*sq, index);
        }
        for sq in ALL_SQUARES.iter() {
            index = gen_bishops(*sq, index);
        }

        for sq1 in ALL_SQUARES.iter() {
            for sq2 in ALL_SQUARES.iter() {
                gen_line_and_between(*sq1, *sq2);
            }
        }

        for sq in ALL_SQUARES.iter() {
            gen_king_moves(*sq);
            gen_knight_moves(*sq);
        }

        gen_pawn_moves();
        gen_pawn_attacks();
    });
}

/// Get the rays for a bishop on a particular square.
pub fn get_bishop_rays(sq: Square) -> BitBoard {
    unsafe {
        *RAYS.get_unchecked(BISHOP).get_unchecked(sq.to_int() as usize)
    }
}

/// Get the rays for a rook on a particular square.
pub fn get_rook_rays(sq: Square) -> BitBoard {
    unsafe {
        *RAYS.get_unchecked(ROOK).get_unchecked(sq.to_int() as usize)
    }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
pub fn get_rook_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS.get_unchecked(ROOK).get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked((magic.offset as usize) + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift))
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
pub fn get_bishop_moves(sq: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS.get_unchecked(BISHOP).get_unchecked(sq.to_int() as usize);
        *MOVES.get_unchecked((magic.offset as usize) + (magic.magic_number * (blockers & magic.mask)).to_size(magic.rightshift))
    }
}

/// Get the king moves for a particular square.
pub fn get_king_moves(sq: Square) -> BitBoard {
    unsafe {
        *KING_MOVES.get_unchecked(sq.to_index())
    }
}

/// Get the knight moves for a particular square.
pub fn get_knight_moves(sq: Square) -> BitBoard {
    unsafe {
        *KNIGHT_MOVES.get_unchecked(sq.to_index())
    }
}

/// Get the pawn capture move for a particular square, given the pawn's color and the potential
/// victims
pub fn get_pawn_attacks(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        *PAWN_ATTACKS.get_unchecked(color.to_index()).get_unchecked(sq.to_index()) & blockers
    }
}

/// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
/// the potential blocking pieces.
pub fn get_pawn_quiets(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        if (BitBoard::from_square(sq.uforward(color)) & blockers) != EMPTY {
            EMPTY
        } else {
            *PAWN_MOVES.get_unchecked(color.to_index()).get_unchecked(sq.to_index()) & !blockers
        }
    }
}

/// Get all the pawn moves for a particular square, given the pawn's color and the potential
/// blocking pieces and victims.
pub fn get_pawn_moves(sq: Square, color: Color, blockers: BitBoard) -> BitBoard {
    get_pawn_attacks(sq, color, blockers) ^ get_pawn_quiets(sq, color, blockers)
}

/// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
/// This line does extend past the squares.
pub fn line(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *LINE.get_unchecked(sq1.to_index()).get_unchecked(sq2.to_index())
    }
}

/// Get a line between these two squares, not including the squares themselves.
pub fn between(sq1: Square, sq2: Square) -> BitBoard {
    unsafe {
        *BETWEEN.get_unchecked(sq1.to_index()).get_unchecked(sq2.to_index())
    }
}

/// generate a random bitboard with few bits
fn random_bitboard<R: Rng>(rng: &mut R) -> BitBoard {
    BitBoard::new(rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>())
}

/// guess and check to find a hashing function which can map questions directly onto answers
/// store said hashing fuction at result_index inside MOVES
fn genmagic(questions: &Vec<BitBoard>, answers: &Vec<BitBoard>, result_index: usize) -> Option<Magic> {
    let mut max_guess = 1000000000i64;
    let mut rng = thread_rng();

    let length = questions.len();
    let rightshift = (length.leading_zeros() + 1) as u8;
    let max_write = result_index + length;

    let mask = questions.iter().fold(EMPTY, |cur, next| cur | *next);

    while max_guess > 0 {
        // make a guess
        let guess = random_bitboard(&mut rng);
        if (mask * guess).popcnt() < 6 {
            continue;
        }

        // magic is *NOT* safe, kids
        unsafe {
            // zero out the table we want to write to
            for i in result_index..max_write {
                MOVES[i] = EMPTY;
            }

            // let's be optimistic
            let mut success = true;

            // lets make a fancy iterator, too
            let it = questions.iter().zip(answers.iter());

            // see if this guess can be used to create a lookup table
            for (question, answer) in it {
                // create an index using our fancy hashing algorithm
                // (multiplication)
                let index: usize = (guess * *question).to_size(rightshift);

                // if the index is empty, great.  Add this answer to that index
                if MOVES[result_index + index] == EMPTY {
                    MOVES[result_index + index] = *answer;
                    // if not, see if the answer matches whats already in the table
                } else if MOVES[result_index + index] != *answer {
                    // no? This guess fails then
                    success = false;
                    break;
                }
            }
            if success {
                return Some(Magic { magic_number: guess, mask: mask, offset: result_index as u32, rightshift: rightshift });
            }
        }
        
        max_guess -= 1;
    }

    None // Magic isn't real :(
}

/// Given:
///  * a starting square,
///  * some pieces that may block me,
///  * some `directions` (which are functions that take in a square, and give a new square in a particular direction),
///  * a boolean to include the edge moves (destination moves where the next move is off the
///    board)
/// Generate a BitBoard for all valid sliding moves in those directions, from that square
fn gen_rays(sq: Square, blockers: BitBoard, directions: &Vec<fn(Square) -> Option<Square>>, include_edges: bool) -> BitBoard {
    let mut rays: BitBoard = EMPTY;

    for x in directions.iter() {
        let mut cur_square = sq;
        loop {
            match x(cur_square) {
                None => break,
                Some(v) => {
                    if !include_edges {
                        cur_square = v;
                        if BitBoard::from_square(cur_square) & blockers == BitBoard::from_square(cur_square) || x(v) == None{
                            break;
                        }
                        rays |= BitBoard::from_square(cur_square);
                    } else {
                        cur_square = v;
                        rays |= BitBoard::from_square(cur_square);
                        if BitBoard::from_square(cur_square) & blockers == BitBoard::from_square(cur_square) {
                            break;
                        }
                    }
                }
            }
        }
    }

    rays
}

/// Generate the 'quiet' (non-capture) pawn moves for every square.  Ignore potential blockers.
fn gen_pawn_moves() {
    unsafe {
        for sq in ALL_SQUARES.iter() {
            for c in ALL_COLORS.iter() {
                if (BitBoard::from_square(*sq) & get_rank(c.to_their_backrank())) != EMPTY {
                    continue;
                } else if sq.get_rank() == c.to_second_rank() {
                    PAWN_MOVES[c.to_index()][sq.to_index()] = BitBoard::from_square(sq.uforward(*c).uforward(*c));
                }
                PAWN_MOVES[c.to_index()][sq.to_index()] ^= BitBoard::from_square(sq.uforward(*c));
            }
        }
    }
}

/// Generate the capture pawn moves for every square.
fn gen_pawn_attacks() {
    unsafe {
        for sq in ALL_SQUARES.iter() {
            for c in ALL_COLORS.iter() {
                if (BitBoard::from_square(*sq) & get_rank(c.to_their_backrank())) != EMPTY {
                    continue;
                }
                PAWN_ATTACKS[c.to_index()][sq.to_index()] = get_rank(sq.uforward(*c).get_rank()) &
                                                            get_adjacent_files(sq.get_file());
            }
        }
    }
}

/// Given some 'rays' (lines in a particular set of directions), generate a list of potential
/// blocking piece configurations.  AKA, if I'm going directly up, generate EVERY piece
/// configuration where my opponent may block my sliding up.
///
/// Given 'n' bits set, this returns 2^n new `BitBoard`s.
fn permute(rays: BitBoard) -> Vec<BitBoard> {
    let squares = rays.collect::<Vec<Square>>();

    let count: u64 = 1u64 << squares.len();

    let mut result: Vec<BitBoard> = Vec::new();
    for x in 0..count {
        let mut next_bb: BitBoard = EMPTY;

        for y in 0..squares.len() {
            if (x & (1<<y)) == (1<<y) {
                next_bb |= BitBoard::from_square(squares[y]);
            }
        }
        result.push(next_bb);

    }

    result
}

/// Given:
///  * A piece_type (BISHOP or ROOK),
///  * a square,
///  * the directions that piece can travel in,
///  * and a starting index for writing
/// Generate:
///  * the RAYS for that piece (moves which would be valid for no blocking pieces).
///  * the MAGIC_NUMBER for that piece.
/// Return:
///  * The new index to write to for the next piece.
fn save_magic(piece_type: usize, sq: Square, directions: &Vec<fn(Square) -> Option<Square>>, result_index: usize) -> usize {
    unsafe {
        RAYS[piece_type][sq.to_int() as usize] = gen_rays(sq, EMPTY, directions, true);
    }

    let questions = permute(gen_rays(sq, EMPTY, directions, false));

    let answers = questions.iter().map(|bb| gen_rays(sq, *bb, directions, true)).collect::<Vec<BitBoard>>();

    let length = questions.len();

    if length.count_ones() != 1 {
        panic!("Length of questions is invalid.");
    }

    unsafe {
        MAGIC_NUMBERS[piece_type][sq.to_int() as usize] = genmagic(&questions, &answers, result_index).unwrap();
    }

    result_index + length
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

/// Return a list of directions for the knight.
fn knight_directions() -> Vec<fn(Square) -> Option<Square>> {
    fn d1(sq: Square) -> Option<Square> { sq.up().map_or(None, |s| s.up()).map_or(None, |s| s.left()) }
    fn d2(sq: Square) -> Option<Square> { sq.up().map_or(None, |s| s.up()).map_or(None, |s| s.right()) }
    fn d3(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.left()).map_or(None, |s| s.up()) }
    fn d4(sq: Square) -> Option<Square> { sq.left().map_or(None, |s| s.left()).map_or(None, |s| s.down()) }
    fn d5(sq: Square) -> Option<Square> { sq.down().map_or(None, |s| s.down()).map_or(None, |s| s.left()) }
    fn d6(sq: Square) -> Option<Square> { sq.down().map_or(None, |s| s.down()).map_or(None, |s| s.right()) }
    fn d7(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.right()).map_or(None, |s| s.down()) }
    fn d8(sq: Square) -> Option<Square> { sq.right().map_or(None, |s| s.right()).map_or(None, |s| s.up()) }

    vec![d1, d2, d3, d4, d5, d6, d7, d8]
}

/// Return all rook and bishop directions.
fn all_directions() -> Vec<fn(Square) -> Option<Square>> {
    let mut v = Vec::new();
    v.extend(rook_directions());
    v.extend(bishop_directions());
    v
}

/// Generate the rook magic number for a particular square
fn gen_rooks(sq: Square, result_index: usize) -> usize {
    let directions: Vec<fn(Square) -> Option<Square>> = rook_directions();
    save_magic(ROOK, sq, &directions, result_index)
}

/// Generate the bishop magic number for a particular square
fn gen_bishops(sq: Square, result_index: usize) -> usize {
    let directions: Vec<fn(Square) -> Option<Square>> = bishop_directions();
    save_magic(BISHOP, sq, &directions, result_index)
}

/// Generate all king moves for a particular square.
fn gen_king_moves(sq: Square) {
    unsafe {
        KING_MOVES[sq.to_index()] = all_directions().iter()
                                                           .map(|d| d(sq))
                                                           .fold(EMPTY, |cur, os| match os {
                                                                None => cur, 
                                                                Some(sq) => cur | BitBoard::from_square(sq)
                                                            });
    }
}

/// Generate all knight moves for a particular square.
fn gen_knight_moves(sq: Square) {
    unsafe {
        KNIGHT_MOVES[sq.to_index()] = knight_directions().iter()
                                                                .map(|d| d(sq))
                                                                .fold(EMPTY, |cur, os| match os {
                                                                    None => cur,
                                                                    Some(sq) => cur | BitBoard::from_square(sq)
                                                                });
    }
}

/// generate a line from sq1 to sq2 which spans the entire chess board
/// while you're at it, generate the BETWEEN bitboard
fn gen_line_and_between(sq1: Square, sq2: Square) {
    let directions: Vec<fn(Square) -> Option<Square>> = all_directions();
    let mut rays = get_rook_rays(sq1) | get_bishop_rays(sq1);

    // short circut for non-aligned squares and equal squares
    if rays & BitBoard::from_square(sq2) == EMPTY || sq1 == sq2 {
        unsafe {
            LINE[sq1.to_index()][sq2.to_index()] = EMPTY;
            BETWEEN[sq1.to_index()][sq2.to_index()] = EMPTY;
        }
        return;
    }

    // we need to find two directions, one from sq1 to sq2, and one from sq2 to sq1
    let mut dir1: Option<fn(Square) -> Option<Square>> = None;
    let mut dir2: Option<fn(Square) -> Option<Square>> = None;


    for d in directions.iter() {
        let mut cur = sq1;
        let mut between = EMPTY;
        let mut matched = false;

        loop {
            match d(cur) {
                None => break,
                Some(sq) => {
                    if sq == sq2 { matched = true; dir1 = Some(*d); break; }
                    between |= BitBoard::from_square(sq);
                    cur = sq;
                }
            }
        }
        if matched {
            unsafe {
                BETWEEN[sq1.to_index()][sq2.to_index()] = between;
            }
        }
    }

    for d in directions.iter() {
        let mut cur = sq2;
        loop {
            match d(cur) {
                None => break,
                Some(sq) => {
                    if sq == sq1 { dir2 = Some(*d); break; }
                    cur = sq;
                }
            }
        }
    }

    // we now have our two directions, so lets iterate in both directions from sq1 and set the
    // result to LINE
    rays = BitBoard::from_square(sq1);

    let mut cur = sq1;
    loop { 
        match dir1.unwrap()(cur) {
            None => break,
            Some(sq) => { rays |= BitBoard::from_square(sq); cur = sq; }
        }
    }

    cur = sq1;
    loop {
        match dir2.unwrap()(cur) {
            None => break,
            Some(sq) => { rays |= BitBoard::from_square(sq); cur = sq; }
        }
    }

    unsafe {
        LINE[sq1.to_index()][sq2.to_index()] = rays;
        if (BETWEEN[sq1.to_index()][sq2.to_index()] & LINE[sq1.to_index()][sq2.to_index()]) != BETWEEN[sq1.to_index()][sq2.to_index()] {
            panic!();
        }
    }
}

