use crate::bitboard::{BitBoard, EMPTY};
use crate::castle_rights::CastleRights;
use crate::chess_move::ChessMove;
use crate::color::{Color, ALL_COLORS, NUM_COLORS};
use crate::file::File;
use crate::magic::{
    between, get_adjacent_files, get_bishop_rays, get_castle_moves, get_file, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_dest_double_moves, get_pawn_source_double_moves,
    get_rank, get_rook_rays,
};
use crate::movegen::*;
use crate::piece::{Piece, ALL_PIECES, NUM_PIECES};
use crate::rank::Rank;
use crate::square::Square;
use crate::zobrist::Zobrist;
use std::mem;

/// A representation of a chess board.  That's why you're here, right?
#[derive(Copy, Clone, PartialEq, Debug)]
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
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

/// Construct the initial position.
impl Default for Board {
    fn default() -> Board {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned())
            .expect("Valid FEN")
    }
}

impl Board {
    /// Construct a new `Board` that is completely empty.
    /// Note: This does NOT give you the initial position.  Just a blank slate.
    fn new() -> Board {
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
    ///
    /// let init_position = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()).expect("Valid FEN");
    /// assert_eq!(init_position, Board::default());
    /// ```
    pub fn from_fen(fen: String) -> Option<Board> {
        let mut cur_rank = Rank::Eighth;
        let mut cur_file = File::A;
        let mut board: Board = Board::new();

        let tokens: Vec<&str> = fen.split(' ').collect();
        if tokens.len() != 6 {
            return None;
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let ep = tokens[3];
        //let irreversable_moves = tokens[4];
        //let total_moves = tokens[5];

        for x in pieces.chars() {
            match x {
                '/' => {
                    cur_rank = cur_rank.down();
                    cur_file = File::A;
                }
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    cur_file =
                        File::from_index(cur_file.to_index() + (x as usize) - ('0' as usize));
                }
                'r' => {
                    board.xor(Piece::Rook, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file = cur_file.right();
                }
                'R' => {
                    board.xor(Piece::Rook, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file = cur_file.right();
                }
                'n' => {
                    board.xor(
                        Piece::Knight,
                        BitBoard::set(cur_rank, cur_file),
                        Color::Black,
                    );
                    cur_file = cur_file.right();
                }
                'N' => {
                    board.xor(
                        Piece::Knight,
                        BitBoard::set(cur_rank, cur_file),
                        Color::White,
                    );
                    cur_file = cur_file.right();
                }
                'b' => {
                    board.xor(
                        Piece::Bishop,
                        BitBoard::set(cur_rank, cur_file),
                        Color::Black,
                    );
                    cur_file = cur_file.right();
                }
                'B' => {
                    board.xor(
                        Piece::Bishop,
                        BitBoard::set(cur_rank, cur_file),
                        Color::White,
                    );
                    cur_file = cur_file.right();
                }
                'p' => {
                    board.xor(Piece::Pawn, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file = cur_file.right();
                }
                'P' => {
                    board.xor(Piece::Pawn, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file = cur_file.right();
                }
                'q' => {
                    board.xor(
                        Piece::Queen,
                        BitBoard::set(cur_rank, cur_file),
                        Color::Black,
                    );
                    cur_file = cur_file.right();
                }
                'Q' => {
                    board.xor(
                        Piece::Queen,
                        BitBoard::set(cur_rank, cur_file),
                        Color::White,
                    );
                    cur_file = cur_file.right();
                }
                'k' => {
                    board.xor(Piece::King, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file = cur_file.right();
                }
                'K' => {
                    board.xor(Piece::King, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file = cur_file.right();
                }
                _ => {
                    panic!();
                }
            }
        }
        match side {
            "w" | "W" => board.side_to_move = Color::White,
            "b" | "B" => {
                board.side_to_move = Color::Black;
                board.hash ^= Zobrist::color();
            }
            _ => panic!(),
        }

        if castles.contains("K") && castles.contains("Q") {
            board.castle_rights[Color::White.to_index()] = CastleRights::Both;
        } else if castles.contains("K") {
            board.castle_rights[Color::White.to_index()] = CastleRights::KingSide;
        } else if castles.contains("Q") {
            board.castle_rights[Color::White.to_index()] = CastleRights::QueenSide;
        } else {
            board.castle_rights[Color::White.to_index()] = CastleRights::NoRights;
        }

        if castles.contains("k") && castles.contains("q") {
            board.castle_rights[Color::Black.to_index()] = CastleRights::Both;
        } else if castles.contains("k") {
            board.castle_rights[Color::Black.to_index()] = CastleRights::KingSide;
        } else if castles.contains("q") {
            board.castle_rights[Color::Black.to_index()] = CastleRights::QueenSide;
        } else {
            board.castle_rights[Color::Black.to_index()] = CastleRights::NoRights;
        }

        let color = board.side_to_move;

        match Square::from_string(ep.to_owned()) {
            None => {}
            Some(sq) => {
                board.side_to_move = !board.side_to_move;
                board.set_ep(sq.ubackward(color));
                board.side_to_move = !board.side_to_move;
            }
        };

        board.update_pin_info();

        if board.is_sane() {
            Some(board)
        } else {
            None
        }
    }

    /// Is this game Ongoing, is it Stalemate, or is it Checkmate?
    ///
    /// ```
    /// use chess::{Board, BoardStatus, Square, Rank, File, ChessMove};
    ///
    /// let mut board = Board::default();
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::make_square(Rank::Second, File::E),
    ///                                            Square::make_square(Rank::Fourth, File::E),
    ///                                            None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::make_square(Rank::Seventh, File::F),
    ///                                            Square::make_square(Rank::Sixth, File::F),
    ///                                            None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::make_square(Rank::Second, File::D),
    ///                                            Square::make_square(Rank::Fourth, File::D),
    ///                                            None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::make_square(Rank::Seventh, File::G),
    ///                                            Square::make_square(Rank::Fifth, File::G),
    ///                                            None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move_new(ChessMove::new(Square::make_square(Rank::First, File::D),
    ///                                            Square::make_square(Rank::Fifth, File::H),
    ///                                            None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Checkmate);
    /// ```
    pub fn status(&self) -> BoardStatus {
        let moves = MoveGen::new_legal(&self).len();
        match moves {
            0 => {
                if self.checkers == EMPTY {
                    BoardStatus::Stalemate
                } else {
                    BoardStatus::Checkmate
                }
            }
            _ => BoardStatus::Ongoing,
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
    /// assert_eq!(board.combined(), combined_should_be);
    /// ```
    pub fn combined(&self) -> &BitBoard {
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
    /// assert_eq!(board.color_combined(Color::White), white_pieces);
    /// assert_eq!(board.color_combined(Color::Black), black_pieces);
    /// ```
    pub fn color_combined(&self, color: Color) -> &BitBoard {
        unsafe { self.color_combined.get_unchecked(color.to_index()) }
    }

    /// Give me the `Square` the `color` king is on.
    ///
    /// ```
    /// use chess::{Board, Square, Color, Rank, File};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.king_square(Color::White), Square::make_square(Rank::First, File::E));
    /// assert_eq!(board.king_square(Color::Black), Square::make_square(Rank::Eighth, File::E));
    /// ```
    pub fn king_square(&self, color: Color) -> Square {
        (self.pieces(Piece::King) & self.color_combined(color)).to_square()
    }

    /// Grab the "pieces" `BitBoard`.  This is a `BitBoard` with every piece of a particular type.
    ///
    /// ```
    /// use chess::{Board, BitBoard, Piece, Square, Rank, File};
    ///
    /// // The rooks should be in each corner of the board
    /// let rooks = BitBoard::from_square(Square::make_square(Rank::First, File::A)) |
    ///             BitBoard::from_square(Square::make_square(Rank::First, File::H)) |
    ///             BitBoard::from_square(Square::make_square(Rank::Eighth, File::A)) |
    ///             BitBoard::from_square(Square::make_square(Rank::Eighth, File::H));
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.pieces(Piece::Rook), rooks);
    /// ```
    pub fn pieces(&self, piece: Piece) -> &BitBoard {
        unsafe { self.pieces.get_unchecked(piece.to_index()) }
    }

    /// Grab the `CastleRights` for a particular side.
    ///
    /// ```
    /// use chess::{Board, Square, Rank, File, CastleRights, Color, ChessMove};
    ///
    /// let move1 = ChessMove::new(Square::make_square(Rank::Second, File::A),
    ///                            Square::make_square(Rank::Fourth, File::A),
    ///                            None);
    ///
    /// let move2 = ChessMove::new(Square::make_square(Rank::Seventh, File::E),
    ///                            Square::make_square(Rank::Fifth, File::E),
    ///                            None);
    ///
    /// let move3 = ChessMove::new(Square::make_square(Rank::First, File::A),
    ///                            Square::make_square(Rank::Second, File::A),
    ///                            None);
    ///
    /// let move4 = ChessMove::new(Square::make_square(Rank::Eighth, File::E),
    ///                            Square::make_square(Rank::Seventh, File::E),
    ///                            None);
    ///
    /// let mut board = Board::default();
    ///
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::Both);
    /// assert_eq!(board.castle_rights(Color::Black), CastleRights::Both);
    ///
    /// board = board.make_move_new(move1)
    ///              .make_move_new(move2)
    ///              .make_move_new(move3)
    ///              .make_move_new(move4);
    ///
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::KingSide);
    /// assert_eq!(board.castle_rights(Color::Black), CastleRights::NoRights);
    /// ```
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        unsafe { *self.castle_rights.get_unchecked(color.to_index()) }
    }

    /// Add castle rights for a particular side.  Note: this can create an invalid position.
    pub fn add_castle_rights(&mut self, color: Color, add: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut(color.to_index()) =
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
    /// board.remove_castle_rights(Color::White, CastleRights::KingSide);
    /// assert_eq!(board.castle_rights(Color::White), CastleRights::QueenSide);
    /// ```
    pub fn remove_castle_rights(&mut self, color: Color, remove: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut(color.to_index()) =
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
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Grab my `CastleRights`.
    ///
    /// ```
    /// use chess::{Board, Color, CastleRights};
    ///
    /// let mut board = Board::default();
    /// board.remove_castle_rights(Color::White, CastleRights::KingSide);
    /// board.remove_castle_rights(Color::Black, CastleRights::QueenSide);
    ///
    /// assert_eq!(board.my_castle_rights(), board.castle_rights(Color::White));
    /// ```
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.side_to_move())
    }

    /// Add to my `CastleRights`.  Note: This can make the position invalid.
    pub fn add_my_castle_rights(&mut self, add: CastleRights) {
        let color = self.side_to_move();
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
    /// board.remove_my_castle_rights(CastleRights::KingSide);
    /// assert_eq!(board.my_castle_rights(), CastleRights::QueenSide);
    /// ```
    pub fn remove_my_castle_rights(&mut self, remove: CastleRights) {
        let color = self.side_to_move();
        self.remove_castle_rights(color, remove);
    }

    /// My opponents `CastleRights`.
    ///
    /// ```
    /// use chess::{Board, Color, CastleRights};
    ///
    /// let mut board = Board::default();
    /// board.remove_castle_rights(Color::White, CastleRights::KingSide);
    /// board.remove_castle_rights(Color::Black, CastleRights::QueenSide);
    ///
    /// assert_eq!(board.their_castle_rights(), board.castle_rights(Color::Black));
    /// ```
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.side_to_move())
    }

    /// Add to my opponents `CastleRights`. Note: This can make the position invalid.
    pub fn add_their_castle_rights(&mut self, add: CastleRights) {
        let color = !self.side_to_move();
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
    /// board.remove_their_castle_rights(CastleRights::KingSide);
    /// assert_eq!(board.their_castle_rights(), CastleRights::QueenSide);
    /// ```
    pub fn remove_their_castle_rights(&mut self, remove: CastleRights) {
        let color = !self.side_to_move();
        self.remove_castle_rights(color, remove);
    }

    /// Add or remove a piece from the bitboards in this struct.
    fn xor(&mut self, piece: Piece, bb: BitBoard, color: Color) {
        unsafe {
            *self.pieces.get_unchecked_mut(piece.to_index()) ^= bb;
            *self.color_combined.get_unchecked_mut(color.to_index()) ^= bb;
            self.combined ^= bb;
            self.hash ^= Zobrist::piece(piece, bb.to_square(), color);
        }
    }

    /// For a chess UI: set a piece on a particular square.
    ///
    /// ```
    /// use chess::{Board, Piece, Color, Square, Rank, File};
    ///
    /// let board = Board::default();
    ///
    /// let new_board = board.set_piece(Piece::Queen,
    ///                                 Color::White,
    ///                                 Square::make_square(Rank::Fourth, File::E))
    ///                      .expect("Valid Position");
    ///
    /// assert_eq!(new_board.pieces(Piece::Queen).count(), 3);
    /// ```
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
    /// use chess::{Board, Square, Rank, File, Piece};
    ///
    /// let board = Board::default();
    ///
    /// let new_board = board.clear_square(Square::make_square(Rank::First, File::A))
    ///                      .expect("Valid Position");
    ///
    /// assert_eq!(new_board.pieces(Piece::Rook).count(), 3);
    /// ```
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
    pub fn null_move(&self) -> Option<Board> {
        if self.checkers != EMPTY {
            None
        } else {
            let mut result = *self;
            result.side_to_move = !result.side_to_move;
            result.hash ^= Zobrist::color();
            result.update_pin_info();
            Some(result)
        }
    }

    /// Does this board "make sense"?
    /// Do all the pieces make sense, do the bitboards combine correctly, etc?
    /// This is for sanity checking.
    ///
    /// ```
    /// use chess::{Board, Color, Piece, Square, Rank, File};
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.is_sane(), true);
    ///
    /// // Remove the king
    /// let bad_board = board.clear_square(Square::make_square(Rank::First, File::E)).expect("Valid Position");
    /// assert_eq!(bad_board.is_sane(), false);
    /// ```
    pub fn is_sane(&self) -> bool {
        // make sure there is no square with multiple pieces on it
        for x in ALL_PIECES.iter() {
            for y in ALL_PIECES.iter() {
                if *x != *y {
                    if self.pieces(*x) & self.pieces(*y) != EMPTY {
                        return false;
                    }
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
        if (self.pieces(Piece::King) & self.color_combined(Color::White)).popcnt() != 1 {
            return false;
        }

        // make sure there is exactly one black king
        if (self.pieces(Piece::King) & self.color_combined(Color::Black)).popcnt() != 1 {
            return false;
        }

        // make sure the en_passant square has a pawn on it of the right color
        match self.en_passant {
            None => {}
            Some(x) => {
                if self.pieces(Piece::Pawn)
                    & self.color_combined(!self.side_to_move)
                    & BitBoard::from_square(x)
                    == EMPTY
                {
                    return false;
                }
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
            if castle_rights != CastleRights::NoRights {
                if self.pieces(Piece::King) & self.color_combined(*color)
                    != get_file(File::E) & get_rank(color.to_my_backrank())
                {
                    return false;
                }
            }
        }

        // we must make sure the kings aren't touching
        if get_king_moves(self.king_square(Color::White)) & self.pieces(Piece::King) != EMPTY {
            return false;
        }

        // it checks out
        return true;
    }

    /// Get a hash of the board.
    pub fn get_hash(&self) -> u64 {
        self.hash
            ^ if let Some(ep) = self.en_passant {
                Zobrist::en_passant(ep.get_file(), !self.side_to_move)
            } else {
                0
            }
            ^ Zobrist::castles(
                self.castle_rights[self.side_to_move.to_index()],
                self.side_to_move,
            )
            ^ Zobrist::castles(
                self.castle_rights[(!self.side_to_move).to_index()],
                !self.side_to_move,
            )
    }

    /// Get a pawn hash of the board (a hash that only changes on color change and pawn moves).
    pub fn get_pawn_hash(&self) -> u64 {
        0
    }

    /// What piece is on a particular `Square`?  Is there even one?
    ///
    /// ```
    /// use chess::{Board, Piece, Square, Rank, File};
    ///
    /// let board = Board::default();
    ///
    /// let sq1 = Square::make_square(Rank::First, File::A);
    /// let sq2 = Square::make_square(Rank::Fourth, File::D);
    ///
    /// assert_eq!(board.piece_on(sq1), Some(Piece::Rook));
    /// assert_eq!(board.piece_on(sq2), None);
    /// ```
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        let opp = BitBoard::from_square(square);
        if self.combined() & opp == EMPTY {
            None
        } else {
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
                    Some(Piece::Pawn)
                } else if self.pieces(Piece::Knight) & opp != EMPTY {
                    Some(Piece::Knight)
                } else {
                    Some(Piece::Bishop)
                }
            } else {
                if self.pieces(Piece::Rook) & opp != EMPTY {
                    Some(Piece::Rook)
                } else if self.pieces(Piece::Queen) & opp != EMPTY {
                    Some(Piece::Queen)
                } else {
                    Some(Piece::King)
                }
            }
        }
    }

    /// Unset the en_passant square.
    fn remove_ep(&mut self) {
        self.en_passant = None;
    }

    /// Give me the en_passant square, if it exists.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, Rank, File};
    ///
    /// let move1 = ChessMove::new(Square::make_square(Rank::Second, File::D),
    ///                            Square::make_square(Rank::Fourth, File::D),
    ///                            None);
    ///
    /// let move2 = ChessMove::new(Square::make_square(Rank::Seventh, File::H),
    ///                            Square::make_square(Rank::Fifth, File::H),
    ///                            None);
    ///
    /// let move3 = ChessMove::new(Square::make_square(Rank::Fourth, File::D),
    ///                            Square::make_square(Rank::Fifth, File::D),
    ///                            None);
    ///
    /// let move4 = ChessMove::new(Square::make_square(Rank::Seventh, File::E),
    ///                            Square::make_square(Rank::Fifth, File::E),
    ///                            None);
    ///
    /// let board = Board::default().make_move_new(move1)
    ///                             .make_move_new(move2)
    ///                             .make_move_new(move3)
    ///                             .make_move_new(move4);
    ///
    /// assert_eq!(board.en_passant(), Some(Square::make_square(Rank::Fifth, File::E)));
    /// ```
    pub fn en_passant(self) -> Option<Square> {
        self.en_passant
    }

    /// Set the en_passant square.  Note: This must only be called when self.en_passant is already
    /// None.
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
    /// use chess::{Board, ChessMove, Square, Rank, File};
    ///
    /// let move1 = ChessMove::new(Square::make_square(Rank::Second, File::E),
    ///                            Square::make_square(Rank::Fourth, File::E),
    ///                            None);
    ///
    /// let move2 = ChessMove::new(Square::make_square(Rank::Second, File::E),
    ///                            Square::make_square(Rank::Fifth, File::E),
    ///                            None);
    ///
    /// let board = Board::default();
    ///
    /// assert_eq!(board.legal(move1), true);
    /// assert_eq!(board.legal(move2), false);
    pub fn legal(&self, m: ChessMove) -> bool {
        MoveGen::new_legal(&self).find(|x| *x == m).is_some()
    }

    /// Make a chess move onto a new board.
    ///
    /// panic!() if king is captured.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, Rank, File, Color};
    ///
    /// let m = ChessMove::new(Square::make_square(Rank::Second, File::D),
    ///                        Square::make_square(Rank::Fourth, File::D),
    ///                        None);
    ///
    /// let board = Board::default();
    /// assert_eq!(board.make_move_new(m).side_to_move(), Color::Black);
    /// ```
    pub fn make_move_new(&self, m: ChessMove) -> Board {
        let mut result = unsafe { mem::uninitialized() };
        self.make_move(m, &mut result);
        result
    }

    /// Make a chess move onto an already allocated `Board`.
    ///
    /// panic!() if king is captured.
    ///
    /// ```
    /// use chess::{Board, ChessMove, Square, Rank, File, Color};
    ///
    /// let m = ChessMove::new(Square::make_square(Rank::Second, File::D),
    ///                        Square::make_square(Rank::Fourth, File::D),
    ///                        None);
    ///
    /// let board = Board::default();
    /// let mut result = Board::default();
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

        result.remove_their_castle_rights(CastleRights::square_to_castle_rights(
            !self.side_to_move,
            dest,
        ));

        result.remove_my_castle_rights(CastleRights::square_to_castle_rights(
            self.side_to_move,
            source,
        ));

        let opp_king = result.pieces(Piece::King) & result.color_combined(!result.side_to_move);

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
            let index = dest.get_file().to_index();
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
        result.hash ^= Zobrist::color();
    }

    /// Update the pin information.
    fn update_pin_info(&mut self) {
        self.pinned = EMPTY;
        self.checkers = EMPTY;

        let ksq = (self.pieces(Piece::King) & self.color_combined(self.side_to_move)).to_square();

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
    pub fn pinned(&self) -> &BitBoard {
        &self.pinned
    }

    /// Give me the `Bitboard` of the pieces putting me in check.
    pub fn checkers(&self) -> &BitBoard {
        &self.checkers
    }
}
