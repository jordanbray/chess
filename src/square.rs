use color::Color;
use rank::Rank;
use file::File;

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
    pub fn make_square(rank: Rank, file: File) -> Square {
        Square((rank.to_index() as u8)<<3 | (file.to_index() as u8))
    }


    /// Return the rank given this square.
    pub fn get_rank(&self) -> Rank {
        Rank::from_index((self.0 >> 3) as usize)
    }

    /// Return the file given this square.
    pub fn get_file(&self) -> File {
        File::from_index((self.0 & 7) as usize)
    }

    /// If there is a square above me, return that.  Otherwise, None.
    pub fn up(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth {
            None
        } else {
            Some(Square::make_square(self.get_rank().up(), self.get_file()))
        }
    }

    /// If there is a square below me, return that.  Otherwise, None.
    pub fn down(&self) -> Option<Square> {
        if self.get_rank() == Rank::First {
            None
        } else {
            Some(Square::make_square(self.get_rank().down(), self.get_file()))
        }
    }

    /// If there is a square to the left of me, return that.  Otherwise, None.
    pub fn left(&self) -> Option<Square> {
        if self.get_file() == File::A {
            None
        } else {
            Some(Square::make_square(self.get_rank(), self.get_file().left()))
        }
    }

    /// If there is a square to the right of me, return that.  Otherwise, None.
    pub fn right(&self) -> Option<Square> {
        if self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(self.get_rank(), self.get_file().right()))
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


    /// If there is a square above me, return that.  Otherwise, return invalid data to crash the
    /// program.
    pub fn uup(&self) -> Square {
        Square::make_square(self.get_rank().up(), self.get_file())
    }

    /// If there is a square below me, return that.  Otherwise, return invalid data to crash the
    /// program.
    pub fn udown(&self) -> Square {
        Square::make_square(self.get_rank().down(), self.get_file())
    }

    /// If there is a square to the left of me, return that.  Otherwise, return invalid data to
    /// crash the program.
    pub fn uleft(&self) -> Square {
        Square::make_square(self.get_rank(), self.get_file().left())
    }

    /// If there is a square to the right of me, return that.  Otherwise, return invalid data to
    /// crash the program.
    pub fn uright(&self) -> Square {
        Square::make_square(self.get_rank(), self.get_file().right())
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
        Some(Square::make_square(Rank::from_index((ch[1] as usize) - ('1' as usize)),
                                 File::from_index((ch[0] as usize) - ('a' as usize))))
    }
}

lazy_static! {
    /// A list of every square on the chessboard.
    pub static ref ALL_SQUARES: Vec<Square> = (0..NUM_SQUARES).map(|i| Square::new(i as u8)).collect();
}
