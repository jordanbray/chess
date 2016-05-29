/// Describe a file (column) on a chess board
#[derive(Copy, Clone, PartialEq, PartialOrd)]
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

pub const NUM_FILES: usize = 8;
pub const ALL_FILES: [File; NUM_FILES] = [File::A, File::B, File::C, File::D, File::E, File::F, File::G, File::H];

impl File {
    pub fn from_index(i: usize) -> File {
        match i & 7 {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => unreachable!()
        }
    }

    pub fn left(&self) -> File {
        match *self {
            File::A => File::H,
            File::B => File::A,
            File::C => File::B,
            File::D => File::C,
            File::E => File::D,
            File::F => File::E,
            File::G => File::F,
            File::H => File::G
        }
    }

    pub fn right(&self) -> File {
        match *self {
            File::A => File::B,
            File::B => File::C,
            File::C => File::D,
            File::D => File::E,
            File::E => File::F,
            File::F => File::G,
            File::G => File::H,
            File::H => File::A
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }
}
