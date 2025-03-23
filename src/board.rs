use crate::bitboard::{BitBoard, EMPTY};
use crate::board_builder::BoardBuilder;
use crate::castle_rights::CastleRights;
use crate::chess_move::ChessMove;
use crate::color::{Color, ALL_COLORS, NUM_COLORS};
use crate::error::InvalidError;
use crate::file::File;
use crate::magic::{
    between, get_adjacent_files, get_bishop_rays, get_castle_moves, get_file, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_dest_double_moves, get_pawn_source_double_moves,
    get_rank, get_rook_rays,
};
use crate::movegen::*;
use crate::piece::{Piece, ALL_PIECES, NUM_PIECES};
use crate::square::{Square, ALL_SQUARES};
use crate::zobrist::Zobrist;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
#[cfg(feature = "std")]
use std::sync::LazyLock;

/// A representation of a chess board.  That's why you're here, right?
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    color_combined: [BitBoard; NUM_COLORS],
    combined: BitBoard,
    side_to_move: Color,
    castle_rights: [CastleRights; NUM_COLORS],
    pinned: BitBoard,
    checkers: BitBoard,
    hash: u64,
    en_passant: Option<Square>,
}

/// What is the status of this game?
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

/// Construct the initial position.
impl Default for Board {
    /// A board set up with the initial position of all chess games.
    ///
    /// ```
    /// use chess::Board;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Board::default(), Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap());
    /// ```
    #[inline(always)]
    #[cfg(feature = "std")]
    fn default() -> Board {
        *STARTPOS
    }

    /// A board set up with the initial position of all chess games.
    /// ```
    /// use chess::Board;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Board::default(), Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap());
    /// ```
    #[inline(always)]
    #[cfg(not(feature = "std"))]
    fn default() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Startpos FEN is valid FEN")
    }
}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

/// The starting position of a chess board.
/// This `static` is of type `LazyLock<Board>` so that it only has to be computed once.
///
/// ```
/// use chess::{Board, STARTPOS};
/// use std::str::FromStr;
///
/// assert_eq!(*STARTPOS, Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap());
/// ```
#[cfg(feature = "std")]
pub static STARTPOS: LazyLock<Board> = LazyLock::new(|| {
    Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("Startpos FEN is valid FEN")
});

impl Board {
    /// Construct a new `Board` that is completely empty.
    ///
    /// Note: This does **NOT** give you the initial position. Just a blank slate.
    /// To obtain the initial position used in normal chess games, call `Board::default()`.
    ///
    /// `Board::new()` is cheaper than the first call of `Board::default()` or first dereference of `STARTPOS` but is otherwise exactly as expensive,
    /// as it is a simple `Copy` of a `Board`.
    #[inline(always)]
    pub const fn new() -> Board {
        Board {
            pieces: [EMPTY; NUM_PIECES],
            color_combined: [EMPTY; NUM_COLORS],
            combined: EMPTY,
            side_to_move: Color::White,
            castle_rights: [CastleRights::NoRights; NUM_COLORS],
            pinned: EMPTY,
            checkers: EMPTY,
            hash: 0,
            en_passant: None,
        }
    }

    /// Construct a board from a FEN string.
    ///
    /// ```
    /// use chess::Board;
    /// use std::str::FromStr;
    /// # use chess::InvalidError;
    ///
    /// # fn main() -> Result<(), InvalidError> {
    ///
    /// // This is no longer supported
    /// let init_position = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()).expect("Valid FEN");
    /// assert_eq!(init_position, Board::default());
    ///
    /// // This is the new way
    /// let init_position_2 = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?;
    /// assert_eq!(init_position_2, Board::default());
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "Internally this is a wrapper for `Board::from_str`, please use this function directly instead"
    )]
    #[inline(always)]
    #[cfg(feature = "std")]
    pub fn from_fen(fen: String) -> Option<Board> {
        Board::from_str(&fen).ok()
    }

    #[deprecated(
        since = "3.0.0",
        note = "Internally this wraps `MoveGen::new_legal`, please use this structure instead"
    )]
    #[inline(always)]
    pub fn enumerate_moves(&self, moves: &mut [ChessMove; 256]) -> usize {
        let movegen = MoveGen::new_legal(self);
        let mut size = 0;
        for m in movegen {
            moves[size] = m;
            size += 1;
        }
        size
    }

    /// Is this game Ongoing, is it Stalemate, or is it Checkmate?
    ///
    /// Note: This function is optimized to only find the first possible move to minimize time cost.
    ///
    /// ```
    /// use chess::{Board, BoardStatus, Square, ChessMove};
    ///
    /// let mut board = Board::default();
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::E2, Square::E4, None));
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::F7, Square::F6, None));
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::D2, Square::D4, None));
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::G7, Square::G5, None));
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::D1, Square::H5, None));
    /// assert_eq!(board.status(), BoardStatus::Checkmate);
    /// ```
    #[inline(always)]
    pub fn status(&self) -> BoardStatus {
        if !MoveGen::has_legals(self) {
            if self.checkers == EMPTY {
                BoardStatus::Stalemate
            } else {
                BoardStatus::Checkmate
            }
        } else {
            BoardStatus::Ongoing
        }
    }

    /// Grab the "combined" `BitBoard`.  This is a `BitBoard` with every piece.
    ///
    /// ```
    /// use chess::{Board, BitBoard, Rank, get_rank};
    ///
    /// let board = Board::default();
    ///
    /// let combined_should_be = get_rank(Rank::First) |
    ///                          get_rank(Rank::Second) |
    ///                          get_rank(Rank::Seventh) |
    ///                          get_rank(Rank::Eighth);
    ///
    /// assert_eq!(*board.combined(), combined_should_be);
    /// ```
    #[inline(always)]
    pub const fn combined(&self) -> &BitBoard {
        &self.combined
    }

    /// Grab the "color combined" `BitBoard`.  This is a `BitBoard` with every piece of a particular
    /// color.
    ///
    /// ```
    /// use chess::{Board, BitBoard, Rank, get_rank, Color};
    ///
    /// let board = Board::default();
    ///
    /// let white_pieces = get_rank(Rank::First) |
    ///                    get_rank(Rank::Second);
    ///
    /// let black_pieces = get_rank(Rank::Seventh) |
    ///                    get_rank(Rank::Eighth);
    ///
    /// assert_eq!(*board.color_combined(Color::White), white_pieces);
    /// assert_eq!(*board.color_combined(Color::Black), black_pieces);
    /// ```
    #[inline(always)]
    pub fn color_combined(&self, color: Color) -> &BitBoard {
        unsafe { self.color_combined.get_unchecked(color.into_index()) }
    }

    /// Get the set of pieces of a particular color.
    ///
    /// ```
    /// use chess::{Board, BitBoard, Piece, Color, Square};
    ///
    /// let white_rooks = BitBoard::from_square(Square::A1) |
    ///                   BitBoard::from_square(Square::H1);
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.pieces_with_color(Piece::Rook, Color::White), white_rooks);
    /// ```
    #[inline(always)]
    pub fn pieces_with_color(&self, piece: Piece, color: Color) -> BitBoard {
        self.pieces(piece) & self.color_combined(color)
    }

    /// Give me the `Square` the `color` king is on.
    ///
    /// ```
    /// use chess::{Board, Square, Color};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.king_square(Color::White), Square::E1);
    /// assert_eq!(board.king_square(Color::Black), Square::E8);
    /// ```
    #[inline(always)]
    pub fn king_square(&self, color: Color) -> Square {
        self.pieces_with_color(Piece::King, color).to_square()
    }

    /// Grab the "pieces" `BitBoard`.  This is a `BitBoard` with every piece of a particular type.
    ///
    /// ```
    /// use chess::{Board, BitBoard, Piece, Square};
    ///
    /// // The rooks should be in each corner of the board
    /// let rooks = BitBoard::from_square(Square::A1) |
    ///             BitBoard::from_square(Square::H1) |
    ///             BitBoard::from_square(Square::A8) |
    ///             BitBoard::from_square(Square::H8);
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(*board.pieces(Piece::Rook), rooks);
    /// ```
    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> &BitBoard {
        unsafe { self.pieces.get_unchecked(piece.into_index()) }
    }

    /// Grab the `CastleRights` for a particular side.
    ///
    /// ```
    /// use chess::{Board, Square, CastleRights, Color, ChessMove};
    ///
    /// let moves = [ChessMove::new(Square::A2, Square::A4, None),
    ///              ChessMove::new(Square::E7, Square::E5, None),
    ///              ChessMove::new(Square::A1, Square::A2, None),
    ///              ChessMove::new(Square::E8, Square::E7, None)];
    ///
    /// let mut board = Board::default();
    ///
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::Both);
    /// assert_eq!(board.castle_rights(Color::Black), CastleRights::Both);
    ///
    /// board = board.make_moves_new(moves);
    ///
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::KingSide);
    /// assert_eq!(board.castle_rights(Color::Black), CastleRights::NoRights);
    /// ```
    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        unsafe { *self.castle_rights.get_unchecked(color.into_index()) }
    }

    /// Add castle rights for a particular side.  Note: this can create an invalid position.
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn add_castle_rights(&mut self, color: Color, add: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut(color.into_index()) =
                self.castle_rights(color).add(add);
        }
    }

    /// Remove castle rights for a particular side.
    ///
    /// ```
    /// use chess::{Board, CastleRights, Color};
    ///
    /// let mut board = Board::default();
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::Both);
    ///
    /// #[allow(deprecated)]
    /// board.remove_castle_rights(Color::White, CastleRights::KingSide);
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::QueenSide);
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn remove_castle_rights(&mut self, color: Color, remove: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut(color.into_index()) =
                self.castle_rights(color).remove(remove);
        }
    }

    /// Who's turn is it?
    ///
    /// ```
    /// use chess::{Board, Color};
    ///
    /// let mut board = Board::default();
    /// assert_eq!(board.side_to_move(), Color::White);
    /// ```
    #[inline(always)]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// My `CastleRights`
    ///
    /// ```
    /// use chess::{Board, CastleRights};
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1").unwrap();
    ///
    /// assert_eq!(board.my_castle_rights(), CastleRights::KingSide);
    /// ```
    #[inline(always)]
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.side_to_move())
    }

    /// Add to my `CastleRights`.  Note: This can make the position invalid.
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn add_my_castle_rights(&mut self, add: CastleRights) {
        let color = self.side_to_move();
        #[allow(deprecated)]
        self.add_castle_rights(color, add);
    }

    /// Remove some of my `CastleRights`.
    ///
    /// ```
    /// use chess::{Board, CastleRights};
    ///
    /// let mut board = Board::default();
    /// assert_eq!(board.my_castle_rights(), CastleRights::Both);
    ///
    /// #[allow(deprecated)]
    /// board.remove_my_castle_rights(CastleRights::KingSide);
    /// assert_eq!(board.my_castle_rights(), CastleRights::QueenSide);
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline(always)]
    pub fn remove_my_castle_rights(&mut self, remove: CastleRights) {
        let color = self.side_to_move();
        #[allow(deprecated)]
        self.remove_castle_rights(color, remove);
    }

    /// My opponents `CastleRights`.
    ///
    /// ```
    /// use chess::{Board, CastleRights};
    /// use std::str::FromStr;
    ///
    /// let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1").unwrap();
    ///
    /// assert_eq!(board.their_castle_rights(), CastleRights::QueenSide);
    /// ```
    #[inline(always)]
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.side_to_move())
    }

    /// Add to my opponents `CastleRights`. Note: This can make the position invalid.
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn add_their_castle_rights(&mut self, add: CastleRights) {
        let color = !self.side_to_move();
        #[allow(deprecated)]
        self.add_castle_rights(color, add)
    }

    /// Remove some of my opponents `CastleRights`.
    ///
    /// ```
    /// use chess::{Board, CastleRights};
    ///
    /// let mut board = Board::default();
    /// assert_eq!(board.their_castle_rights(), CastleRights::Both);
    ///
    /// #[allow(deprecated)]
    /// board.remove_their_castle_rights(CastleRights::KingSide);
    /// assert_eq!(board.their_castle_rights(), CastleRights::QueenSide);
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline(always)]
    pub fn remove_their_castle_rights(&mut self, remove: CastleRights) {
        let color = !self.side_to_move();
        #[allow(deprecated)]
        self.remove_castle_rights(color, remove);
    }

    /// Add or remove a piece from the bitboards in this struct.
    #[inline(always)]
    fn xor(&mut self, piece: Piece, bb: BitBoard, color: Color) {
        unsafe {
            *self.pieces.get_unchecked_mut(piece.into_index()) ^= bb;
            *self.color_combined.get_unchecked_mut(color.into_index()) ^= bb;
            self.combined ^= bb;
            self.hash ^= Zobrist::piece(piece, bb.to_square(), color);
        }
    }

    /// For a chess UI: set a piece on a particular square.
    ///
    /// ```
    /// use chess::{Board, Piece, Color, Square};
    ///
    /// let board = Board::default();
    ///
    /// #[allow(deprecated)]
    /// let new_board = board.set_piece(Piece::Queen, Color::White, Square::E4)
    ///                      .expect("Valid Position");
    ///
    /// assert_eq!(new_board.pieces(Piece::Queen).count(), 3);
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn set_piece(&self, piece: Piece, color: Color, square: Square) -> Option<Board> {
        let mut result = *self;
        let square_bb = BitBoard::from_square(square);
        match self.piece_on(square) {
            None => result.xor(piece, square_bb, color),
            Some(x) => {
                // remove x from the bitboard
                if self.color_combined(Color::White) & square_bb == square_bb {
                    result.xor(x, square_bb, Color::White);
                } else {
                    result.xor(x, square_bb, Color::Black);
                }
                // add piece to the bitboard
                result.xor(piece, square_bb, color);
            }
        }

        // If setting this piece down leaves my opponent in check, and it's my move, then the
        // position is not a valid chess board
        result.side_to_move = !result.side_to_move;
        result.update_pin_info();
        if result.checkers != EMPTY {
            return None;
        }

        // undo our damage
        result.side_to_move = !result.side_to_move;
        result.update_pin_info();

        Some(result)
    }

    /// For a chess UI: clear a particular square.
    ///
    /// ```
    /// use chess::{Board, Square, Piece};
    ///
    /// let board = Board::default();
    ///
    /// #[allow(deprecated)]
    /// let new_board = board.clear_square(Square::A1)
    ///                      .expect("Valid Position");
    ///
    /// assert_eq!(new_board.pieces(Piece::Rook).count(), 3);
    /// ```
    #[deprecated(
        since = "3.1.0",
        note = "When doing board setup, use the BoardBuilder structure.  It ensures you don't end up with an invalid position."
    )]
    #[inline]
    pub fn clear_square(&self, square: Square) -> Option<Board> {
        let mut result = *self;
        let square_bb = BitBoard::from_square(square);
        match self.piece_on(square) {
            None => {}
            Some(x) => {
                // remove x from the bitboard
                if self.color_combined(Color::White) & square_bb == square_bb {
                    result.xor(x, square_bb, Color::White);
                } else {
                    result.xor(x, square_bb, Color::Black);
                }
            }
        }

        // If setting this piece down leaves my opponent in check, and it's my move, then the
        // position is not a valid chess board
        result.side_to_move = !result.side_to_move;
        result.update_pin_info();
        if result.checkers != EMPTY {
            return None;
        }

        // undo our damage
        result.side_to_move = !result.side_to_move;
        result.update_pin_info();

        Some(result)
    }

    /// Switch the color of the player without actually making a move.  Returns None if the current
    /// player is in check.
    ///
    /// Note that this erases the en-passant information, so applying this function twice does not
    /// always give the same result back.
    ///
    /// ```
    /// use chess::{Board, Color};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.side_to_move(), Color::White);
    ///
    /// let new_board = board.null_move().expect("Valid Position");
    ///
    /// assert_eq!(new_board.side_to_move(), Color::Black);
    /// ```
    #[inline(always)]
    pub fn null_move(&self) -> Option<Board> {
        if self.checkers != EMPTY {
            None
        } else {
            let mut result = *self;
            result.side_to_move = !result.side_to_move;
            result.remove_ep();
            result.update_pin_info();
            Some(result)
        }
    }

    /// Does this board "make sense"?
    /// Do all the pieces make sense, do the bitboards combine correctly, etc?
    /// This is for sanity checking.
    ///
    /// ```
    /// use chess::{Board, Color, Piece, Square};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.is_sane(), true);
    ///
    /// // Remove the king
    /// let bad_board = board.clear_square(Square::E1).expect("Valid Position");
    /// assert_eq!(bad_board.is_sane(), false);
    /// ```
    pub fn is_sane(&self) -> bool {
        // make sure there is no square with multiple pieces on it
        for x in ALL_PIECES.iter() {
            for y in ALL_PIECES.iter() {
                if *x != *y && self.pieces(*x) & self.pieces(*y) != EMPTY {
                    return false;
                }
            }
        }

        // make sure the colors don't overlap, either
        if self.color_combined(Color::White) & self.color_combined(Color::Black) != EMPTY {
            return false;
        }

        // grab all the pieces by OR'ing together each piece() BitBoard
        let combined = ALL_PIECES
            .iter()
            .fold(EMPTY, |cur, next| cur | self.pieces(*next));

        // make sure that's equal to the combined bitboard
        if combined != *self.combined() {
            return false;
        }

        // make sure there is exactly one white king
        if self.pieces_with_color(Piece::King, Color::White).popcnt() != 1 {
            return false;
        }

        // make sure there is exactly one black king
        if self.pieces_with_color(Piece::King, Color::Black).popcnt() != 1 {
            return false;
        }

        // make sure the en_passant square has a pawn on it of the right color
        if let Some(x) = self.en_passant {
            if self.pieces(Piece::Pawn)
                & self.color_combined(!self.side_to_move)
                & BitBoard::from_square(x)
                == EMPTY
            {
                return false;
            }
        }

        // make sure my opponent is not currently in check (because that would be illegal)
        let mut board_copy = *self;
        board_copy.side_to_move = !board_copy.side_to_move;
        board_copy.update_pin_info();
        if board_copy.checkers != EMPTY {
            return false;
        }

        // for each color, verify that, if they have castle rights, that they haven't moved their
        // rooks or king
        for color in ALL_COLORS.iter() {
            // get the castle rights
            let castle_rights = self.castle_rights(*color);

            // the castle rights object will tell us which rooks shouldn't have moved yet.
            // verify there are rooks on all those squares
            if castle_rights.unmoved_rooks(*color)
                & self.pieces(Piece::Rook)
                & self.color_combined(*color)
                != castle_rights.unmoved_rooks(*color)
            {
                return false;
            }
            // if we have castle rights, make sure we have a king on the (E, {1,8}) square,
            // depending on the color
            if castle_rights != CastleRights::NoRights
                && self.pieces_with_color(Piece::King, *color)
                    != get_file(File::E) & get_rank(color.to_my_backrank())
            {
                return false;
            }
        }

        // we must make sure the kings aren't touching
        if get_king_moves(self.king_square(Color::White)) & self.pieces(Piece::King) != EMPTY {
            return false;
        }

        // it checks out
        true
    }

    /// Get a hash of the board.
    #[inline]
    pub fn get_hash(&self) -> u64 {
        self.hash
            ^ if let Some(ep) = self.en_passant {
                Zobrist::en_passant(ep.get_file(), !self.side_to_move)
            } else {
                0
            }
            ^ Zobrist::castles(self.my_castle_rights(), self.side_to_move)
            ^ Zobrist::castles(self.their_castle_rights(), !self.side_to_move)
            ^ Zobrist::color(self.side_to_move)
    }

    /// Get a pawn hash of the board (a hash that only changes on color change and pawn moves).
    #[inline]
    pub fn get_pawn_hash(&self) -> u64 {
        let white_pawns = self.pieces_with_color(Piece::Pawn, Color::White);
        let black_pawns = self.pieces_with_color(Piece::Pawn, Color::Black);

        Zobrist::color(self.side_to_move)
            ^ white_pawns.into_iter().fold(0, |acc, square| {
                acc ^ Zobrist::piece(Piece::Pawn, square, Color::White)
            })
            ^ black_pawns.into_iter().fold(0, |acc, square| {
                acc ^ Zobrist::piece(Piece::Pawn, square, Color::White)
            })
    }

    /// Get a hash that depends only on king and pawn placement and color change.
    #[inline(always)]
    pub fn get_pawn_king_hash(&self) -> u64 {
        self.get_pawn_hash()
            ^ Zobrist::piece(Piece::King, self.king_square(Color::White), Color::White)
            ^ Zobrist::piece(Piece::King, self.king_square(Color::Black), Color::Black)
    }

    /// What piece is on a particular `Square`?  Is there even one?
    ///
    /// ```
    /// use chess::{Board, Piece, Square};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.piece_on(Square::A1), Some(Piece::Rook));
    /// assert_eq!(board.piece_on(Square::D4), None);
    /// ```
    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        let opp = BitBoard::from_square(square);
        if self.combined() & opp == EMPTY {
            None
        } else {
            Some(unsafe { self.piece_on_unchecked(square) })
        }
    }

    /// Get the piece on a particular `Square`, it is undefined behaviour to call this function on an empty square.
    ///
    /// ```
    /// use chess::{Board, Piece, Square};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(unsafe { board.piece_on_unchecked(Square::A1) }, Piece::Rook);
    /// // The following is undefined behaviour
    /// unsafe { board.piece_on_unchecked(Square::A4) };
    /// ```
    #[inline]
    pub unsafe fn piece_on_unchecked(&self, square: Square) -> Piece {
        let opp = BitBoard::from_square(square);
        //naiive algorithm
        /*
        for p in ALL_PIECES {
            if self.pieces(*p) & opp {
                return p;
            }
        } */
        if (self.pieces(Piece::Pawn) ^ self.pieces(Piece::Knight) ^ self.pieces(Piece::Bishop))
            & opp
            != EMPTY
        {
            if self.pieces(Piece::Pawn) & opp != EMPTY {
                Piece::Pawn
            } else if self.pieces(Piece::Knight) & opp != EMPTY {
                Piece::Knight
            } else {
                Piece::Bishop
            }
        } else if self.pieces(Piece::Rook) & opp != EMPTY {
            Piece::Rook
        } else if self.pieces(Piece::Queen) & opp != EMPTY {
            Piece::Queen
        } else {
            Piece::King
        }
    }

    /// What color piece is on a particular square?
    ///
    /// ```
    /// use chess::{Board, Square, Color};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.color_on(Square::A1), Some(Color::White));
    /// assert_eq!(board.color_on(Square::A3), None);
    /// ```
    #[inline]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        if (self.color_combined(Color::White) & BitBoard::from_square(square)) != EMPTY {
            Some(Color::White)
        } else if (self.color_combined(Color::Black) & BitBoard::from_square(square)) != EMPTY {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Unset the en_passant square.
    #[inline(always)]
    fn remove_ep(&mut self) {
        self.en_passant = None;
    }

    /// Give me the en_passant square, if it exists.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square};
    ///
    /// let moves = [ChessMove::new(Square::D2, Square::D4, None),
    ///              ChessMove::new(Square::H7, Square::H5, None),
    ///              ChessMove::new(Square::D4, Square::D5, None),
    ///              ChessMove::new(Square::E7, Square::E5, None)];
    ///
    /// let board = Board::default().make_moves_new(moves);
    ///
    /// assert_eq!(board.en_passant(), Some(Square::E5));
    /// ```
    #[inline(always)]
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Give me the en_passant target square, if it exists.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square};
    ///
    /// let moves = [ChessMove::new(Square::D2, Square::D4, None),
    ///              ChessMove::new(Square::H7, Square::H5, None),
    ///              ChessMove::new(Square::D4, Square::D5, None),
    ///              ChessMove::new(Square::E7, Square::E5, None)];
    ///
    /// let board = Board::default().make_moves_new(moves);
    ///
    /// assert_eq!(board.en_passant_target(), Some(Square::E6));
    /// ```
    #[inline(always)]
    pub fn en_passant_target(&self) -> Option<Square> {
        let color = !self.side_to_move();
        self.en_passant().map(|square| square.ubackward(color))
    }

    /// Set the en_passant square.  Note: This must only be called when self.en_passant is already
    /// None.
    #[inline]
    fn set_ep(&mut self, sq: Square) {
        // Only set self.en_passant if the pawn can actually be captured next move.
        if get_adjacent_files(sq.get_file())
            & get_rank(sq.get_rank())
            & self.pieces(Piece::Pawn)
            & self.color_combined(!self.side_to_move)
            != EMPTY
        {
            self.en_passant = Some(sq);
        }
    }

    /// Is a particular move legal?  This function is very slow, but will work on unsanitized
    /// input.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, MoveGen};
    ///
    /// let m1 = ChessMove::new(Square::E2, Square::E4, None);
    /// let m2 = ChessMove::new(Square::E2, Square::E5, None);
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.legal(m1), true);
    /// assert_eq!(board.legal(m2), false);
    /// ```
    #[inline(always)]
    pub fn legal(&self, m: ChessMove) -> bool {
        MoveGen::new_legal(&self).any(|x| x == m)
    }

    /// Make a chess move onto a new board.
    ///
    /// panic!() if king is captured.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, Color};
    ///
    /// let m = ChessMove::new(Square::D2, Square::D4, None);
    ///
    /// let board = Board::default();
    /// assert_eq!(board.make_move_new(m).side_to_move(), Color::Black);
    /// ```
    #[inline(always)]
    pub fn make_move_new(&self, m: ChessMove) -> Board {
        let mut result = Board::new();
        self.make_move(m, &mut result);
        result
    }

    /// This function exists to mimick the functionality of `make_move`, internally it uses `make_moves_new`
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, BoardStatus};
    ///
    /// let moves = [ChessMove::new(Square::E2, Square::E4, None),
    ///              ChessMove::new(Square::F7, Square::F6, None),
    ///              ChessMove::new(Square::D2, Square::D4, None),
    ///              ChessMove::new(Square::G7, Square::G5, None),
    ///              ChessMove::new(Square::D1, Square::H5, None)];
    ///
    /// let board = Board::default();
    /// let mut result = Board::new();
    /// board.make_moves(moves, &mut result);
    /// assert_eq!(result.status(), BoardStatus::Checkmate);
    /// ```
    #[inline(always)]
    pub fn make_moves<T: IntoIterator<Item = ChessMove>>(&self, moves: T, result: &mut Board) {
        *result = self.make_moves_new(moves);
    }

    /// Apply a series of moves to a board.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, BoardStatus};
    ///
    /// let moves = [ChessMove::new(Square::E2, Square::E4, None),
    ///              ChessMove::new(Square::F7, Square::F6, None),
    ///              ChessMove::new(Square::D2, Square::D4, None),
    ///              ChessMove::new(Square::G7, Square::G5, None),
    ///              ChessMove::new(Square::D1, Square::H5, None)];
    ///
    /// let board = Board::default();
    /// let board2 = board.make_moves_new(moves);
    /// assert_eq!(board2.status(), BoardStatus::Checkmate);
    /// ```
    #[inline(always)]
    pub fn make_moves_new<T: IntoIterator<Item = ChessMove>>(&self, moves: T) -> Board {
        moves
            .into_iter()
            .fold(*self, |acc: Board, m| acc.make_move_new(m))
    }

    /// Make a chess move onto an already allocated `Board`.
    ///
    /// panic!() if king is captured.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, Color};
    ///
    /// let m = ChessMove::new(Square::D2,
    ///                        Square::D4,
    ///                        None);
    ///
    /// let board = Board::default();
    /// // It is generally less expensive to call Board::new() than Board::default()
    /// let mut result = Board::new();
    /// board.make_move(m, &mut result);
    /// assert_eq!(result.side_to_move(), Color::Black);
    /// ```
    pub fn make_move(&self, m: ChessMove, result: &mut Board) {
        *result = *self;
        result.remove_ep();
        result.checkers = EMPTY;
        result.pinned = EMPTY;
        let source = m.get_source();
        let dest = m.get_dest();

        let source_bb = BitBoard::from_square(source);
        let dest_bb = BitBoard::from_square(dest);
        let move_bb = source_bb ^ dest_bb;
        let moved = self.piece_on(source).unwrap();

        result.xor(moved, source_bb, self.side_to_move);
        result.xor(moved, dest_bb, self.side_to_move);
        if let Some(captured) = self.piece_on(dest) {
            result.xor(captured, dest_bb, !self.side_to_move);
        }

        #[allow(deprecated)]
        result.remove_their_castle_rights(CastleRights::square_to_castle_rights(
            !self.side_to_move,
            dest,
        ));

        #[allow(deprecated)]
        result.remove_my_castle_rights(CastleRights::square_to_castle_rights(
            self.side_to_move,
            source,
        ));

        let opp_king = result.pieces_with_color(Piece::King, !result.side_to_move);

        let castles = moved == Piece::King && (move_bb & get_castle_moves()) == move_bb;

        let ksq = opp_king.to_square();

        const CASTLE_ROOK_START: [File; 8] = [
            File::A,
            File::A,
            File::A,
            File::A,
            File::H,
            File::H,
            File::H,
            File::H,
        ];
        const CASTLE_ROOK_END: [File; 8] = [
            File::D,
            File::D,
            File::D,
            File::D,
            File::F,
            File::F,
            File::F,
            File::F,
        ];

        if moved == Piece::Knight {
            result.checkers ^= get_knight_moves(ksq) & dest_bb;
        } else if moved == Piece::Pawn {
            if let Some(Piece::Knight) = m.get_promotion() {
                result.xor(Piece::Pawn, dest_bb, self.side_to_move);
                result.xor(Piece::Knight, dest_bb, self.side_to_move);
                result.checkers ^= get_knight_moves(ksq) & dest_bb;
            } else if let Some(promotion) = m.get_promotion() {
                result.xor(Piece::Pawn, dest_bb, self.side_to_move);
                result.xor(promotion, dest_bb, self.side_to_move);
            } else if (source_bb & get_pawn_source_double_moves()) != EMPTY
                && (dest_bb & get_pawn_dest_double_moves()) != EMPTY
            {
                result.set_ep(dest);
                result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
            } else if Some(dest.ubackward(self.side_to_move)) == self.en_passant {
                result.xor(
                    Piece::Pawn,
                    BitBoard::from_square(dest.ubackward(self.side_to_move)),
                    !self.side_to_move,
                );
                result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
            } else {
                result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
            }
        } else if castles {
            let my_backrank = self.side_to_move.to_my_backrank();
            let index = dest.get_file().into_index();
            let start = BitBoard::set(my_backrank, unsafe {
                *CASTLE_ROOK_START.get_unchecked(index)
            });
            let end = BitBoard::set(my_backrank, unsafe {
                *CASTLE_ROOK_END.get_unchecked(index)
            });
            result.xor(Piece::Rook, start, self.side_to_move);
            result.xor(Piece::Rook, end, self.side_to_move);
        }
        // now, lets see if we're in check or pinned
        let attackers = result.color_combined(result.side_to_move)
            & ((get_bishop_rays(ksq)
                & (result.pieces(Piece::Bishop) | result.pieces(Piece::Queen)))
                | (get_rook_rays(ksq)
                    & (result.pieces(Piece::Rook) | result.pieces(Piece::Queen))));

        for sq in attackers {
            let between = between(sq, ksq) & result.combined();
            if between == EMPTY {
                result.checkers ^= BitBoard::from_square(sq);
            } else if between.popcnt() == 1 {
                result.pinned ^= between;
            }
        }

        result.side_to_move = !result.side_to_move;
    }

    /// Update the pin information.
    fn update_pin_info(&mut self) {
        self.pinned = EMPTY;
        self.checkers = EMPTY;

        let ksq = self.king_square(self.side_to_move);

        let pinners = self.color_combined(!self.side_to_move)
            & ((get_bishop_rays(ksq) & (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)))
                | (get_rook_rays(ksq) & (self.pieces(Piece::Rook) | self.pieces(Piece::Queen))));

        for sq in pinners {
            let between = between(sq, ksq) & self.combined();
            if between == EMPTY {
                self.checkers ^= BitBoard::from_square(sq);
            } else if between.popcnt() == 1 {
                self.pinned ^= between;
            }
        }

        self.checkers ^= get_knight_moves(ksq)
            & self.color_combined(!self.side_to_move)
            & self.pieces(Piece::Knight);

        self.checkers ^= get_pawn_attacks(
            ksq,
            self.side_to_move,
            self.color_combined(!self.side_to_move) & self.pieces(Piece::Pawn),
        );
    }

    /// Give me the `BitBoard` of my pinned pieces.
    #[inline(always)]
    pub fn pinned(&self) -> &BitBoard {
        &self.pinned
    }

    /// Give me the `Bitboard` of the pieces putting me in check.
    #[inline(always)]
    pub fn checkers(&self) -> &BitBoard {
        &self.checkers
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fen: BoardBuilder = self.into();
        write!(f, "{}", fen)
    }
}

impl TryFrom<&BoardBuilder> for Board {
    type Error = InvalidError;

    fn try_from(fen: &BoardBuilder) -> Result<Self, Self::Error> {
        let mut board = Board::new();

        for sq in ALL_SQUARES.iter() {
            if let Some((piece, color)) = fen[*sq] {
                board.xor(piece, BitBoard::from_square(*sq), color);
            }
        }

        board.side_to_move = fen.get_side_to_move();

        if let Some(ep) = fen.get_en_passant() {
            board.side_to_move = !board.side_to_move;
            board.set_ep(ep);
            board.side_to_move = !board.side_to_move;
        }

        #[allow(deprecated)]
        board.add_castle_rights(Color::White, fen.get_castle_rights(Color::White));
        #[allow(deprecated)]
        board.add_castle_rights(Color::Black, fen.get_castle_rights(Color::Black));

        board.update_pin_info();

        if board.is_sane() {
            Ok(board)
        } else {
            Err(InvalidError::Board)
        }
    }
}

impl TryFrom<&mut BoardBuilder> for Board {
    type Error = InvalidError;

    fn try_from(fen: &mut BoardBuilder) -> Result<Self, Self::Error> {
        (&*fen).try_into()
    }
}

impl TryFrom<BoardBuilder> for Board {
    type Error = InvalidError;

    fn try_from(fen: BoardBuilder) -> Result<Self, Self::Error> {
        (&fen).try_into()
    }
}

impl FromStr for Board {
    type Err = InvalidError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(BoardBuilder::from_str(value)?.try_into()?)
    }
}

#[test]
fn test_null_move_en_passant() {
    let start =
        Board::from_str("rnbqkbnr/pppp2pp/8/4pP2/8/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 0").unwrap();
    let expected =
        Board::from_str("rnbqkbnr/pppp2pp/8/4pP2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 0").unwrap();
    assert_eq!(start.null_move().unwrap(), expected);
}

#[test]
fn check_startpos_correct() {
    let startpos_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(startpos_fen).unwrap();
    let startpos = *STARTPOS;
    assert_eq!(board, startpos, "Startpos is not correct");
}
