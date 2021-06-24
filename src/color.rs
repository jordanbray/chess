use crate::rank::Rank;
use std::ops::Not;

/// Represent a color.
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

/// How many colors are there?
pub const NUM_COLORS: usize = 2;
/// List all colors
pub const ALL_COLORS: [Color; NUM_COLORS] = [Color::White, Color::Black];

impl Color {
    /// Convert the `Color` to a `usize` for table lookups.
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert a `Color` to my backrank, which represents the starting rank
    /// for my pieces.
    #[inline]
    pub fn to_my_backrank(&self) -> Rank {
        match *self {
            Color::White => Rank::First,
            Color::Black => Rank::Eighth,
        }
    }

    /// Convert a `Color` to my opponents backrank, which represents the starting rank for the
    /// opponents pieces.
    #[inline]
    pub fn to_their_backrank(&self) -> Rank {
        match *self {
            Color::White => Rank::Eighth,
            Color::Black => Rank::First,
        }
    }

    /// Convert a `Color` to my second rank, which represents the starting rank for my pawns.
    #[inline]
    pub fn to_second_rank(&self) -> Rank {
        match *self {
            Color::White => Rank::Second,
            Color::Black => Rank::Seventh,
        }
    }

    /// Convert a `Color` to my fourth rank, which represents the rank of my pawns when
    /// moving two squares forward.
    #[inline]
    pub fn to_fourth_rank(&self) -> Rank {
        match *self {
            Color::White => Rank::Fourth,
            Color::Black => Rank::Fifth,
        }
    }

    /// Convert a `Color` to my seventh rank, which represents the rank before pawn promotion.
    #[inline]
    pub fn to_seventh_rank(&self) -> Rank {
        match *self {
            Color::White => Rank::Seventh,
            Color::Black => Rank::Second,
        }
    }
}

impl Not for Color {
    type Output = Color;

    /// Get the other color.
    #[inline]
    fn not(self) -> Color {
        if self == Color::White {
            Color::Black
        } else {
            Color::White
        }
    }
}
