use bitboard::{BitBoard, EMPTY};
use cache_table::CacheTable;
use castle_rights::CastleRights;
use chess_move::ChessMove;
use color::{Color, ALL_COLORS, NUM_COLORS};
use construct;
use file::File;
use magic::{
    between, get_adjacent_files, get_bishop_moves, get_bishop_rays, get_file, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_moves, get_rank, get_rook_moves, get_rook_rays,
    line,
};
use movegen::*;
use piece::{Piece, ALL_PIECES, NUM_PIECES};
use rank::Rank;
use square::{Square, ALL_SQUARES};
use std::fmt;
use zobrist::Zobrist;

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
    pawn_hash: u64,
    en_passant: Option<Square>,
}

/// What is the status of this game?
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

/// Never Call Directly!
///
/// Generate the pseudo-legal moves (moves that *may* leave you in check) for a particular piece
/// Do this as a macro for optimization purposes.
///
/// You must pass in the piece type, the source square, the color of the piece to move, and the
/// combined `BitBoard` which represents blocking pieces.
macro_rules! pseudo_legal_moves {
    ($piece_type:expr, $src:expr, $color:expr, $combined:expr) => {
        match $piece_type {
            Piece::Pawn => get_pawn_moves($src, $color, $combined),
            Piece::Knight => get_knight_moves($src),
            Piece::Bishop => get_bishop_moves($src, $combined),
            Piece::Rook => get_rook_moves($src, $combined),
            Piece::Queen => get_bishop_moves($src, $combined) ^ get_rook_moves($src, $combined),
            Piece::King => get_king_moves($src),
        }
    };
}

macro_rules! push_pawn {
    ($move_list:expr, $index:expr) => {{
        #[inline(always)]
        |src, bb, promotion| {
            if promotion {
                for dest in bb {
                    unsafe {
                        *$move_list.get_unchecked_mut($index) =
                            ChessMove::new(src, dest, Some(Piece::Knight));
                        *$move_list.get_unchecked_mut($index + 1) =
                            ChessMove::new(src, dest, Some(Piece::Bishop));
                        *$move_list.get_unchecked_mut($index + 2) =
                            ChessMove::new(src, dest, Some(Piece::Queen));
                        *$move_list.get_unchecked_mut($index + 3) =
                            ChessMove::new(src, dest, Some(Piece::Rook));
                        $index += 4;
                    }
                }
            } else {
                for dest in bb {
                    unsafe {
                        *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                        $index += 1;
                    }
                }
            }
        }
    }};
}

macro_rules! push {
    ($move_list:expr, $index:expr) => {{
        #[inline(always)]
        |src, bb, _promotion| {
            for dest in bb {
                unsafe {
                    *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                    $index += 1;
                }
            }
        }
    }};
}

/// Never Call Directly!
///
/// Enumerate all legal moves for a particular board.
///
/// You must pass in:
///  * a `Board`
///  * an array of `ChessMove` objects big enough to store the moves
///  ** Note: If the array of `ChessMove`s is not large enough, the program will seg. fault.
///  * the current index you want to write to in $move_list
///  * a `BitBoard` mask which represents squares you want to land on
macro_rules! enumerate_moves {
    ($board:expr, $move_list:expr, $index:expr, $mask:expr) => {{
        let checkers = $board.checkers();
        let combined = $board.combined();
        let color = $board.side_to_move();
        let my_pieces = $board.color_combined(color);
        let ksq = ($board.pieces(Piece::King) & my_pieces).to_square();
        if checkers == EMPTY {
            PawnType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push_pawn!($move_list, $index),
            );
            KnightType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            BishopType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            RookType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            QueenType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            KingType::legals::<NotInCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
        } else if checkers.popcnt() == 1 {
            PawnType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push_pawn!($move_list, $index),
            );
            KnightType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            BishopType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            RookType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            QueenType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
            KingType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
        } else {
            KingType::legals::<InCheckType, _>(
                $board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                push!($move_list, $index),
            );
        }
    }};
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
            pawn_hash: 0,
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
                board.pawn_hash ^= Zobrist::color();
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

        board.hash ^= Zobrist::castles(board.castle_rights[Color::Black.to_index()], Color::Black);
        board.hash ^= Zobrist::castles(board.castle_rights[Color::White.to_index()], Color::White);

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
    /// board = board.make_move(ChessMove::new(Square::make_square(Rank::Second, File::E),
    ///                                        Square::make_square(Rank::Fourth, File::E),
    ///                                        None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move(ChessMove::new(Square::make_square(Rank::Seventh, File::F),
    ///                                        Square::make_square(Rank::Sixth, File::F),
    ///                                        None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move(ChessMove::new(Square::make_square(Rank::Second, File::D),
    ///                                        Square::make_square(Rank::Fourth, File::D),
    ///                                        None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move(ChessMove::new(Square::make_square(Rank::Seventh, File::G),
    ///                                        Square::make_square(Rank::Fifth, File::G),
    ///                                        None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Ongoing);
    ///
    /// board = board.make_move(ChessMove::new(Square::make_square(Rank::First, File::D),
    ///                                        Square::make_square(Rank::Fifth, File::H),
    ///                                        None));
    ///
    /// assert_eq!(board.status(), BoardStatus::Checkmate);
    /// ```
    pub fn status(&self) -> BoardStatus {
        let moves =
            self.enumerate_moves(&mut [ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256]);
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
    pub fn combined(&self) -> BitBoard {
        self.combined
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
    pub fn color_combined(&self, color: Color) -> BitBoard {
        unsafe { *self.color_combined.get_unchecked(color.to_index()) }
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
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        unsafe { *self.pieces.get_unchecked(piece.to_index()) }
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
    /// board = board.make_move(move1)
    ///              .make_move(move2)
    ///              .make_move(move3)
    ///              .make_move(move4);
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
            self.hash ^= Zobrist::castles(self.castle_rights(color), color);
            *self.castle_rights.get_unchecked_mut(color.to_index()) =
                self.castle_rights(color).add(add);
            self.hash ^= Zobrist::castles(self.castle_rights(color), color);
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
            self.hash ^= Zobrist::castles(self.castle_rights(color), color);
            *self.castle_rights.get_unchecked_mut(color.to_index()) =
                self.castle_rights(color).remove(remove);
            self.hash ^= Zobrist::castles(self.castle_rights(color), color);
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
            match piece {
                Piece::Pawn => {
                    self.hash ^= Zobrist::piece(piece, bb.to_square(), color);
                    self.pawn_hash ^= Zobrist::piece(piece, bb.to_square(), color);
                }
                _ => {
                    self.hash ^= Zobrist::piece(piece, bb.to_square(), color);
                }
            }
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
            result.pawn_hash ^= Zobrist::color();
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
        if combined != self.combined() {
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
    }

    /// Get a pawn hash of the board (a hash that only changes on color change and pawn moves).
    pub fn get_pawn_hash(&self) -> u64 {
        self.pawn_hash
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
            // naiive algorithm
            /*
            for p in ALL_PIECES {
                if self.pieces(*p) & opp {
                    return p;
                }
            } */
            if (self.pieces(Piece::Pawn) ^ self.pieces(Piece::Knight) ^ self.pieces(Piece::Bishop))
                & opp
                == opp
            {
                if self.pieces(Piece::Pawn) & opp == opp {
                    Some(Piece::Pawn)
                } else if self.pieces(Piece::Knight) & opp == opp {
                    Some(Piece::Knight)
                } else {
                    Some(Piece::Bishop)
                }
            } else {
                if self.pieces(Piece::Rook) & opp == opp {
                    Some(Piece::Rook)
                } else if self.pieces(Piece::Queen) & opp == opp {
                    Some(Piece::Queen)
                } else {
                    Some(Piece::King)
                }
            }
        }
    }

    /// Test the legal move generation by brute-forcing all legal moves.
    fn enumerate_moves_brute_force(&self, moves: &mut [ChessMove; 256]) -> usize {
        let mut index = 0;
        for source in ALL_SQUARES.iter() {
            for dest in ALL_SQUARES.iter() {
                if self.legal(ChessMove::new(*source, *dest, None)) {
                    moves[index] = ChessMove::new(*source, *dest, None);
                    index += 1;
                }
                for promotion in ALL_PIECES.iter() {
                    if self.legal(ChessMove::new(*source, *dest, Some(*promotion))) {
                        moves[index] = ChessMove::new(*source, *dest, Some(*promotion));
                        index += 1;
                    }
                }
            }
        }
        index
    }

    /// Give me all the legal moves for this board.
    ///
    /// Note: You may want to build a `MoveGen` structure to iterate over
    ///       the moves instead.
    ///
    /// Additionally, you must allocate the move array yourself if you want to call this function.
    /// it massively helps with performance to reuse that array.
    ///
    /// ```
    /// use chess::{Board, ChessMove};
    ///
    /// let board = Board::default();
    /// let mut moves = [ChessMove::default(); 256];
    /// let count = board.enumerate_moves(&mut moves);
    /// assert_eq!(count, 20);
    /// ```
    pub fn enumerate_moves(&self, moves: &mut [ChessMove; 256]) -> usize {
        let mut index = 0usize;
        let not_my_pieces = !self.color_combined(self.side_to_move);
        enumerate_moves!(*self, moves, index, not_my_pieces);
        index
    }

    /// Unset the en_passant square.
    fn remove_ep(&mut self) {
        match self.en_passant {
            None => {}
            Some(sq) => {
                self.en_passant = None;
                self.hash ^= Zobrist::en_passant(sq.get_file(), !self.side_to_move);
                self.pawn_hash ^= Zobrist::en_passant(sq.get_file(), !self.side_to_move);
            }
        }
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
    /// let board = Board::default().make_move(move1)
    ///                             .make_move(move2)
    ///                             .make_move(move3)
    ///                             .make_move(move4);
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
            self.hash ^= Zobrist::en_passant(sq.get_file(), self.side_to_move);
            self.pawn_hash ^= Zobrist::en_passant(sq.get_file(), self.side_to_move);
        }
    }

    /// Is a particular move legal?
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
        // Do you have a piece on that source square?
        if self.color_combined(self.side_to_move) & BitBoard::from_square(m.get_source()) == EMPTY {
            return false;
        }

        if m.get_source() == m.get_dest() {
            return false;
        }

        let piece = self.piece_on(m.get_source()).unwrap();

        // Are you trying to promote?  Also, can you promote?
        match m.get_promotion() {
            None => {
                if piece == Piece::Pawn
                    && (m.get_dest().get_rank() == self.side_to_move.to_their_backrank())
                {
                    return false;
                }
            }
            Some(Piece::Pawn) => {
                return false;
            }
            Some(Piece::King) => {
                return false;
            }
            Some(_) => {
                if piece != Piece::Pawn {
                    return false;
                }
                if m.get_dest().get_rank() != self.side_to_move.to_their_backrank() {
                    return false;
                }
            }
        }

        if self.checkers.popcnt() >= 2 || piece == Piece::King {
            // double-check means only the king can move anyways
            if self.checkers == EMPTY {
                // must be a king move, because popcnt() == 0
                // If the piece is a king, can we castle?
                let ksq = m.get_source();

                // If we can castle kingside, and we're trying to castle kingside
                if self.my_castle_rights().has_kingside() && m.get_dest() == ksq.uright().uright() {
                    // make sure the squares that need to be empty are empty
                    if (self.combined()
                        & self.my_castle_rights().kingside_squares(self.side_to_move))
                        == EMPTY
                    {
                        // is the castle legal?
                        if self.legal_king_move(ksq.uright())
                            && self.legal_king_move(ksq.uright().uright())
                        {
                            return true;
                        }
                    }
                }

                // same thing, but queenside
                if self.my_castle_rights().has_queenside() && m.get_dest() == ksq.uleft().uleft() {
                    // are the squares empty?
                    if (self.combined()
                        & self.my_castle_rights().queenside_squares(self.side_to_move))
                        == EMPTY
                    {
                        // is the queenside castle legal?
                        if self.legal_king_move(ksq.uleft())
                            && self.legal_king_move(ksq.uleft().uleft())
                        {
                            return true;
                        }
                    }
                }
            }

            // only king moves are legal, and even then, you need to check to see if that
            // particular king move is legal
            match piece {
                Piece::King => {
                    let moves = pseudo_legal_moves!(
                        piece,
                        m.get_source(),
                        self.side_to_move,
                        self.combined()
                    ) & !self.color_combined(self.side_to_move);
                    return moves & BitBoard::from_square(m.get_dest()) != EMPTY
                        && self.legal_king_move(m.get_dest());
                }
                _ => {
                    return false;
                }
            };
        } else if self.checkers != EMPTY {
            // single-check
            // Are you pinned?  Because, if so, you can't move at all (because we are in check)
            if self.pinned & BitBoard::from_square(m.get_source()) != EMPTY {
                return false;
            }

            // If it's a pawn, and the en_passant rule is in effect, and the passed pawn is the
            // checker, see if this is said legal passed pawn move
            if piece == Piece::Pawn && self.en_passant.is_some() {
                // grab the passed pawn square
                let ep_sq = self.en_passant.unwrap();

                // make sure the passed pawn is the checker
                if (self.checkers & BitBoard::from_square(ep_sq)) != EMPTY {
                    // grab the rank for the passed pawn (to see if we can capture it)
                    let rank = get_rank(ep_sq.get_rank());

                    // get all the squares where a pawn could be to capture this passed pawn
                    let passed_pawn_pieces = get_adjacent_files(ep_sq.get_file()) & rank;

                    // if we are on one of those squares...
                    if passed_pawn_pieces & BitBoard::from_square(m.get_source()) != EMPTY {
                        // get the destination square we'd have to be trying to move to in order to
                        // capture this passed pawn
                        let dest = ep_sq.uforward(self.side_to_move);

                        // if we are trying to move there...
                        if dest == m.get_dest() {
                            // see if the move is legal (in that it doesn't leave us in check).
                            return self.legal_ep_move(m.get_source(), m.get_dest());
                        }
                    }
                }
            }

            // Ok, you can move, but only if that move captures the checker OR places the piece
            // between the checker and the king
            // Also, you can't capture your own pieces (not sure if that's actually relevant here)
            let moves =
                pseudo_legal_moves!(piece, m.get_source(), self.side_to_move, self.combined())
                    & !self.color_combined(self.side_to_move)
                    & (self.checkers | between(
                        self.checkers.to_square(),
                        (self.pieces(Piece::King) & self.color_combined(self.side_to_move))
                            .to_square(),
                    ));
            return moves & BitBoard::from_square(m.get_dest()) != EMPTY;
        } else {
            // not in check
            // check for the passed pawn rule (similar to above, but slightly faster)

            // If it's a pawn, and the en_passant rule is in effect, and the passed pawn is the
            // checker, see if this is said legal passed pawn move
            if piece == Piece::Pawn && self.en_passant.is_some() {
                // grab the passed pawn square
                let ep_sq = self.en_passant.unwrap();

                // grab the rank for the passed pawn (to see if we can capture it)
                let rank = get_rank(ep_sq.get_rank());

                // get all the squares where a pawn could be to capture this passed pawn
                let passed_pawn_pieces = get_adjacent_files(ep_sq.get_file()) & rank;

                // if we are on one of those squares...
                if passed_pawn_pieces & BitBoard::from_square(m.get_source()) != EMPTY {
                    // get the destination square we'd have to be trying to move to in order to
                    // capture this passed pawn
                    let dest = ep_sq.uforward(self.side_to_move);

                    // if we are trying to move there...
                    if dest == m.get_dest() {
                        // see if the move is legal (in that it doesn't leave us in check).
                        return self.legal_ep_move(m.get_source(), m.get_dest());
                    }
                }
            }

            // If you are pinned, you can move, but only along the line between your king and
            // yourself
            // If you are not pinned, you can move anywhere
            // BUT, you cannot capture your own pieces
            let move_mask = !self.color_combined(self.side_to_move) & if self.pinned
                & BitBoard::from_square(m.get_source())
                != EMPTY
            {
                line(
                    m.get_source(),
                    (self.pieces(Piece::King) & self.color_combined(self.side_to_move)).to_square(),
                )
            } else {
                !EMPTY
            };
            let moves =
                pseudo_legal_moves!(piece, m.get_source(), self.side_to_move, self.combined())
                    & move_mask;
            return moves & BitBoard::from_square(m.get_dest()) != EMPTY;
        }
    }

    /// This function checks the legality *only for moves generated by `MoveGen`*.
    ///
    /// Calling this function for moves not generated by `MoveGen` will result in possibly
    /// incorrect results, and making that move on the `Board` will result in undefined behavior.
    /// This function may panic! if these rules are not followed.
    ///
    /// If you are validating a move from a user, you should call the .legal() function.
    pub fn legal_quick(&self, chess_move: ChessMove) -> bool {
        let piece = self.piece_on(chess_move.get_source()).unwrap();
        match piece {
            Piece::Rook => true,
            Piece::Bishop => true,
            Piece::Knight => true,
            Piece::Queen => true,
            Piece::Pawn => {
                if chess_move.get_source().get_file() != chess_move.get_dest().get_file()
                    && self.piece_on(chess_move.get_dest()).is_none()
                {
                    // en-passant
                    self.legal_ep_move(chess_move.get_source(), chess_move.get_dest())
                } else {
                    true
                }
            }
            Piece::King => {
                let bb = between(chess_move.get_source(), chess_move.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !self.legal_king_move(bb.to_square()) {
                        false
                    } else {
                        self.legal_king_move(chess_move.get_dest())
                    }
                } else {
                    self.legal_king_move(chess_move.get_dest())
                }
            }
        }
    }

    /// Make a chess move.
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
    /// assert_eq!(board.make_move(m).side_to_move(), Color::Black);
    /// ```
    pub fn make_move(&self, m: ChessMove) -> Board {
        let mut result = *self;
        let source = BitBoard::from_square(m.get_source());
        let dest = BitBoard::from_square(m.get_dest());
        let moved = self.piece_on(m.get_source()).unwrap();

        result.xor(moved, source, self.side_to_move);
        result.xor(moved, dest, self.side_to_move);
        let captured = self.piece_on(m.get_dest());

        match captured {
            None => {}
            Some(Piece::King) => panic!(),
            Some(p) => {
                result.xor(p, dest, !self.side_to_move);
                if p == Piece::Rook {
                    // if I capture their rook, and their rook has not moved yet, remove the castle
                    // rights for that side of the board
                    if dest & result
                        .their_castle_rights()
                        .unmoved_rooks(!result.side_to_move)
                        != EMPTY
                    {
                        result.remove_their_castle_rights(
                            CastleRights::rook_square_to_castle_rights(m.get_dest()),
                        );
                    }
                }
            }
        }

        result.remove_ep();
        result.checkers = EMPTY;
        result.pinned = EMPTY;

        match moved {
            Piece::King => {
                result.remove_my_castle_rights(CastleRights::Both);

                // if we castle, move the rook over too!
                if m.get_source().get_file() == File::E && m.get_dest().get_file() == File::C {
                    // queenside castle
                    result.xor(
                        Piece::Rook,
                        BitBoard::set(self.side_to_move.to_my_backrank(), File::A),
                        self.side_to_move,
                    );
                    result.xor(
                        Piece::Rook,
                        BitBoard::set(self.side_to_move.to_my_backrank(), File::D),
                        self.side_to_move,
                    );
                } else if m.get_source().get_file() == File::E && m.get_dest().get_file() == File::G
                {
                    // kingside castle
                    result.xor(
                        Piece::Rook,
                        BitBoard::set(self.side_to_move.to_my_backrank(), File::H),
                        self.side_to_move,
                    );
                    result.xor(
                        Piece::Rook,
                        BitBoard::set(self.side_to_move.to_my_backrank(), File::F),
                        self.side_to_move,
                    );
                }
            }

            Piece::Pawn => {
                // e.p. capture.  the capture variable is 'None' because no piece is on the
                // destination square
                if m.get_source().get_file() != m.get_dest().get_file() && captured.is_none() {
                    result.xor(
                        Piece::Pawn,
                        BitBoard::from_square(self.en_passant.unwrap()),
                        !self.side_to_move,
                    );
                }

                match m.get_promotion() {
                    None => {
                        // double-move
                        if (m.get_source().get_rank() == Rank::Second
                            && m.get_dest().get_rank() == Rank::Fourth)
                            || (m.get_source().get_rank() == Rank::Seventh
                                && m.get_dest().get_rank() == Rank::Fifth)
                        {
                            result.set_ep(m.get_dest());
                        }

                        // could be check!
                        if get_pawn_attacks(
                            m.get_dest(),
                            result.side_to_move,
                            result.pieces(Piece::King)
                                & result.color_combined(!result.side_to_move),
                        ) != EMPTY
                        {
                            result.checkers ^= BitBoard::from_square(m.get_dest());
                        }
                    }

                    Some(Piece::Knight) => {
                        result.xor(Piece::Pawn, dest, self.side_to_move);
                        result.xor(Piece::Knight, dest, self.side_to_move);

                        // promotion to a knight check is handled specially because checks from all other
                        // pieces are handled down below automatically
                        if (get_knight_moves(m.get_dest())
                            & result.pieces(Piece::King)
                            & result.color_combined(!result.side_to_move))
                            != EMPTY
                        {
                            result.checkers ^= BitBoard::from_square(m.get_dest());
                        }
                    }

                    Some(p) => {
                        result.xor(Piece::Pawn, dest, self.side_to_move);
                        result.xor(p, dest, self.side_to_move);
                    }
                }
            }

            Piece::Knight => {
                if (get_knight_moves(m.get_dest())
                    & result.pieces(Piece::King)
                    & result.color_combined(!result.side_to_move))
                    != EMPTY
                {
                    result.checkers ^= BitBoard::from_square(m.get_dest());
                }
            }

            Piece::Rook => {
                // if I move my rook, remove my castle rights on that side
                if source & result.my_castle_rights().unmoved_rooks(result.side_to_move) == source {
                    result.remove_my_castle_rights(CastleRights::rook_square_to_castle_rights(
                        m.get_source(),
                    ));
                }
            }
            _ => {}
        }

        // now, lets see if we're in check or pinned
        let ksq =
            (result.pieces(Piece::King) & result.color_combined(!result.side_to_move)).to_square();

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
        result.pawn_hash ^= Zobrist::color();

        result
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

    /// Run a perft-test with a cache and the chess moves and cache tables already allocated for
    /// each depth.
    fn internal_perft_cache(
        &self,
        depth: u64,
        move_list: &mut Vec<[ChessMove; 256]>,
        caches: &mut Vec<CacheTable<u64>>,
    ) -> u64 {
        let cur = unsafe { caches.get_unchecked(depth as usize) }.get(self.hash);
        match cur {
            Some(x) => x,
            None => {
                let mut result = 0;
                if depth == 0 {
                    result = 1;
                } else if depth == 1 {
                    unsafe {
                        result = self.enumerate_moves(move_list.get_unchecked_mut(depth as usize))
                            as u64;
                    }
                } else {
                    let length = unsafe {
                        self.enumerate_moves(move_list.get_unchecked_mut(depth as usize))
                    };
                    for x in 0..length {
                        let m =
                            unsafe { *move_list.get_unchecked(depth as usize).get_unchecked(x) };
                        let cur =
                            self.make_move(m)
                                .internal_perft_cache(depth - 1, move_list, caches);
                        result += cur;
                    }
                }
                unsafe { caches.get_unchecked_mut(depth as usize) }.add(self.hash, result);
                result
            }
        }
    }

    /// Run a perft-test with the [ChessMove; 256] already allocated for each depth.
    fn internal_perft(&self, depth: u64, move_list: &mut Vec<[ChessMove; 256]>) -> u64 {
        let mut result = 0;
        if depth == 0 {
            1
        } else if depth == 1 {
            unsafe { self.enumerate_moves(move_list.get_unchecked_mut(depth as usize)) as u64 }
        } else {
            let length =
                unsafe { self.enumerate_moves(move_list.get_unchecked_mut(depth as usize)) };
            for x in 0..length {
                let m = unsafe { *move_list.get_unchecked(depth as usize).get_unchecked(x) };
                let cur = self.make_move(m).internal_perft(depth - 1, move_list);
                result += cur;
            }
            result
        }
    }

    /// Run a perft-test with the [ChessMove; 256] already allocated for each depth... BUT: brute
    /// force the move list.
    fn internal_perft_brute_force(&self, depth: u64, move_list: &mut Vec<[ChessMove; 256]>) -> u64 {
        let mut result = 0;
        let actual = if depth == 0 {
            1
        } else if depth == 1 {
            unsafe {
                self.enumerate_moves_brute_force(move_list.get_unchecked_mut(depth as usize)) as u64
            }
        } else {
            let length = unsafe {
                self.enumerate_moves_brute_force(move_list.get_unchecked_mut(depth as usize))
            };
            for x in 0..length {
                let m = unsafe { *move_list.get_unchecked(depth as usize).get_unchecked(x) };
                let cur = self.make_move(m).internal_perft(depth - 1, move_list);
                result += cur;
            }
            result
        };

        // test the result with the perft() function
        if actual != self.perft(depth) {
            if depth == 1 {
                println!(
                    "Got {} moves. Correct is {} moves\n{}",
                    actual,
                    self.perft(depth),
                    self
                );
            } else {
                let good = self.enumerate_moves(&mut move_list[depth as usize]);
                let bad = self.enumerate_moves_brute_force(&mut move_list[depth as usize]);
                if good != bad {
                    println!("Got {} moves. Correct is {} moves\n{}", bad, good, self);
                }
            }
        }
        result
    }

    /// Run a perft-test.
    pub fn perft(&self, depth: u64) -> u64 {
        let mut move_list: Vec<[ChessMove; 256]> = Vec::new();
        for _ in 0..(depth + 1) {
            move_list.push([ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256]);
        }
        self.internal_perft(depth, &mut move_list)
    }

    /// Run a perft-test using brute force move generation.
    pub fn perft_brute_force(&self, depth: u64) -> u64 {
        let mut move_list: Vec<[ChessMove; 256]> = Vec::new();
        for _ in 0..(depth + 1) {
            move_list.push([ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256]);
        }
        self.internal_perft_brute_force(depth, &mut move_list)
    }

    /// Run a perft test with a cache table.
    pub fn perft_cache(&self, depth: u64, cache_size_per_depth: usize) -> u64 {
        let mut move_list: Vec<[ChessMove; 256]> = Vec::new();
        let mut caches: Vec<CacheTable<u64>> = Vec::new();
        for _ in 0..(depth + 1) {
            move_list.push([ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256]);
            caches.push(CacheTable::new(cache_size_per_depth, 0));
        }
        self.internal_perft_cache(depth, &mut move_list, &mut caches)
    }

    /// Give me the `BitBoard` of my pinned pieces.
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    /// Give me the `Bitboard` of the pieces putting me in check.
    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    /// Is a particular king move legal?
    #[inline(always)]
    pub fn legal_king_move(&self, dest: Square) -> bool {
        let combined = self.combined()
            ^ (self.pieces(Piece::King) & self.color_combined(self.side_to_move))
            | BitBoard::from_square(dest);

        let mut attackers = EMPTY;

        let rooks = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen))
            & self.color_combined(!self.side_to_move);

        if (get_rook_rays(dest) & rooks) != EMPTY {
            attackers |= get_rook_moves(dest, combined) & rooks;
        }

        let bishops = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen))
            & self.color_combined(!self.side_to_move);

        if (get_bishop_rays(dest) & bishops) != EMPTY {
            attackers |= get_bishop_moves(dest, combined) & bishops;
        }

        let knight_rays = get_knight_moves(dest);
        attackers |=
            knight_rays & self.pieces(Piece::Knight) & self.color_combined(!self.side_to_move);

        let king_rays = get_king_moves(dest);
        attackers |= king_rays & self.pieces(Piece::King) & self.color_combined(!self.side_to_move);

        if attackers != EMPTY {
            return false;
        }
        attackers |= get_pawn_attacks(
            dest,
            self.side_to_move,
            self.pieces(Piece::Pawn) & self.color_combined(!self.side_to_move),
        );

        return attackers == EMPTY;
    }

    /// Is a particular en-passant capture legal?
    pub fn legal_ep_move(&self, source: Square, dest: Square) -> bool {
        let combined = self.combined()
            ^ BitBoard::from_square(self.en_passant.unwrap())
            ^ BitBoard::from_square(source)
            ^ BitBoard::from_square(dest);

        let ksq = (self.pieces(Piece::King) & self.color_combined(self.side_to_move)).to_square();

        let rooks = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen))
            & self.color_combined(!self.side_to_move);

        if (get_rook_rays(ksq) & rooks) != EMPTY {
            if (get_rook_moves(ksq, combined) & rooks) != EMPTY {
                return false;
            }
        }

        let bishops = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen))
            & self.color_combined(!self.side_to_move);

        if (get_bishop_rays(ksq) & bishops) != EMPTY {
            if (get_bishop_moves(ksq, combined) & bishops) != EMPTY {
                return false;
            }
        }

        return true;
    }

    /// Run every type of perft test, and panic! if the leaf-node count of any version is not equal
    /// to `result`.
    pub fn perft_test(fen: String, depth: u64, result: u64) {
        construct::construct();
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.perft(depth), result);
        assert_eq!(board.perft_cache(depth, 65536), result);
        assert_eq!(board.perft_brute_force(depth), result);
    }
}

#[test]
fn perft_kiwipete() {
    Board::perft_test(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(),
        5,
        193690690,
    );
}

#[test]
fn perft_1() {
    Board::perft_test("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064); // Invalid FEN
}

#[test]
fn perft_2() {
    Board::perft_test("8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064); // Invalid FEN
}

#[test]
fn perft_3() {
    Board::perft_test("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467);
}

#[test]
fn perft_4() {
    Board::perft_test("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467);
}

#[test]
fn perft_5() {
    Board::perft_test("5k2/8/8/8/8/8/8/4K2R w K - 0 1".to_owned(), 6, 661072);
}

#[test]
fn perft_6() {
    Board::perft_test("4k2r/8/8/8/8/8/8/5K2 b k - 0 1".to_owned(), 6, 661072);
}

#[test]
fn perft_7() {
    Board::perft_test("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn perft_8() {
    Board::perft_test("r3k3/8/8/8/8/8/8/3K4 b q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn perft_9() {
    Board::perft_test(
        "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn perft_10() {
    Board::perft_test(
        "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn perft_11() {
    Board::perft_test(
        "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn perft_12() {
    Board::perft_test(
        "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn perft_13() {
    Board::perft_test("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn perft_14() {
    Board::perft_test("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn perft_15() {
    Board::perft_test("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn perft_16() {
    Board::perft_test("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn perft_17() {
    Board::perft_test("4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn perft_18() {
    Board::perft_test("8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn perft_19() {
    Board::perft_test("8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn perft_20() {
    Board::perft_test("8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn perft_21() {
    Board::perft_test("K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn perft_22() {
    Board::perft_test("8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn perft_23() {
    Board::perft_test("8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn perft_24() {
    Board::perft_test("8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn perft_25() {
    Board::perft_test("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

#[test]
fn perft_26() {
    Board::perft_test("8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: String = "".to_owned();
        for rank in (0..8).rev() {
            s.push_str(&(rank + 1).to_string());
            s.push_str(" ");
            for file in 0..8 {
                let sq = Square::make_square(Rank::from_index(rank), File::from_index(file));
                let bb = BitBoard::from_square(sq);
                if self.combined() & bb == EMPTY {
                    s.push_str(" . ");
                } else {
                    let color = if (self.color_combined(Color::White) & bb) == bb {
                        Color::White
                    } else {
                        Color::Black
                    };

                    let mut piece = match self.piece_on(sq).unwrap() {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };
                    if color == Color::White {
                        piece = piece.to_uppercase().last().unwrap();
                    }

                    if bb & self.checkers != EMPTY {
                        s.push_str("c");
                    } else {
                        s.push_str(" ");
                    }
                    s.push(piece);
                    s.push_str(" ");
                }
            }
            s.push_str("\n");
        }
        s.push_str("   A  B  C  D  E  F  G  H\n");
        s.push_str(if self.side_to_move() == Color::White {
            "Whites Turn\n"
        } else {
            "Blacks Turn\n"
        });
        write!(f, "{}", s)
    }
}
