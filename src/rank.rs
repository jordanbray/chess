use crate::error::Error;
use std::mem::transmute;
use std::str::FromStr;

/// Describe a rank (row) on a chess board
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
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
    #[inline]
    pub fn from_index(i: usize) -> Rank {
        unsafe { transmute((i as u8) & 7) }
    }

    #[inline]
    pub const fn from_index_const(i: usize) -> Rank {
        match i & 7 {
            0 => Rank::First,
            1 => Rank::Second,
            2 => Rank::Third,
            3 => Rank::Fourth,
            4 => Rank::Fifth,
            5 => Rank::Sixth,
            6 => Rank::Seventh,
            7 => Rank::Eighth,
            // unreachable:
            _ => Rank::First,
        }
        // unsafe { transmute((i as u8) & 7) }
    }

    /// Go one rank down.  If impossible, wrap around.
    #[inline]
    pub fn down(&self) -> Rank {
        Rank::from_index(self.to_index().wrapping_sub(1))
    }

    /// Go one file up.  If impossible, wrap around.
    #[inline]
    pub fn up(&self) -> Rank {
        Rank::from_index(self.to_index() + 1)
    }

    /// Convert this `Rank` into a `usize` between 0 and 7 (inclusive).
    #[inline]
    pub const fn to_index(&self) -> usize {
        *self as usize
    }
}

impl FromStr for Rank {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 1 {
            return Err(Error::InvalidRank);
        }
        match s.chars().next().unwrap() {
            '1' => Ok(Rank::First),
            '2' => Ok(Rank::Second),
            '3' => Ok(Rank::Third),
            '4' => Ok(Rank::Fourth),
            '5' => Ok(Rank::Fifth),
            '6' => Ok(Rank::Sixth),
            '7' => Ok(Rank::Seventh),
            '8' => Ok(Rank::Eighth),
            _ => Err(Error::InvalidRank),
        }
    }
}
