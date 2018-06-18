use color::Color;
use rank::Rank;
use file::File;
use std::fmt;

/// Represent a square on the chess board
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Debug)]
pub struct Square(u8);

/// How many squares are there?
pub const NUM_SQUARES: usize = 64;

impl Default for Square {
    /// Create a square on A1.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let explicit_sq = Square::make_square(Rank::First, File::A);
    /// let implicit_sq = Square::default();
    ///
    /// assert_eq!(explicit_sq, implicit_sq);
    /// ```
    fn default() -> Square {
        unsafe {
            Square::new(0)
        }
    }
}

impl Square {
    /// Create a new square, given an index.
    /// Note: It is invalid, but allowed, to pass in a number >= 64.  Doing so will crash stuff.
    ///
    /// ```
    ///
    /// use chess::{Square, Rank, File, EMPTY};
    ///
    /// assert_eq!(unsafe { Square::new(0) }, Square::default());
    ///
    /// let bad_sq = unsafe { Square::new(64) };
    ///
    /// // Iterate over all possible squares and ensure that *none* of them are equal to `bad_sq`.
    /// for sq in !EMPTY {
    ///     assert_ne!(bad_sq, sq);
    /// }
    /// ```
    pub unsafe fn new(sq: u8) -> Square {
        Square(sq)
    }

    /// Make a square given a rank and a file
    ///
    /// ```
    /// use chess::{Square, Rank, File, BitBoard};
    ///
    /// // Make the A1 square
    /// let sq = Square::make_square(Rank::First, File::A);
    ///
    /// // Convert it to a bitboard
    /// let bb = BitBoard::from_square(sq);
    ///
    /// // loop over all squares in the bitboard (should be only one), and ensure that the square
    /// // is what we created
    /// for x in bb {
    ///     assert_eq!(sq, x);
    /// }
    /// ```
    pub fn make_square(rank: Rank, file: File) -> Square {
        Square((rank.to_index() as u8)<<3 ^ (file.to_index() as u8))
    }

    /// Return the rank given this square.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.get_rank(), Rank::Seventh);
    /// ```
    pub fn get_rank(&self) -> Rank {
        Rank::from_index((self.0 >> 3) as usize)
    }

    /// Return the file given this square.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.get_file(), File::D);
    /// ```
    pub fn get_file(&self) -> File {
        File::from_index((self.0 & 7) as usize)
    }

    /// If there is a square above me, return that.  Otherwise, None.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.up().expect("Valid Square"), Square::make_square(Rank::Eighth, File::D));
    ///
    /// assert_eq!(sq.up().expect("Valid Square").up(), None);
    /// ```
    pub fn up(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth {
            None
        } else {
            Some(Square::make_square(self.get_rank().up(), self.get_file()))
        }
    }

    /// If there is a square below me, return that.  Otherwise, None.
    /// 
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.down().expect("Valid Square"), Square::make_square(Rank::First, File::D));
    ///
    /// assert_eq!(sq.down().expect("Valid Square").down(), None);
    /// ```
    pub fn down(&self) -> Option<Square> {
        if self.get_rank() == Rank::First {
            None
        } else {
            Some(Square::make_square(self.get_rank().down(), self.get_file()))
        }
    }

    /// If there is a square to the left of me, return that.  Otherwise, None.
    /// 
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::B);
    ///
    /// assert_eq!(sq.left().expect("Valid Square"), Square::make_square(Rank::Seventh, File::A));
    ///
    /// assert_eq!(sq.left().expect("Valid Square").left(), None);
    /// ```
    pub fn left(&self) -> Option<Square> {
        if self.get_file() == File::A {
            None
        } else {
            Some(Square::make_square(self.get_rank(), self.get_file().left()))
        }
    }

    /// If there is a square to the right of me, return that.  Otherwise, None.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::G);
    ///
    /// assert_eq!(sq.right().expect("Valid Square"), Square::make_square(Rank::Seventh, File::H));
    ///
    /// assert_eq!(sq.right().expect("Valid Square").right(), None);
    /// ```
     pub fn right(&self) -> Option<Square> {
        if self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(self.get_rank(), self.get_file().right()))
        }
    }

    /// If there is a square "forward", given my `Color`, go in that direction.  Otherwise, None.
    ///
    /// ```
    /// use chess::{Square, Rank, File, Color};
    ///
    /// let mut sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.forward(Color::White).expect("Valid Square"), Square::make_square(Rank::Eighth, File::D));
    /// assert_eq!(sq.forward(Color::White).expect("Valid Square").forward(Color::White), None);
    ///
    /// sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.forward(Color::Black).expect("Valid Square"), Square::make_square(Rank::First, File::D));
    /// assert_eq!(sq.forward(Color::Black).expect("Valid Square").forward(Color::Black), None);
    /// ```
    pub fn forward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    /// If there is a square "backward" given my `Color`, go in that direction.  Otherwise, None.
    ///
    /// ```
    /// use chess::{Square, Rank, File, Color};
    ///
    /// let mut sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.backward(Color::Black).expect("Valid Square"), Square::make_square(Rank::Eighth, File::D));
    /// assert_eq!(sq.backward(Color::Black).expect("Valid Square").backward(Color::Black), None);
    ///
    /// sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.backward(Color::White).expect("Valid Square"), Square::make_square(Rank::First, File::D));
    /// assert_eq!(sq.backward(Color::White).expect("Valid Square").backward(Color::White), None);
    /// ```
    pub fn backward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up()
        }
    }


    /// If there is a square above me, return that.  If not, wrap around to the other side.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.uup(), Square::make_square(Rank::Eighth, File::D));
    ///
    /// assert_eq!(sq.uup().uup(), Square::make_square(Rank::First, File::D));
    /// ```
    pub fn uup(&self) -> Square {
        Square::make_square(self.get_rank().up(), self.get_file())
    }

    /// If there is a square below me, return that.  If not, wrap around to the other side.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.udown(), Square::make_square(Rank::First, File::D));
    ///
    /// assert_eq!(sq.udown().udown(), Square::make_square(Rank::Eighth, File::D));
    /// ```
     pub fn udown(&self) -> Square {
        Square::make_square(self.get_rank().down(), self.get_file())
    }

    /// If there is a square to the left of me, return that. If not, wrap around to the other side.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::B);
    ///
    /// assert_eq!(sq.uleft(), Square::make_square(Rank::Seventh, File::A));
    ///
    /// assert_eq!(sq.uleft().uleft(), Square::make_square(Rank::Seventh, File::H));
    /// ```
     pub fn uleft(&self) -> Square {
        Square::make_square(self.get_rank(), self.get_file().left())
    }

    /// If there is a square to the right of me, return that.  If not, wrap around to the other
    /// side.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// let sq = Square::make_square(Rank::Seventh, File::G);
    ///
    /// assert_eq!(sq.uright(), Square::make_square(Rank::Seventh, File::H));
    ///
    /// assert_eq!(sq.uright().uright(), Square::make_square(Rank::Seventh, File::A));
    /// ```
     pub fn uright(&self) -> Square {
        Square::make_square(self.get_rank(), self.get_file().right())
    }

    /// If there is a square "forward", given my color, return that.  If not, wrap around to the
    /// other side.
    ///
    /// ```
    /// use chess::{Square, Rank, File, Color};
    ///
    /// let mut sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.uforward(Color::White), Square::make_square(Rank::Eighth, File::D));
    /// assert_eq!(sq.uforward(Color::White).uforward(Color::White), Square::make_square(Rank::First, File::D));
    ///
    /// sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.uforward(Color::Black), Square::make_square(Rank::First, File::D));
    /// assert_eq!(sq.uforward(Color::Black).uforward(Color::Black), Square::make_square(Rank::Eighth, File::D));
    /// ```
     pub fn uforward(&self, color: Color) -> Square {
        match color {
            Color::White => self.uup(),
            Color::Black => self.udown()
        }
    }

    /// If there is a square "backward", given my color, return that.  If not, wrap around to the
    /// other side.
    ///
    /// ```
    /// use chess::{Square, Rank, File, Color};
    ///
    /// let mut sq = Square::make_square(Rank::Seventh, File::D);
    ///
    /// assert_eq!(sq.ubackward(Color::Black), Square::make_square(Rank::Eighth, File::D));
    /// assert_eq!(sq.ubackward(Color::Black).ubackward(Color::Black), Square::make_square(Rank::First, File::D));
    ///
    /// sq = Square::make_square(Rank::Second, File::D);
    ///
    /// assert_eq!(sq.ubackward(Color::White), Square::make_square(Rank::First, File::D));
    /// assert_eq!(sq.ubackward(Color::White).ubackward(Color::White), Square::make_square(Rank::Eighth, File::D));
    /// ```
    pub fn ubackward(&self, color: Color) -> Square {
        match color {
            Color::White => self.udown(),
            Color::Black => self.uup()
        }
    }

    /// Convert this square to an integer.
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::make_square(Rank::First, File::A).to_int(), 0);
    /// assert_eq!(Square::make_square(Rank::Second, File::A).to_int(), 8);
    /// assert_eq!(Square::make_square(Rank::First, File::B).to_int(), 1);
    /// assert_eq!(Square::make_square(Rank::Eighth, File::H).to_int(), 63);
    /// ```
    pub fn to_int(&self) -> u8 {
        self.0
    }

    /// Convert this `Square` to a `usize` for table lookup purposes
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::make_square(Rank::First, File::A).to_index(), 0);
    /// assert_eq!(Square::make_square(Rank::Second, File::A).to_index(), 8);
    /// assert_eq!(Square::make_square(Rank::First, File::B).to_index(), 1);
    /// assert_eq!(Square::make_square(Rank::Eighth, File::H).to_index(), 63);
    /// ```
    pub fn to_index(&self) -> usize {
        self.0 as usize
    }

    /// Convert a UCI `String` to a square.  If invalid, return `None`
    ///
    /// ```
    /// use chess::Square;
    ///
    /// let sq = Square::default();
    ///
    /// assert_eq!(Square::from_string("a1".to_owned()).expect("Valid Square"), sq);
    /// ```
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
impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", (('a' as u8) + ((self.0 & 7) as u8)) as char,
                          (('1' as u8) + ((self.0 >> 3) as u8)) as char)
    }
}
 
/// A list of every square on the chessboard.
///
/// ```
/// use chess::{ALL_SQUARES, BitBoard, EMPTY};
///
/// let universe = !EMPTY;
///
/// let mut new_universe = EMPTY;
///
/// for sq in ALL_SQUARES.iter() {
///     new_universe ^= BitBoard::from_square(*sq);
/// }
///
/// assert_eq!(new_universe, universe);
/// ```
pub const ALL_SQUARES: [Square; 64] = [
     Square(0),  Square(1),  Square(2),  Square(3),  Square(4),  Square(5),  Square(6), Square(7),
     Square(8),  Square(9), Square(10), Square(11), Square(12), Square(13), Square(14), Square(15),
    Square(16), Square(17), Square(18), Square(19), Square(20), Square(21), Square(22), Square(23),
    Square(24), Square(25), Square(26), Square(27), Square(28), Square(29), Square(30), Square(31),
    Square(32), Square(33), Square(34), Square(35), Square(36), Square(37), Square(38), Square(39),
    Square(40), Square(41), Square(42), Square(43), Square(44), Square(45), Square(46), Square(47),
    Square(48), Square(49), Square(50), Square(51), Square(52), Square(53), Square(54), Square(55),
    Square(56), Square(57), Square(58), Square(59), Square(60), Square(61), Square(62), Square(63) ];

