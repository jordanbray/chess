/// Describe a rank (row) on a chess board
#[derive(Copy, Clone, PartialEq, PartialOrd)]
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

pub const NUM_RANKS: usize = 8;
pub const ALL_RANKS: [Rank; NUM_RANKS] = [Rank::First, Rank::Second, Rank::Third, Rank::Fourth, Rank::Fifth, Rank::Sixth, Rank::Seventh, Rank::Eighth];

impl Rank {
    pub fn from_index(i: usize) -> Rank {
        match i & 7 {
            0 => Rank::First,
            1 => Rank::Second,
            2 => Rank::Third,
            3 => Rank::Fourth,
            4 => Rank::Fifth,
            5 => Rank::Sixth,
            6 => Rank::Seventh,
            7 => Rank::Eighth,
            _ => unreachable!()
        }
    }

    pub fn down(&self) -> Rank {
        match *self {
            Rank::First => Rank::Eighth,
            Rank::Second => Rank::First,
            Rank::Third => Rank::Second,
            Rank::Fourth => Rank::Third,
            Rank::Fifth => Rank::Fourth,
            Rank::Sixth => Rank::Fifth,
            Rank::Seventh => Rank::Sixth,
            Rank::Eighth => Rank::Seventh
        }
    }

    pub fn up(&self) -> Rank {
        match *self {
            Rank::First => Rank::Second,
            Rank::Second => Rank::Third,
            Rank::Third => Rank::Fourth,
            Rank::Fourth => Rank::Fifth,
            Rank::Fifth => Rank::Sixth,
            Rank::Sixth => Rank::Seventh,
            Rank::Seventh => Rank::Eighth,
            Rank::Eighth => Rank::First
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }
}
