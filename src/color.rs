use std::ops::Not;

/// Represent a color.
#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum Color {
    White,
    Black
}

/// How many colors are there?
pub const NUM_COLORS: usize = 2;
/// List all colors
pub const ALL_COLORS: [Color; NUM_COLORS] = [Color::White, Color::Black];

impl Color {
    /// Convert the `Color` to a `usize` for table lookups.
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Covert the `Color` to a rank, which reperesnts the starting position
    /// for that colors pieces.
    pub fn to_my_backrank(&self) -> u8 {
        (self.to_index() * 7) as u8
    }

    /// Convert a `Color` to my opponents backrank, which represents the starting position for the
    /// opponents pieces.
    pub fn to_their_backrank(&self) -> u8 {
        // if self == Color::White { 7 } else { 0 }
        ((1 - self.to_index()) * 7) as u8
    }

    /// Convert a `Color` to my second rank, which represents the starting position for my pawns.
    pub fn to_second_rank(&self) -> u8 {
        (self.to_index() * 5 + 1) as u8
    }
}

impl Not for Color {
    type Output = Color;

    /// Get the other color.
    fn not(self) -> Color {
        if self == Color::White {
            Color::Black
        } else {
            Color::White
        }
    }
}
