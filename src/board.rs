use bitboard::{BitBoard, EMPTY};
use piece::{Piece, NUM_PIECES, ALL_PIECES};
use color::{Color, NUM_COLORS};
use castle_rights::CastleRights;
use square::{Square, NUM_SQUARES};
use magic::Magic;
use chess_move::ChessMove;
use std::fmt;
use construct;

/// A representation of a chess board.  That's why you're here, right?
#[derive(Copy, Clone)]
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

/// Generate the pseudo-legal moves (moves that *may* leave you in check) for a particular piece
/// Do this as a macro for optimization purposes.
///
/// You must pass in the piece type, the source square, the color of the piece to move, and the
/// combined `BitBoard` which represents blocking pieces.
macro_rules! pseudo_legal_moves {
    ($piece_type:expr, $src:expr, $color:expr, $combined:expr) => {
        match $piece_type {
            Piece::Pawn => Magic::get_pawn_moves($src, $color, $combined),
            Piece::Knight => Magic::get_knight_moves($src),
            Piece::Bishop => Magic::get_bishop_moves($src, $combined),
            Piece::Rook => Magic::get_rook_moves($src, $combined),
            Piece::Queen => Magic::get_bishop_moves($src, $combined) ^ Magic::get_rook_moves($src, $combined),
            Piece::King => Magic::get_king_moves($src)
        } 
    };
}

/// Enumerate all legal moves for a particular board.
///
/// You must pass in:
///  * a `Board`
///  * an array of `ChessMove` objects big enough to store the moves
///  ** Note: If the array of `ChessMove`s is not large enough, the program will seg. fault.
///  * the current index you want to write to in $move_list
///  * a `BitBoard` mask which represents squares you want to land on
macro_rules! enumerate_moves {
    ($board:expr, $move_list:expr, $index:expr, $dest_mask:expr) => { {
        if $board.checkers == EMPTY {
            enumerate_moves_one_piece!($board, Piece::Pawn, false, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Knight, false, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Bishop, false, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Rook, false, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Queen, false, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::King, false, $board.side_to_move, $move_list, $index, $dest_mask);
        } else if $board.checkers.popcnt() == 1 {
            enumerate_moves_one_piece!($board, Piece::Pawn, true, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Knight, true, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Bishop, true, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Rook, true, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::Queen, true, $board.side_to_move, $move_list, $index, $dest_mask);
            enumerate_moves_one_piece!($board, Piece::King, true, $board.side_to_move, $move_list, $index, $dest_mask);
        } else {
            enumerate_moves_one_piece!($board, Piece::King, true, $board.side_to_move, $move_list, $index, $dest_mask);
        }
    } };
}

/// Note: It is absolutely wrong to call the `enumerate_moves_one_piece` macro by itself.  You will
/// get invalid results due to some internal assumptions about when it will be called.
///
/// Enumerate all legal moves for one piece.
///
/// You must pass in:
///  * a `Board`
///  * a `Piece` type
///  * whether or not you are currently in check (a boolean)
///  * your color
///  * an array of `ChessMove` objects big enough to store the moves
///  ** Note: If the array of `ChessMove`s is not large enough, the program will seg. fault.
///  * the current index you want to write to in $move_list
///  * a `BitBoard` mask which represents the squares you want to land on.
macro_rules! enumerate_moves_one_piece {
    ($board:expr, $piece_type:expr, $in_check:expr, $color:expr, $move_list:expr, $index:expr, $dest_mask:expr) => { {
        unsafe {
            let combined = $board.combined();
            let my_pieces = $board.color_combined($color);
            let pieces = $board.pieces($piece_type) & my_pieces;
            let pinned = $board.pinned;
            let checkers = $board.checkers;

            // if the piece is a king, iterate over all pseudo-legal moves, and check to see if it
            // leaves you in check with `legal_king_move`.
            if $piece_type == Piece::King {
                for src in pieces {
                    let moves = pseudo_legal_moves!($piece_type, src, $color, combined) & $dest_mask;
                    for dest in moves {
                        if $board.legal_king_move(dest) {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }
                    }
                }

                // If we are not in check, we may be able to castle.
                // We can do so iff:
                //  * the `Board` structure says we can.
                //  * the squares between my king and my rook are empty.
                //  * no enemy pieces are attacking the squares between the king, and the kings
                //    destination square.
                //  ** This is determined by going to the left or right, and calling
                //     'legal_king_move' for that square.
                if !$in_check {
                    let ksq = ($board.pieces(Piece::King) & $board.color_combined($color)).to_square();

                    if $board.my_castle_rights().has_kingside() && 
                        ($board.combined() & $board.my_castle_rights().kingside_squares($color)) == EMPTY {
                        if $board.legal_king_move(ksq.uright()) &&
                           $board.legal_king_move(ksq.uright().uright()) {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(ksq, ksq.uright().uright(), None);
                            $index += 1;
                        }
                    }

                    if $board.my_castle_rights().has_queenside() && 
                        ($board.combined() & $board.my_castle_rights().queenside_squares($color)) == EMPTY {
                        if $board.legal_king_move(ksq.uleft()) &&
                           $board.legal_king_move(ksq.uleft().uleft()) {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(ksq, ksq.uleft().uleft(), None);
                            $index += 1;
                        }
                    }
                }
            } else {
                // Just a normal piece move.
                let backrank = BitBoard::get_rank($color.to_their_backrank());
                let ksq = ($board.pieces(Piece::King) & $board.color_combined($color)).to_square();

                // if the piece is not pinned:
                //  * And I'm currently in check:
                //  ** I can move to any square between my king and the dude putting me in check.
                //  ** I can catpure the dude putting me in check
                //  ** I will not be at this section of code if I'm in double-check
                //  * And I'm currently NOT in check:
                //  ** I can move anywhere!
                for src in pieces & !pinned { 
                    let moves = pseudo_legal_moves!($piece_type, src, $color, combined) &
                                $dest_mask &
                                (if $in_check { Magic::between(checkers.to_square(), ksq) ^ checkers } else { !EMPTY });

                    // If I'm not a pawn, just add each move to the move list.
                    if $piece_type != Piece::Pawn {
                        for dest in moves {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }
                    } else {
                        // If I am a pawn, add any 'non-promotion' move to the move list.
                        for dest in moves & !backrank {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }

                        // If I am a pawn, add all 'promotion' moves to the move list.
                        for dest in moves & backrank {
                            *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, Some(Piece::Queen));
                            *$move_list.get_unchecked_mut($index+1) = ChessMove::new(src, dest, Some(Piece::Knight));
                            *$move_list.get_unchecked_mut($index+2) = ChessMove::new(src, dest, Some(Piece::Rook));
                            *$move_list.get_unchecked_mut($index+3) = ChessMove::new(src, dest, Some(Piece::Bishop));
                            $index += 4;
                        }
                    }
                }

                // If I'm not in check AND I'm pinned
                //  * I can still move along the line between my pinner and my king
                //  * I can still capture my pinner
                //  * If I'm a knight, I cannot move at all due to the way knights move.
                if !$in_check && $piece_type != Piece::Knight {
                    let king_sq = ($board.pieces(Piece::King) & my_pieces).to_square();

                    // for each pinned piece of this type
                    for src in pieces & pinned {
                        // grab all the moves that put me between my pinner and my king, and
                        // possibly capture my attacker
                        // * Note: Due to how lines work, the line between my pinner and my king is
                        //         the same as the line between ME and my king.  So lets use the
                        //         second definition because it's easier to code.
                        let moves = pseudo_legal_moves!($piece_type, src, $color, combined) &
                                    $dest_mask &
                                    Magic::line(src, king_sq);

                        // Same as above.  If I'm not a pawn, just add all the moves to the moves
                        // list
                        if $piece_type != Piece::Pawn {
                            for dest in moves {
                                *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                                $index += 1;
                            }
                        } else {
                            // If I am a pawn, add all 'non-promotion' moves to the move list.
                            for dest in moves & !backrank {
                                *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                                $index += 1;
                            }

                            // If I am a pawn, add all 'promotion' moves to the move list.
                            for dest in moves & backrank {
                                *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, Some(Piece::Queen));
                                *$move_list.get_unchecked_mut($index+1) = ChessMove::new(src, dest, Some(Piece::Knight));
                                *$move_list.get_unchecked_mut($index+2) = ChessMove::new(src, dest, Some(Piece::Rook));
                                *$move_list.get_unchecked_mut($index+3) = ChessMove::new(src, dest, Some(Piece::Bishop));
                                $index += 4;
                            }
                        }
                    }
                }

                // and, lastly, passed pawn moves
                // What a stupid chess rule...
                // This type of move has its own implementation of legal_*_move, which is called
                // legal_ep_move.
                if $piece_type == Piece::Pawn && $board.en_passant.is_some() {
                    let ep_sq = $board.en_passant.unwrap();
                    if ($in_check && (checkers & BitBoard::from_square(ep_sq)) != EMPTY) || !$in_check {
                        let rank = BitBoard::get_rank(ep_sq.rank());
                        let passed_pawn_pieces = pieces & !pinned & BitBoard::get_adjacent_files(ep_sq.file()) & rank;
                        let dest = ep_sq.uforward($color);
                        for src in passed_pawn_pieces {
                            if $board.legal_ep_move(src, dest) {
                                *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                                $index += 1;
                            }
                        }
                    }
                }
            }
        }
    } };
}

impl Board {
    /// Construct a new `Board` that is completely empty.
    /// Note: This does NOT give you the initial position.  Just a blank slate.
    pub fn new() -> Board {
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
    pub fn from_fen(fen: String) -> Board {
        let mut cur_rank = 7;
        let mut cur_file = 0;
        let mut board: Board = Board::new();

        let tokens: Vec<&str> = fen.split(' ').collect();
        if tokens.len() != 6 { panic!(); }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let ep = tokens[3];
        //let irreversable_moves = tokens[4];
        //let total_moves = tokens[5];

        for x in pieces.chars() {
            match x {
                '/' => {
                    cur_rank -= 1;
                    cur_file = 0
                }, '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    cur_file += (x as u8) - ('0' as u8);
                }, 'r' => { 
                    board.xor(Piece::Rook, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'R' => {
                    board.xor(Piece::Rook, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, 'n' => {
                    board.xor(Piece::Knight, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'N' => {
                    board.xor(Piece::Knight, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, 'b' => {
                    board.xor(Piece::Bishop, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'B' => {
                    board.xor(Piece::Bishop, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, 'p' => {
                    board.xor(Piece::Pawn, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'P' => {
                    board.xor(Piece::Pawn, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, 'q' => {
                    board.xor(Piece::Queen, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'Q' => {
                    board.xor(Piece::Queen, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, 'k' => {
                    board.xor(Piece::King, BitBoard::set(cur_rank, cur_file), Color::Black);
                    cur_file += 1;
                }, 'K' => {
                    board.xor(Piece::King, BitBoard::set(cur_rank, cur_file), Color::White);
                    cur_file += 1;
                }, _ => { panic!(); }
            }
        }
        match side {
            "w" | "W" => board.side_to_move = Color::White,
            "b" | "B" => board.side_to_move = Color::Black,
            _ => panic!()
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

        board.en_passant = Square::from_string(ep.to_owned());

        board.update_pin_info();

        board
    }

    /// Grab the "combined" `BitBoard`.  This is a `BitBoard` with every piece
    pub fn combined(&self) -> BitBoard {
        self.combined
    }

    /// Grab the "color combined" `BitBoard`.  This is a `BitBoard` with every piece of a particular
    /// color.
    pub fn color_combined(&self, color: Color) -> BitBoard {
        unsafe {
            *self.color_combined.get_unchecked(color.to_index())
        }
    }

    /// Grab the "pieces" `BitBoard`.  This is a `BitBoard` with every piece of a particular type.
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        unsafe {
            *self.pieces.get_unchecked(piece.to_index())
        }
    }

    /// Grab the `CastleRights` for a particular side.
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        unsafe {
            *self.castle_rights.get_unchecked(color.to_index())
        }
    }

    /// Who's turn is it?
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Grab my `CastleRights`.
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.side_to_move())
    }

    /// Remove some of my `CastleRights`.
    fn remove_my_castle_rights(&mut self, remove: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut(self.side_to_move.to_index()) = self.my_castle_rights().remove(remove);
        }
    }

    /// My opponents `CastleRights`.
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.side_to_move())
    }

    /// Remove some of my opponents `CastleRights`.
    fn remove_their_castle_rights(&mut self, remove: CastleRights) {
        unsafe {
            *self.castle_rights.get_unchecked_mut((!self.side_to_move).to_index()) = self.their_castle_rights().remove(remove);
        }
    }

    /// Add or remove a piece from the bitboards in this struct.
    fn xor(&mut self, piece: Piece, bb: BitBoard, color: Color) {
        unsafe {
            *self.pieces.get_unchecked_mut(piece.to_index()) ^= bb;
            *self.color_combined.get_unchecked_mut(color.to_index()) ^= bb;
            self.combined ^= bb;
        }
    }

    /// Does this board "make sense"?
    /// Do all the pieces make sense, do the bitboards combine correctly, etc?
    /// This is for sanity checking.
    pub fn is_sane(&self) -> bool {
        for x in ALL_PIECES.iter() {
            for y in ALL_PIECES.iter() {
                if *x != *y {
                    if self.pieces(*x) & self.pieces(*y) != EMPTY {
                        return false;
                    }
                }
            }
        }

        if self.color_combined(Color::White) & self.color_combined(Color::Black) != EMPTY {
            return false;
        }

        let combined = ALL_PIECES.iter().fold(EMPTY, |cur, next| cur | self.pieces(*next));

        return combined == self.combined();
    }

    /// What piece is on a particular `Square`?  Is there even one?
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
            if (self.pieces(Piece::Pawn) ^ self.pieces(Piece::Knight) ^ self.pieces(Piece::Bishop)) & opp == opp {
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

    /// Give me all the legal moves for this board.
    pub fn enumerate_moves(&self, moves: &mut [ChessMove; 256]) -> usize {
        let mut index = 0usize;
        let not_my_pieces = !self.color_combined(self.side_to_move);
        enumerate_moves!(self, moves, index, not_my_pieces);
        index
    }

    /// Make a chess move
    pub fn next(&self, m: ChessMove) -> Board {
        let mut result = *self;
        let source = BitBoard::from_square(m.get_source());
        let dest = BitBoard::from_square(m.get_dest());
        let moved = self.piece_on(m.get_source()).unwrap();

        result.xor(moved, source, self.side_to_move);
        result.xor(moved, dest, self.side_to_move);
        let captured = self.piece_on(m.get_dest());

        match captured {
            None => {},
            Some(p) => {
                result.xor(p, dest, !self.side_to_move);
                if p == Piece::Rook {
                    // if I capture their rook, and their rook has not moved yet, remove the castle
                    // rights for that side of the board
                    if dest & result.their_castle_rights().unmoved_rooks(!result.side_to_move) != EMPTY {
                        result.remove_their_castle_rights(CastleRights::rook_square_to_castle_rights(m.get_dest()));
                    }
                }
            }
        }

        result.en_passant = None;
        result.checkers = EMPTY;
        result.pinned = EMPTY;

        match moved {
            Piece::King => {
                result.remove_my_castle_rights(CastleRights::Both);
                if m.get_source().file().wrapping_sub(m.get_dest().file()) == 2 { // queenside castle
                    result.xor(Piece::Rook, BitBoard::set(self.side_to_move.to_my_backrank(), 0), self.side_to_move);
                    result.xor(Piece::Rook, BitBoard::set(self.side_to_move.to_my_backrank(), 3), self.side_to_move);
                } else if m.get_dest().file().wrapping_sub(m.get_source().file()) == 2 { // kingside castle
                    result.xor(Piece::Rook, BitBoard::set(self.side_to_move.to_my_backrank(), 7), self.side_to_move);
                    result.xor(Piece::Rook, BitBoard::set(self.side_to_move.to_my_backrank(), 5), self.side_to_move);
                }
            }

            Piece::Pawn => {
                // e.p. capture.  the capture variable is 'None' because no piece is on the
                // destination square
                if m.get_source().file() != m.get_dest().file() && captured.is_none() {
                    result.xor(Piece::Pawn, BitBoard::from_square(self.en_passant.unwrap()), !self.side_to_move);
                }

                match m.get_promotion() {
                    None => {
                        // double-move
                        let ranks = (m.get_source().rank() as i8) - (m.get_dest().rank() as i8);
                        if ranks == 2 || ranks == -2 {
                            result.en_passant = Some(m.get_dest());
                        }

                        // could be check!
                        if Magic::get_pawn_attacks(m.get_dest(),
                                                   result.side_to_move,
                                                   result.pieces(Piece::King) &
                                                   result.color_combined(!result.side_to_move)) != EMPTY {
                            result.checkers ^= BitBoard::from_square(m.get_dest());
                        }
                    },

                    Some(Piece::Knight) => {
                        result.xor(Piece::Pawn, dest, self.side_to_move);
                        result.xor(Piece::Knight, dest, self.side_to_move);

                        // promotion to a knight check is handled specially because checks from all other
                        // pieces are handled down below automatically
                        if (Magic::get_knight_moves(m.get_dest()) &
                            result.pieces(Piece::King) &
                            result.color_combined(!result.side_to_move)) != EMPTY {
                            result.checkers ^= BitBoard::from_square(m.get_dest());
                        }
                    },

                    Some(p) => {
                        result.xor(Piece::Pawn, dest, self.side_to_move);
                        result.xor(p, dest, self.side_to_move);
                    }
                }
            }

            Piece::Knight => {
                if (Magic::get_knight_moves(m.get_dest()) &
                    result.pieces(Piece::King) &
                    result.color_combined(!result.side_to_move)) != EMPTY {
                    result.checkers ^= BitBoard::from_square(m.get_dest());
                }
            }

            Piece::Rook => {
                // if I move my rook, remove my castle rights on that side
                if source & result.my_castle_rights().unmoved_rooks(result.side_to_move) == source {
                    result.remove_my_castle_rights(CastleRights::rook_square_to_castle_rights(m.get_source()));
                }
            }
            _ => {}
        }

        // now, lets see if we're in check or pinned
        let ksq = (result.pieces(Piece::King) & result.color_combined(!result.side_to_move)).to_square();

        let pinners = result.color_combined(result.side_to_move) & (
                        (Magic::get_bishop_rays(ksq) &
                            (result.pieces(Piece::Bishop)|result.pieces(Piece::Queen))
                        )|(Magic::get_rook_rays(ksq) &
                            (result.pieces(Piece::Rook)|result.pieces(Piece::Queen))
                        )
                      );

        for sq in pinners {
            let between = Magic::between(sq, ksq) & result.combined();
            if between == EMPTY {
                result.checkers ^= BitBoard::from_square(sq);
            } else if between.popcnt() == 1 {
                result.pinned ^= between;
            }
        }

/*
        if result.in_check() {
            println!("Move left me in check!\nBefore\n{}\nAfter\n{}", self, result);
            panic!();
        } */

        result.side_to_move = !result.side_to_move;
/*
        if !result.is_sane() {
            println!("Insane Board!");
        } */

        result
    }

    /// Update the pin information.
    fn update_pin_info(&mut self) {
        self.pinned = EMPTY;
        self.checkers = EMPTY;

        let ksq = (self.pieces(Piece::King) & self.color_combined(self.side_to_move)).to_square();

        let pinners = self.color_combined(!self.side_to_move) & (
                        (Magic::get_bishop_rays(ksq) &
                            (self.pieces(Piece::Bishop)|self.pieces(Piece::Queen))
                        )|(Magic::get_rook_rays(ksq) &
                            (self.pieces(Piece::Rook)|self.pieces(Piece::Queen))
                        )
                      );

        for sq in pinners {
            let between = Magic::between(sq, ksq) & self.combined();
            if between == EMPTY {
                self.checkers ^= BitBoard::from_square(sq);
            } else if between.popcnt() == 1 {
                self.pinned ^= between;
            }
        }

        self.checkers ^= Magic::get_knight_moves(ksq) &
                         self.color_combined(!self.side_to_move) &
                         self.pieces(Piece::Knight);

        self.checkers ^= Magic::get_pawn_attacks(ksq,
                                                 self.side_to_move,
                                                 self.color_combined(!self.side_to_move) & self.pieces(Piece::Pawn));
    }

    /// Run a perft-test with the [ChessMove; 256] already allocated for each depth.
    fn internal_perft(&self, depth: u64, move_list: &mut Vec<[ChessMove; 256]>) -> u64 {
        let mut result = 0;
        if depth == 0 {
            1
        } else if depth == 1 {
            unsafe {
                self.enumerate_moves(move_list.get_unchecked_mut(depth as usize)) as u64
            }
        } else {
            let length = unsafe { self.enumerate_moves(move_list.get_unchecked_mut(depth as usize)) };
            for x in 0..length {
                let m = unsafe { *move_list.get_unchecked(depth as usize).get_unchecked(x) };
                let cur = self.next(m).internal_perft(depth - 1, move_list);
                result += cur;
            }
            result
        }
    }

    /// Run a perft-test.
    pub fn perft(&self, depth: u64) -> u64{
        let mut move_list: Vec<[ChessMove; 256]> = Vec::new();
        for x in 0..depth {
            move_list.push([ChessMove::new(Square::new(0), Square::new(0), None); 256]);
        }
        self.internal_perft(depth, &mut move_list)
    }

    /// Is a particular king move legal?
    fn legal_king_move(&self, dest: Square) -> bool {
        let combined = self.combined() ^
               (self.pieces(Piece::King) & self.color_combined(self.side_to_move)) |
               BitBoard::from_square(dest);

        let mut attackers = EMPTY;

        let rooks = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen)) & self.color_combined(!self.side_to_move);

        if (Magic::get_rook_rays(dest) & rooks) != EMPTY {
            attackers |= Magic::get_rook_moves(dest, combined) & rooks;
        }

        let bishops = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)) & self.color_combined(!self.side_to_move);

        if (Magic::get_bishop_rays(dest) & bishops) != EMPTY {
            attackers |= Magic::get_bishop_moves(dest, combined) & bishops;
        }

        let knight_rays = Magic::get_knight_moves(dest);
        attackers |= knight_rays &
                     self.pieces(Piece::Knight) &
                     self.color_combined(!self.side_to_move);

        let king_rays = Magic::get_king_moves(dest);
        attackers |= king_rays &
                     self.pieces(Piece::King) &
                     self.color_combined(!self.side_to_move);

        attackers |= Magic::get_pawn_attacks(dest,
                                             self.side_to_move,
                                             self.pieces(Piece::Pawn) & self.color_combined(!self.side_to_move));

        return attackers == EMPTY;
    }

    /// Is a particular en-passant capture legal?
    fn legal_ep_move(&self, source: Square, dest: Square) -> bool {
        let combined = self.combined() ^
                       BitBoard::from_square(self.en_passant.unwrap()) ^
                       BitBoard::from_square(source) ^
                       BitBoard::from_square(dest);

        let ksq = (self.pieces(Piece::King) & self.color_combined(self.side_to_move)).to_square();

        let rooks = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen)) & self.color_combined(!self.side_to_move);

        if (Magic::get_rook_rays(ksq) & rooks) != EMPTY {
            if (Magic::get_rook_moves(ksq, combined) & rooks) != EMPTY {
                return false;
            }
        }

        let bishops = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)) & self.color_combined(!self.side_to_move);

        if (Magic::get_bishop_rays(ksq) & bishops) != EMPTY {
            if (Magic::get_bishop_moves(ksq, combined) & bishops) != EMPTY {
                return false;
            }
        }

        return true;
    }

    pub fn perft_test(fen: String, depth: u64, result: u64) {
        construct::construct();
        let board = Board::from_fen(fen);
        println!("Board:\n{}", board);
        assert_eq!(board.perft(depth), result);
    }
}

#[test]
fn perft_kiwipete() {
    Board::perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(), 5, 193690690);
}

#[test]
fn perft_1() {
    Board::perft_test("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064);
}

#[test]
fn perft_2() {
    Board::perft_test("8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064);
}

#[test]
fn perft_3() {
    Board::perft_test("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467); // INVALID FEN
}

#[test]
fn perft_4() {
    Board::perft_test("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467); // INVALID FEN
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
    Board::perft_test("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(), 4, 1274206);
}

#[test]
fn perft_10() {
    Board::perft_test("r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(), 4, 1274206);
}

#[test]
fn perft_11() {
    Board::perft_test("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(), 4, 1720476);
}

#[test]
fn perft_12() {
    Board::perft_test("r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(), 4, 1720476);
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
            s.push_str(&rank.to_string());
            s.push_str(" ");
            for file in 0..8 {
                let sq = Square::make_square(rank as u8, file as u8);
                let bb = BitBoard::from_square(sq);
                if self.combined() & bb == EMPTY {
                    s.push_str(" . ");
                } else {
                    let color = if (self.color_combined(Color::White) & bb) == bb { Color::White } else { Color::Black };

                    let mut piece = match self.piece_on(sq).unwrap() {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k'
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
        s.push_str(if self.side_to_move() == Color::White { "Whites Turn\n" } else { "Blacks Turn\n" });
        write!(f, "{}", s)
    }
}
