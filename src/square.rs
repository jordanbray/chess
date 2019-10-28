use crate::color::Color;
use crate::error::Error;
use crate::file::File;
use crate::rank::Rank;
use std::fmt;
use std::str::FromStr;

/// Represent a square on the chess board
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Debug, Hash)]
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
        unsafe { Square::new(0) }
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
    #[inline]
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
    #[inline]
    pub fn make_square(rank: Rank, file: File) -> Square {
        Square((rank.to_index() as u8) << 3 ^ (file.to_index() as u8))
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn right(&self) -> Option<Square> {
        if self.get_file() == File::H {
            None
        } else {
            Some(Square::make_square(
                self.get_rank(),
                self.get_file().right(),
            ))
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
    #[inline]
    pub fn forward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down(),
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
    #[inline]
    pub fn backward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up(),
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn uforward(&self, color: Color) -> Square {
        match color {
            Color::White => self.uup(),
            Color::Black => self.udown(),
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
    #[inline]
    pub fn ubackward(&self, color: Color) -> Square {
        match color {
            Color::White => self.udown(),
            Color::Black => self.uup(),
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
    #[inline]
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
    #[inline]
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
    #[deprecated(
        since = "3.1.0",
        note = "please use `Square::from_str(square)?` instead"
    )]
    pub fn from_string(s: String) -> Option<Square> {
        Square::from_str(&s).ok()
    }

    /// The A1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A1, Square::make_square(Rank::First, File::A));
    /// ```
    pub const A1: Square = Square(0);

    /// The B1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B1, Square::make_square(Rank::First, File::B));
    /// ```
    pub const B1: Square = Square(1);

    /// The C1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C1, Square::make_square(Rank::First, File::C));
    /// ```
    pub const C1: Square = Square(2);

    /// The D1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D1, Square::make_square(Rank::First, File::D));
    /// ```
    pub const D1: Square = Square(3);

    /// The E1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E1, Square::make_square(Rank::First, File::E));
    /// ```
    pub const E1: Square = Square(4);

    /// The F1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F1, Square::make_square(Rank::First, File::F));
    /// ```
    pub const F1: Square = Square(5);

    /// The G1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G1, Square::make_square(Rank::First, File::G));
    /// ```
    pub const G1: Square = Square(6);

    /// The H1 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H1, Square::make_square(Rank::First, File::H));
    /// ```
    pub const H1: Square = Square(7);

    /// The A2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A2, Square::make_square(Rank::Second, File::A));
    /// ```
    pub const A2: Square = Square(8);

    /// The B2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B2, Square::make_square(Rank::Second, File::B));
    /// ```
    pub const B2: Square = Square(9);

    /// The C2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C2, Square::make_square(Rank::Second, File::C));
    /// ```
    pub const C2: Square = Square(10);

    /// The D2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D2, Square::make_square(Rank::Second, File::D));
    /// ```
    pub const D2: Square = Square(11);

    /// The E2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E2, Square::make_square(Rank::Second, File::E));
    /// ```
    pub const E2: Square = Square(12);

    /// The F2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F2, Square::make_square(Rank::Second, File::F));
    /// ```
    pub const F2: Square = Square(13);

    /// The G2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G2, Square::make_square(Rank::Second, File::G));
    /// ```
    pub const G2: Square = Square(14);

    /// The H2 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H2, Square::make_square(Rank::Second, File::H));
    /// ```
    pub const H2: Square = Square(15);

    /// The A3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A3, Square::make_square(Rank::Third, File::A));
    /// ```
    pub const A3: Square = Square(16);

    /// The B3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B3, Square::make_square(Rank::Third, File::B));
    /// ```
    pub const B3: Square = Square(17);

    /// The C3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C3, Square::make_square(Rank::Third, File::C));
    /// ```
    pub const C3: Square = Square(18);

    /// The D3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D3, Square::make_square(Rank::Third, File::D));
    /// ```
    pub const D3: Square = Square(19);

    /// The E3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E3, Square::make_square(Rank::Third, File::E));
    /// ```
    pub const E3: Square = Square(20);

    /// The F3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F3, Square::make_square(Rank::Third, File::F));
    /// ```
    pub const F3: Square = Square(21);

    /// The G3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G3, Square::make_square(Rank::Third, File::G));
    /// ```
    pub const G3: Square = Square(22);

    /// The H3 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H3, Square::make_square(Rank::Third, File::H));
    /// ```
    pub const H3: Square = Square(23);

    /// The A4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A4, Square::make_square(Rank::Fourth, File::A));
    /// ```
    pub const A4: Square = Square(24);

    /// The B4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B4, Square::make_square(Rank::Fourth, File::B));
    /// ```
    pub const B4: Square = Square(25);

    /// The C4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C4, Square::make_square(Rank::Fourth, File::C));
    /// ```
    pub const C4: Square = Square(26);

    /// The D4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D4, Square::make_square(Rank::Fourth, File::D));
    /// ```
    pub const D4: Square = Square(27);

    /// The E4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E4, Square::make_square(Rank::Fourth, File::E));
    /// ```
    pub const E4: Square = Square(28);

    /// The F4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F4, Square::make_square(Rank::Fourth, File::F));
    /// ```
    pub const F4: Square = Square(29);

    /// The G4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G4, Square::make_square(Rank::Fourth, File::G));
    /// ```
    pub const G4: Square = Square(30);

    /// The H4 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H4, Square::make_square(Rank::Fourth, File::H));
    /// ```
    pub const H4: Square = Square(31);

    /// The A5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A5, Square::make_square(Rank::Fifth, File::A));
    /// ```
    pub const A5: Square = Square(32);

    /// The B5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B5, Square::make_square(Rank::Fifth, File::B));
    /// ```
    pub const B5: Square = Square(33);

    /// The C5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C5, Square::make_square(Rank::Fifth, File::C));
    /// ```
    pub const C5: Square = Square(34);

    /// The D5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D5, Square::make_square(Rank::Fifth, File::D));
    /// ```
    pub const D5: Square = Square(35);

    /// The E5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E5, Square::make_square(Rank::Fifth, File::E));
    /// ```
    pub const E5: Square = Square(36);

    /// The F5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F5, Square::make_square(Rank::Fifth, File::F));
    /// ```
    pub const F5: Square = Square(37);

    /// The G5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G5, Square::make_square(Rank::Fifth, File::G));
    /// ```
    pub const G5: Square = Square(38);

    /// The H5 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H5, Square::make_square(Rank::Fifth, File::H));
    /// ```
    pub const H5: Square = Square(39);

    /// The A6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A6, Square::make_square(Rank::Sixth, File::A));
    /// ```
    pub const A6: Square = Square(40);

    /// The B6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B6, Square::make_square(Rank::Sixth, File::B));
    /// ```
    pub const B6: Square = Square(41);

    /// The C6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C6, Square::make_square(Rank::Sixth, File::C));
    /// ```
    pub const C6: Square = Square(42);

    /// The D6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D6, Square::make_square(Rank::Sixth, File::D));
    /// ```
    pub const D6: Square = Square(43);

    /// The E6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E6, Square::make_square(Rank::Sixth, File::E));
    /// ```
    pub const E6: Square = Square(44);

    /// The F6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F6, Square::make_square(Rank::Sixth, File::F));
    /// ```
    pub const F6: Square = Square(45);

    /// The G6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G6, Square::make_square(Rank::Sixth, File::G));
    /// ```
    pub const G6: Square = Square(46);

    /// The H6 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H6, Square::make_square(Rank::Sixth, File::H));
    /// ```
    pub const H6: Square = Square(47);

    /// The A7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A7, Square::make_square(Rank::Seventh, File::A));
    /// ```
    pub const A7: Square = Square(48);

    /// The B7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B7, Square::make_square(Rank::Seventh, File::B));
    /// ```
    pub const B7: Square = Square(49);

    /// The C7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C7, Square::make_square(Rank::Seventh, File::C));
    /// ```
    pub const C7: Square = Square(50);

    /// The D7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D7, Square::make_square(Rank::Seventh, File::D));
    /// ```
    pub const D7: Square = Square(51);

    /// The E7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E7, Square::make_square(Rank::Seventh, File::E));
    /// ```
    pub const E7: Square = Square(52);

    /// The F7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F7, Square::make_square(Rank::Seventh, File::F));
    /// ```
    pub const F7: Square = Square(53);

    /// The G7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G7, Square::make_square(Rank::Seventh, File::G));
    /// ```
    pub const G7: Square = Square(54);

    /// The H7 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H7, Square::make_square(Rank::Seventh, File::H));
    /// ```
    pub const H7: Square = Square(55);

    /// The A8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::A8, Square::make_square(Rank::Eighth, File::A));
    /// ```
    pub const A8: Square = Square(56);

    /// The B8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::B8, Square::make_square(Rank::Eighth, File::B));
    /// ```
    pub const B8: Square = Square(57);

    /// The C8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::C8, Square::make_square(Rank::Eighth, File::C));
    /// ```
    pub const C8: Square = Square(58);

    /// The D8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::D8, Square::make_square(Rank::Eighth, File::D));
    /// ```
    pub const D8: Square = Square(59);

    /// The E8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::E8, Square::make_square(Rank::Eighth, File::E));
    /// ```
    pub const E8: Square = Square(60);

    /// The F8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::F8, Square::make_square(Rank::Eighth, File::F));
    /// ```
    pub const F8: Square = Square(61);

    /// The G8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::G8, Square::make_square(Rank::Eighth, File::G));
    /// ```
    pub const G8: Square = Square(62);

    /// The H8 square on the chess board
    ///
    /// ```
    /// use chess::{Square, Rank, File};
    ///
    /// assert_eq!(Square::H8, Square::make_square(Rank::Eighth, File::H));
    /// ```
    pub const H8: Square = Square(63);
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (('a' as u8) + ((self.0 & 7) as u8)) as char,
            (('1' as u8) + ((self.0 >> 3) as u8)) as char
        )
    }
}

impl FromStr for Square {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(Error::InvalidSquare);
        }
        let ch: Vec<char> = s.chars().collect();
        match ch[0] {
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {}
            _ => {
                return Err(Error::InvalidSquare);
            }
        }
        match ch[1] {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {}
            _ => {
                return Err(Error::InvalidSquare);
            }
        }
        Ok(Square::make_square(
            Rank::from_index((ch[1] as usize) - ('1' as usize)),
            File::from_index((ch[0] as usize) - ('a' as usize)),
        ))
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
    Square(0),
    Square(1),
    Square(2),
    Square(3),
    Square(4),
    Square(5),
    Square(6),
    Square(7),
    Square(8),
    Square(9),
    Square(10),
    Square(11),
    Square(12),
    Square(13),
    Square(14),
    Square(15),
    Square(16),
    Square(17),
    Square(18),
    Square(19),
    Square(20),
    Square(21),
    Square(22),
    Square(23),
    Square(24),
    Square(25),
    Square(26),
    Square(27),
    Square(28),
    Square(29),
    Square(30),
    Square(31),
    Square(32),
    Square(33),
    Square(34),
    Square(35),
    Square(36),
    Square(37),
    Square(38),
    Square(39),
    Square(40),
    Square(41),
    Square(42),
    Square(43),
    Square(44),
    Square(45),
    Square(46),
    Square(47),
    Square(48),
    Square(49),
    Square(50),
    Square(51),
    Square(52),
    Square(53),
    Square(54),
    Square(55),
    Square(56),
    Square(57),
    Square(58),
    Square(59),
    Square(60),
    Square(61),
    Square(62),
    Square(63),
];
