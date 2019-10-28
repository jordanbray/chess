use crate::board::Board;
use crate::error::Error;
use crate::file::File;
use crate::movegen::MoveGen;
use crate::piece::Piece;
use crate::rank::Rank;
use crate::square::Square;

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represent a ChessMove in memory
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Default, Debug, Hash)]
pub struct ChessMove {
    source: Square,
    dest: Square,
    promotion: Option<Piece>,
}

impl ChessMove {
    /// Create a new chess move, given a source `Square`, a destination `Square`, and an optional
    /// promotion `Piece`
    #[inline]
    pub fn new(source: Square, dest: Square, promotion: Option<Piece>) -> ChessMove {
        ChessMove {
            source: source,
            dest: dest,
            promotion: promotion,
        }
    }

    /// Get the source square (square the piece is currently on).
    #[inline]
    pub fn get_source(&self) -> Square {
        self.source
    }

    /// Get the destination square (square the piece is going to).
    #[inline]
    pub fn get_dest(&self) -> Square {
        self.dest
    }

    /// Get the promotion piece (maybe).
    #[inline]
    pub fn get_promotion(&self) -> Option<Piece> {
        self.promotion
    }

    pub fn from_san(board: &Board, move_text: &str) -> Result<ChessMove, Error> {
        // Castles first...
        if move_text == "O-O" || move_text == "O-O-O" {
            let rank = board.side_to_move().to_my_backrank();
            let source_file = File::E;
            let dest_file = if move_text == "O-O" { File::G } else { File::C };

            let m = ChessMove::new(
                Square::make_square(rank, source_file),
                Square::make_square(rank, dest_file),
                None,
            );
            if MoveGen::new_legal(&board).any(|l| l == m) {
                return Ok(m);
            } else {
                return Err(Error::InvalidSanMove);
            }
        }

        // forms of SAN moves
        // a4 (Pawn moves to a4)
        // exd4 (Pawn on e file takes on d4)
        // xd4 (Illegal, source file must be specified)
        // 1xd4 (Illegal, source file (not rank) must be specified)
        // Nc3 (Knight (or any piece) on *some square* to c3
        // Nb1c3 (Knight (or any piece) on b1 to c3
        // Nbc3 (Knight on b file to c3)
        // N1c3 (Knight on first rank to c3)
        // Nb1xc3 (Knight on b1 takes on c3)
        // Nbxc3 (Knight on b file takes on c3)
        // N1xc3 (Knight on first rank takes on c3)
        // Nc3+ (Knight moves to c3 with check)
        // Nc3# (Knight moves to c3 with checkmate)

        // Because I'm dumb, I'm wondering if a hash table of all possible moves would be stupid.
        // There are only 186624 possible moves in SAN notation.
        //
        // Would this even be faster?  Somehow I doubt it because caching, but maybe, I dunno...
        // This could take the form of a:
        // struct CheckOrCheckmate {
        //      Neither,
        //      Check,
        //      CheckMate,
        // }
        // struct FromSan {
        //      piece: Piece,
        //      source: Vec<Square>, // possible source squares
        //      // OR
        //      source_rank: Option<Rank>,
        //      source_file: Option<File>,
        //      dest: Square,
        //      takes: bool,
        //      check: CheckOrCheckmate
        // }
        //
        // This could be kept internally as well, and never tell the user about such an abomination
        //
        // I estimate this table would take around 2 MiB, but I had to approximate some things.  It
        // may be less

        // This can be described with the following format
        // [Optional Piece Specifier] ("" | "N" | "B" | "R" | "Q" | "K")
        // [Optional Source Specifier] ( "" | "a-h" | "1-8" | ("a-h" + "1-8"))
        // [Optional Takes Specifier] ("" | "x")
        // [Full Destination Square] ("a-h" + "0-8")
        // [Optional Promotion Specifier] ("" | "N" | "B" | "R" | "Q")
        // [Optional Check(mate) Specifier] ("" | "+" | "#")
        // [Optional En Passant Specifier] ("" | " e.p.")

        let error = Error::InvalidSanMove;
        let mut cur_index: usize = 0;
        let moving_piece = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "N" => {
                cur_index += 1;
                Piece::Knight
            }
            "B" => {
                cur_index += 1;
                Piece::Bishop
            }
            "Q" => {
                cur_index += 1;
                Piece::Queen
            }
            "R" => {
                cur_index += 1;
                Piece::Rook
            }
            "K" => {
                cur_index += 1;
                Piece::King
            }
            _ => Piece::Pawn,
        };

        println!("Piece: {}", moving_piece);

        let mut source_file = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "a" => {
                cur_index += 1;
                Some(File::A)
            }
            "b" => {
                cur_index += 1;
                Some(File::B)
            }
            "c" => {
                cur_index += 1;
                Some(File::C)
            }
            "d" => {
                cur_index += 1;
                Some(File::D)
            }
            "e" => {
                cur_index += 1;
                Some(File::E)
            }
            "f" => {
                cur_index += 1;
                Some(File::F)
            }
            "g" => {
                cur_index += 1;
                Some(File::G)
            }
            "h" => {
                cur_index += 1;
                Some(File::H)
            }
            _ => None,
        };

        let mut source_rank = match move_text
            .get(cur_index..(cur_index + 1))
            .ok_or(error.clone())?
        {
            "1" => {
                cur_index += 1;
                Some(Rank::First)
            }
            "2" => {
                cur_index += 1;
                Some(Rank::Second)
            }
            "3" => {
                cur_index += 1;
                Some(Rank::Third)
            }
            "4" => {
                cur_index += 1;
                Some(Rank::Fourth)
            }
            "5" => {
                cur_index += 1;
                Some(Rank::Fifth)
            }
            "6" => {
                cur_index += 1;
                Some(Rank::Sixth)
            }
            "7" => {
                cur_index += 1;
                Some(Rank::Seventh)
            }
            "8" => {
                cur_index += 1;
                Some(Rank::Eighth)
            }
            _ => None,
        };

        let takes = if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
            match s {
                "x" => {
                    cur_index += 1;
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        let dest = if let Some(s) = move_text.get(cur_index..(cur_index + 2)) {
            if let Ok(q) = Square::from_str(s) {
                cur_index += 2;
                q
            } else {
                let sq = Square::make_square(
                    source_rank.ok_or(error.clone())?,
                    source_file.ok_or(error.clone())?,
                );
                source_rank = None;
                source_file = None;
                sq
            }
        } else {
            let sq = Square::make_square(
                source_rank.ok_or(error.clone())?,
                source_file.ok_or(error.clone())?,
            );
            source_rank = None;
            source_file = None;
            sq
        };

        println!("Destination: {}", dest);

        let promotion = if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
            match s {
                "N" => {
                    cur_index += 1;
                    Some(Piece::Knight)
                }
                "B" => {
                    cur_index += 1;
                    Some(Piece::Bishop)
                }
                "R" => {
                    cur_index += 1;
                    Some(Piece::Rook)
                }
                "Q" => {
                    cur_index += 1;
                    Some(Piece::Queen)
                }
                _ => None,
            }
        } else {
            None
        };

        println!("Promotion: {:?}", promotion);

        if let Some(s) = move_text.get(cur_index..(cur_index + 1)) {
            let _maybe_check_or_mate = match s {
                "+" => {
                    cur_index += 1;
                    Some(false)
                }
                "#" => {
                    cur_index += 1;
                    Some(true)
                }
                _ => None,
            };
        }

        let ep = if let Some(s) = move_text.get(cur_index..) {
            s == " e.p."
        } else {
            false
        };

        println!("EP: {}", ep);

        if ep {
            cur_index += 5;
        }

        // Ok, now we have all the data from the SAN move, in the following structures
        // moveing_piece, source_rank, source_file, taks, dest, promotion, maybe_check_or_mate, and
        // ep

        let mut found_move: Option<ChessMove> = None;
        for m in &mut MoveGen::new_legal(board) {
            // check that the move has the properties specified
            if board.piece_on(m.get_source()) != Some(moving_piece) {
                continue;
            }

            if let Some(rank) = source_rank {
                if m.get_source().get_rank() != rank {
                    continue;
                }
            }

            if let Some(file) = source_file {
                if m.get_source().get_file() != file {
                    continue;
                }
            }

            if m.get_dest() != dest {
                continue;
            }

            if m.get_promotion() != promotion {
                continue;
            }

            if found_move.is_some() {
                return Err(error);
            }

            // takes is complicated, because of e.p.
            if !takes {
                if board.piece_on(m.get_dest()).is_some() {
                    continue;
                }
            }

            if !ep && takes {
                if board.piece_on(m.get_dest()).is_none() {
                    continue;
                }
            }

            found_move = Some(m);
        }

        println!("Must have not found the move...");
        found_move.ok_or(error.clone())
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            None => write!(f, "{}{}", self.source, self.dest),
            Some(x) => write!(f, "{}{}{}", self.source, self.dest, x),
        }
    }
}

impl Ord for ChessMove {
    fn cmp(&self, other: &ChessMove) -> Ordering {
        if self.source != other.source {
            self.source.cmp(&other.source)
        } else if self.dest != other.dest {
            self.dest.cmp(&other.dest)
        } else if self.promotion != other.promotion {
            match self.promotion {
                None => Ordering::Less,
                Some(x) => match other.promotion {
                    None => Ordering::Greater,
                    Some(y) => x.cmp(&y),
                },
            }
        } else {
            Ordering::Equal
        }
    }
}

/// Convert a UCI `String` to a move. If invalid, return `None`
/// ```
/// use chess::{ChessMove, Square, Piece};
/// use std::str::FromStr;
///
/// let mv = ChessMove::new(Square::E7, Square::E8, Some(Piece::Queen));
///
/// assert_eq!(ChessMove::from_str("e7e8q").expect("Valid Move"), mv);
/// ```
impl FromStr for ChessMove {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = Square::from_str(s.get(0..2).ok_or(Error::InvalidUciMove)?)?;
        let dest = Square::from_str(s.get(2..4).ok_or(Error::InvalidUciMove)?)?;

        let mut promo = None;
        if s.len() == 5 {
            promo = Some(match s.chars().last().ok_or(Error::InvalidUciMove)? {
                'q' => Piece::Queen,
                'r' => Piece::Rook,
                'n' => Piece::Knight,
                'b' => Piece::Bishop,
                _ => return Err(Error::InvalidUciMove),
            });
        }

        Ok(ChessMove::new(source, dest, promo))
    }
}

#[test]
fn test_basic_moves() {
    let board = Board::default();
    assert_eq!(
        ChessMove::from_san(&board, "e4").expect("e4 is valid in the initial position"),
        ChessMove::new(Square::E2, Square::E4, None)
    );
}
