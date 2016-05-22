use color::Color;

/// Represent a square on the chess board
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct Square(u8);

/// How many squares are there?
pub const NUM_SQUARES: usize = 64;

impl Square {
    /// Create a new square, given an index.
    /// Note: It is invalid, but allowed, to pass in a number >= 64.  Doing so will crash stuff.
    pub fn new(sq: u8) -> Square {
        Square(sq)
    }

    /// Make a square given a rank and a file
    /// Note: It is invalid, but allowed, to pass in a rank or file >= 8.  Doing so will crash
    /// stuff.
    pub fn make_square(rank: u8, file: u8) -> Square {
        Square(rank<<3 | file)
    }

    /// If there is a square above me, return that.  Otherwise, None.
    pub fn up(&self) -> Option<Square> {
        if self.rank() == 7 {
            None
        } else {
            Some(Square::make_square(self.rank() + 1, self.file()))
        }
    }

    /// If there is a square "forward", given my `Color`, go in that direction.  Otherwise, None.
    pub fn forward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    /// If there is a square "backward" given my `Color`, go in that direction.  Otherwise, None.
    pub fn backward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up()
        }
    }

    /// If there is a square below me, return that.  Otherwise, None.
    pub fn down(&self) -> Option<Square> {
        if self.rank() == 0 {
            None
        } else {
            Some(Square::make_square(self.rank() - 1, self.file()))
        }
    }

    /// If there is a square to the left of me, return that.  Otherwise, None.
    pub fn left(&self) -> Option<Square> {
        if self.file() == 0 {
            None
        } else {
            Some(Square::make_square(self.rank(), self.file() - 1))
        }
    }

    /// If there is a square to the right of me, return that.  Otherwise, None.
    pub fn right(&self) -> Option<Square> {
        if self.file() == 7 {
            None
        } else {
            Some(Square::make_square(self.rank(), self.file() + 1))
        }
    }

    /// If there is a square above me, return that.  Otherwise, return invalid data to crash the
    /// program.
    pub fn uup(&self) -> Square {
        Square::make_square(self.rank() + 1, self.file())
    }

    /// If there is a square below me, return that.  Otherwise, return invalid data to crash the
    /// program.
    pub fn udown(&self) -> Square {
        Square::make_square(self.rank() - 1, self.file())
    }

    /// If there is a square to the left of me, return that.  Otherwise, return invalid data to
    /// crash the program.
    pub fn uleft(&self) -> Square {
        Square::make_square(self.rank(), self.file() - 1)
    }

    /// If there is a square to the right of me, return that.  Otherwise, return invalid data to
    /// crash the program.
    pub fn uright(&self) -> Square {
        Square::make_square(self.rank(), self.file() + 1)
    }

    /// If there is a square "forward", given my color, return that.  Otherwise, return invalid
    /// data to crash the program.
    pub fn uforward(&self, color: Color) -> Square {
        match color {
            Color::White => self.uup(),
            Color::Black => self.udown()
        }
    }

    /// If there is a square "backward", given my color, return that.  Otherwise, return invalid
    /// data to crash the program.
    pub fn ubackward(&self, color: Color) -> Square {
        match color {
            Color::White => self.udown(),
            Color::Black => self.uup()
        }
    }

    /// Return the rank given this square.
    pub fn rank(&self) -> u8 {
        self.0 >> 3
    }

    /// Return the file given this square.
    pub fn file(&self) -> u8 {
        self.0 & 7
    }

    /// Convert this square to an integer.
    pub fn to_int(&self) -> u8 {
        self.0
    }

    /// Convert this `Square` to a `usize` for table lookup purposes
    pub fn to_index(&self) -> usize {
        self.0 as usize
    }

    /// Convert a UCI `String` to a square.  If invalid, return `None`
    pub fn from_string(s: String) -> Option<Square> {
        if s.len() != 2 {
            return None;
        }
        let ch: Vec<char> = s.chars().collect();
        match ch[0] {
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {},
            _ => { return None; }
        }
        match ch[1] {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {},
            _ => { return None; }
        }
        Some(Square::make_square((ch[1] as u8) - ('1' as u8), (ch[0] as u8) - ('a' as u8)))
    }
}

lazy_static! {
    /// A list of every square on the chessboard.
    pub static ref ALL_SQUARES: Vec<Square> = (0..NUM_SQUARES).map(|i| Square::new(i as u8)).collect();
}
