use square::*;
use std::ops::{BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, Mul, Not};
use std::fmt;
use std::sync::{Once, ONCE_INIT};
use rank::Rank;
use file::File;

/// A good old-fashioned bitboard
/// You do *not* have access to the actual value.  You *do* have access to operators
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct BitBoard(u64);

static SETUP: Once = ONCE_INIT;

/// An empty bitboard
pub const EMPTY: BitBoard = BitBoard(0);

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 & other.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 | other.0)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ other.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, other: BitBoard) {
        self.0 &= other.0;
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, other: BitBoard) {
        self.0 |= other.0;
    }
}

impl BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, other: BitBoard) {
        self.0 ^= other.0;
    }
}

impl Mul for BitBoard {
    type Output = BitBoard;

    fn mul(self, other: BitBoard) -> BitBoard {
        BitBoard(self.0.wrapping_mul(other.0))
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: String = "".to_owned();
        for x in 0..64 {
            if self.0 & (1u64 << x) == (1u64 << x) {
                s.push_str("X ");
            } else {
                s.push_str(". ");
            }
            if x % 8 == 7 {
                s.push_str("\n");
            }
        }
        write!(f, "{}", s)
    }
}

impl BitBoard {
    /// Construct a new bitboard from a u64
    pub fn new(b: u64) -> BitBoard {
        BitBoard(b)
    }

    /// Construct a new `BitBoard` with a particular `Square` set
    pub fn set(rank: Rank, file: File) -> BitBoard {
        BitBoard::from_square(Square::make_square(rank, file))
    }

    /// Construct a new `BitBoard` with a particular `Square` set
    pub fn from_square(sq: Square) -> BitBoard {
        BitBoard(1u64 << sq.to_int())
    }

    /// Convert an `Option<Square>` to an `Option<BitBoard>`
    pub fn from_maybe_square(sq: Option<Square>) -> Option<BitBoard> {
        sq.map(|s| BitBoard::from_square(s))
    }

    /// Convert a `BitBoard` to a `Square`.  This grabs the least-significant `Square`
    pub fn to_square(&self) -> Square {
        unsafe {
            Square::new(self.0.trailing_zeros() as u8)
        }
    }

    /// Count the number of `Squares` set in this `BitBoard`
    pub fn popcnt(&self) -> u32 {
        self.0.count_ones()
    }

    /// Reverse this `BitBoard`.  Look at it from the opponents perspective.
    pub fn reverse_colors(&self) -> BitBoard {
        BitBoard(self.0.swap_bytes())
    }

    /// Convert this `BitBoard` to a `usize` (for table lookups)
    pub fn to_size(&self, rightshift: u8) -> usize {
        (self.0 >> rightshift) as usize
    }
}

/// Get a `BitBoard` that represents all the squares on a particular rank.
pub fn get_rank(rank: Rank) -> BitBoard {
    unsafe {
        *RANKS.get_unchecked(rank.to_index())
    }
}

/// Get a `BitBoard` that represents all the squares on a particular file.
pub fn get_file(file: File) -> BitBoard {
    unsafe {
        *FILES.get_unchecked(file.to_index())
    }
}

/// Get a `BitBoard` that represents the squares on the 1 or 2 files next to this file.
pub fn get_adjacent_files(file: File) -> BitBoard {
    unsafe {
        *ADJACENT_FILES.get_unchecked(file.to_index())
    }
}

/// Perform initialization.  Must be called before some functions can be used.
pub fn construct() {
    SETUP.call_once(|| {
        unsafe {
            EDGES = ALL_SQUARES.iter()
                               .filter(|x| x.get_rank() == Rank::First ||
                                           x.get_rank() == Rank::Eighth ||
                                           x.get_file() == File::A ||
                                           x.get_file() == File::H)
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
    });
}

static mut EDGES: BitBoard = EMPTY;
static mut RANKS: [BitBoard; 8] = [EMPTY; 8];
static mut FILES: [BitBoard; 8] = [EMPTY; 8];
static mut ADJACENT_FILES: [BitBoard; 8] = [EMPTY; 8];

/// For the `BitBoard`, iterate over every `Square` set.
impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let result = self.to_square();
            *self ^= BitBoard::from_square(result);
            Some(result)
        }
    }
}

