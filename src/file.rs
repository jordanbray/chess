use std::mem::transmute;

/// Describe a file (column) on a chess board
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// How many files are there?
pub const NUM_FILES: usize = 8;

/// Enumerate all files
pub const ALL_FILES: [File; NUM_FILES] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

impl File {
    /// Convert a `usize` into a `File` (the inverse of to_index).  If i > 7, wrap around.
    pub fn from_index(i: usize) -> File {
        unsafe { transmute((i as u8) & 7) }
    }

    /// Go one file to the left.  If impossible, wrap around.
    pub fn left(&self) -> File {
        File::from_index(self.to_index().wrapping_sub(1))
    }

    /// Go one file to the right.  If impossible, wrap around.
    pub fn right(&self) -> File {
        File::from_index(self.to_index() + 1)
    }

    /// Convert this `File` into a `usize` from 0 to 7 inclusive.
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}
