use std::mem::transmute;

/// Describe a rank (row) on a chess board
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Hash)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

/// How many ranks are there?
pub const NUM_RANKS: usize = 8;

/// Enumerate all ranks
pub const ALL_RANKS: [Rank; NUM_RANKS] = [
    Rank::First,
    Rank::Second,
    Rank::Third,
    Rank::Fourth,
    Rank::Fifth,
    Rank::Sixth,
    Rank::Seventh,
    Rank::Eighth,
];

impl Rank {
    /// Convert a `usize` into a `Rank` (the inverse of to_index).  If the number is > 7, wrap
    /// around.
    pub fn from_index(i: usize) -> Rank {
        unsafe { transmute((i as u8) & 7) }
    }

    /// Go one rank down.  If impossible, wrap around.
    pub fn down(&self) -> Rank {
        Rank::from_index(self.to_index().wrapping_sub(1))
    }

    /// Go one file up.  If impossible, wrap around.
    pub fn up(&self) -> Rank {
        Rank::from_index(self.to_index() + 1)
    }

    /// Convert this `Rank` into a `usize` between 0 and 7 (inclusive).
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}
