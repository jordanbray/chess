use bitboard::{BitBoard, EMPTY, get_rank, get_adjacent_files};
use piece::Piece;
use magic::{get_rook_moves, get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_moves, between, line};
use chess_move::ChessMove;
use rank::Rank;
use board::Board;


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
            Piece::King => get_king_moves($src)
        } 
    };
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
    ($board:expr, $move_list:expr, $index:expr, $dest_mask:expr, $skip_legal_check:expr, $pawn_mask:expr) => { {
        if $board.checkers() == EMPTY {
            enumerate_moves_one_piece!($board, Piece::Pawn, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Knight, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Bishop, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Rook, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Queen, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::King, false, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
        } else if $board.checkers().popcnt() == 1 {
            enumerate_moves_one_piece!($board, Piece::Pawn, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Knight, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Bishop, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Rook, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::Queen, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
            enumerate_moves_one_piece!($board, Piece::King, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
        } else {
            enumerate_moves_one_piece!($board, Piece::King, true, $board.side_to_move(), $move_list, $index, $dest_mask, $skip_legal_check);
        }
    } };
}

/// Never Call Directly!
///
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
///  * a boolean to determine if any `legal_*` functions should be called to determine if a move is
///    legal
macro_rules! enumerate_moves_one_piece {
    ($board:expr, $piece_type:expr, $in_check:expr, $color:expr, $move_list:expr, $index:expr, $dest_mask:expr, $skip_legal_check:expr) => { {
        let combined = $board.combined();
        let my_pieces = $board.color_combined($color);
        let pieces = $board.pieces($piece_type) & my_pieces;
        let pinned = $board.pinned();
        let checkers = $board.checkers();

        // if the piece is a king, iterate over all pseudo-legal moves, and check to see if it
        // leaves you in check with `legal_king_move`.
        if $piece_type == Piece::King {
            for src in pieces {
                let moves = pseudo_legal_moves!($piece_type, src, $color, combined) & $dest_mask;
                for dest in moves {
                    if $skip_legal_check || $board.legal_king_move(dest) {
                        $move_list[$index] = ChessMove::new(src, dest, None);
                        //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
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
                    ($board.combined() & $board.my_castle_rights().kingside_squares($color)) == EMPTY &&
                     $dest_mask & BitBoard::from_square(ksq.uright().uright()) != EMPTY {
                    if $skip_legal_check || ($board.legal_king_move(ksq.uright()) &&
                       $board.legal_king_move(ksq.uright().uright())) {
                        $move_list[$index] = ChessMove::new(ksq, ksq.uright().uright(), None);
                        //*$move_list.get_unchecked_mut($index) = ChessMove::new(ksq, ksq.uright().uright(), None);
                        $index += 1;
                    }
                }

                if $board.my_castle_rights().has_queenside() && 
                    ($board.combined() & $board.my_castle_rights().queenside_squares($color)) == EMPTY &&
                    $dest_mask & BitBoard::from_square(ksq.uleft().uleft()) != EMPTY {
                    if $skip_legal_check || ($board.legal_king_move(ksq.uleft()) &&
                       $board.legal_king_move(ksq.uleft().uleft())) {
                        $move_list[$index] = ChessMove::new(ksq, ksq.uleft().uleft(), None);
                        //*$move_list.get_unchecked_mut($index) = ChessMove::new(ksq, ksq.uleft().uleft(), None);
                        $index += 1;
                    }
                }
            }
        } else {
            // Just a normal piece move.
            let backrank = get_rank($color.to_their_backrank());
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
                            (if $in_check { between(checkers.to_square(), ksq) ^ checkers } else { !EMPTY });

                // If I'm not a pawn, just add each move to the move list.
                if $piece_type != Piece::Pawn {
                    for dest in moves {
                        $move_list[$index] = ChessMove::new(src, dest, None);
                        //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                        $index += 1;
                    }
                } else {
                    // If I am a pawn, add any 'non-promotion' move to the move list.
                    for dest in moves & !backrank {
                        $move_list[$index] = ChessMove::new(src, dest, None);
                        //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                        $index += 1;
                    }

                    // If I am a pawn, add all 'promotion' moves to the move list.
                    for dest in moves & backrank {
                        $move_list[$index] = ChessMove::new(src, dest, Some(Piece::Queen));
                        $move_list[$index + 1] = ChessMove::new(src, dest, Some(Piece::Knight));
                        $move_list[$index + 2] = ChessMove::new(src, dest, Some(Piece::Rook));
                        $move_list[$index + 3] = ChessMove::new(src, dest, Some(Piece::Bishop));
                        // *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, Some(Piece::Queen));
                        // *$move_list.get_unchecked_mut($index+1) = ChessMove::new(src, dest, Some(Piece::Knight));
                        // *$move_list.get_unchecked_mut($index+2) = ChessMove::new(src, dest, Some(Piece::Rook));
                        // *$move_list.get_unchecked_mut($index+3) = ChessMove::new(src, dest, Some(Piece::Bishop));
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
                                line(src, king_sq);

                    // Same as above.  If I'm not a pawn, just add all the moves to the moves
                    // list
                    if $piece_type != Piece::Pawn {
                        for dest in moves {
                            $move_list[$index] = ChessMove::new(src, dest, None);
                            //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }
                    } else {
                        // If I am a pawn, add all 'non-promotion' moves to the move list.
                        for dest in moves & !backrank {
                            $move_list[$index] = ChessMove::new(src, dest, None);
                            //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }

                        // If I am a pawn, add all 'promotion' moves to the move list.
                        for dest in moves & backrank {
                            $move_list[$index] = ChessMove::new(src, dest, Some(Piece::Queen));
                            $move_list[$index + 1] = ChessMove::new(src, dest, Some(Piece::Knight));
                            $move_list[$index + 2] = ChessMove::new(src, dest, Some(Piece::Rook));
                            $move_list[$index + 3] = ChessMove::new(src, dest, Some(Piece::Bishop));
                            // *$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, Some(Piece::Queen));
                            // *$move_list.get_unchecked_mut($index+1) = ChessMove::new(src, dest, Some(Piece::Knight));
                            // *$move_list.get_unchecked_mut($index+2) = ChessMove::new(src, dest, Some(Piece::Rook));
                            // *$move_list.get_unchecked_mut($index+3) = ChessMove::new(src, dest, Some(Piece::Bishop));
                            $index += 4;
                        }
                    }
                }
            }

            // and, lastly, passed pawn moves
            // What a stupid chess rule...
            // This type of move has its own implementation of legal_*_move, which is called
            // legal_ep_move.
            if $piece_type == Piece::Pawn && $board.en_passant().is_some() {
                let ep_sq = $board.en_passant().unwrap();
                if !$in_check || ($in_check && (checkers & BitBoard::from_square(ep_sq)) != EMPTY) {
                    let rank = get_rank(ep_sq.get_rank());
                    let passed_pawn_pieces = pieces & !pinned & get_adjacent_files(ep_sq.get_file()) & rank;
                    let dest = ep_sq.uforward($color);
                    for src in passed_pawn_pieces {
                        if $dest_mask & BitBoard::from_square(dest) != EMPTY &&
                            ($skip_legal_check || $board.legal_ep_move(src, dest)) {
                            $move_list[$index] = ChessMove::new(src, dest, None);
                            //*$move_list.get_unchecked_mut($index) = ChessMove::new(src, dest, None);
                            $index += 1;
                        }
                    }
                }
            }
        }
    } };
}

pub enum SortResult {
    SearchMe(i32),
    BadMove(i32),
    SkipMe
}

/// The move generation iterator
pub struct MoveGen<'a> {
    board: &'a Board,
    search_first: &'a Vec<ChessMove>,
    search_first_index: usize,

    moves: &'a mut [ChessMove; 256],
    current_move_index: usize,
    num_moves: usize,

    bad_move_bucket: &'a mut [ChessMove; 256],
    bad_move_index: usize,
    num_bad_moves: usize,

    search_buckets: u32,

    capture_sort: fn(&Board, ChessMove) -> SortResult,
    quiet_sort: fn(&Board, ChessMove) -> SortResult
}

const SEARCH_FIRST: u32 = 1;
const GENERATE_CAPTURES: u32 = 2;
const GOOD_CAPTURES: u32 = 3;
const GENERATE_CHECKS: u32 = 4;
const CHECKS: u32 = 5;
const GENERATE_QUIETS: u32 = 6;
const QUIETS: u32 = 7;
const BAD_MOVES: u32 = 8;

const LEGAL_SEARCH_FIRST: u32 = 9;
const LEGAL_GENERATE_CAPTURES: u32 = 10;
const LEGAL_GOOD_CAPTURES: u32 = 11;
const LEGAL_GENERATE_CHECKS: u32 = 12;
const LEGAL_CHECKS: u32 = 13;
const LEGAL_GENERATE_QUIETS: u32 = 14;
const LEGAL_QUIETS: u32 = 15;
const LEGAL_BAD_MOVES: u32 = 16;

const HIGH_BIT: u32 = (1<<31);

pub type MoveGenParams = u32;

pub const MOVEGEN_FIRST: MoveGenParams = 1;
pub const MOVEGEN_GOOD_CAPTURES: MoveGenParams = 2;
pub const MOVEGEN_CHECKS: MoveGenParams = 4;
pub const MOVEGEN_QUIETS: MoveGenParams = 8;
pub const MOVEGEN_BAD_MOVES: MoveGenParams = 16;

impl<'a> MoveGen<'a> {
    fn new_without_search_buckets(board: &'a Board,
                                  capture_sort: fn(&Board, ChessMove) -> SortResult,
                                  quiet_sort: fn(&Board, ChessMove) -> SortResult,
                                  search_first: &'a Vec<ChessMove>,
                                  buffer1: &'a mut [ChessMove; 256],
                                  buffer2: &'a mut [ChessMove; 256]) -> MoveGen<'a> {
         MoveGen {
            board: board,
            search_first: search_first,
            search_first_index: 0,

            moves: buffer1,
            current_move_index: 0,
            num_moves: 0,

            bad_move_bucket: buffer2,
            bad_move_index: 0,
            num_bad_moves: 0,

            search_buckets: 0,

            capture_sort: capture_sort,
            quiet_sort: quiet_sort
        }
    }

    pub fn new_legal(board: &'a Board,
                 params: MoveGenParams,
                 capture_sort: fn(&Board, ChessMove) -> SortResult,
                 quiet_sort: fn(&Board, ChessMove) -> SortResult,
                 search_first: &'a Vec<ChessMove>,
                 buffer1: &'a mut [ChessMove; 256],
                 buffer2: &'a mut [ChessMove; 256]) -> MoveGen<'a> {
        let mut result = MoveGen::new_without_search_buckets(board, capture_sort, quiet_sort, search_first, buffer1, buffer2);

        if params & MOVEGEN_FIRST == MOVEGEN_FIRST {
            result.search_buckets |= HIGH_BIT >> LEGAL_SEARCH_FIRST;
        }
        if params & MOVEGEN_GOOD_CAPTURES == MOVEGEN_GOOD_CAPTURES {
            result.search_buckets |= HIGH_BIT >> LEGAL_GENERATE_CAPTURES;
            result.search_buckets |= HIGH_BIT >> LEGAL_GOOD_CAPTURES;
        }
        if params & MOVEGEN_CHECKS  == MOVEGEN_CHECKS {
            result.search_buckets |= HIGH_BIT >> LEGAL_GENERATE_CHECKS;
            result.search_buckets |= HIGH_BIT >> LEGAL_CHECKS;
        }
        if params & MOVEGEN_QUIETS == MOVEGEN_QUIETS {
            result.search_buckets |= HIGH_BIT >> LEGAL_GENERATE_QUIETS;
            result.search_buckets |= HIGH_BIT >> LEGAL_QUIETS;
        }
        if params & MOVEGEN_BAD_MOVES == MOVEGEN_BAD_MOVES {
            result.search_buckets |= HIGH_BIT >> LEGAL_BAD_MOVES;
        }

        result
    }

    pub fn new_pseudo_legal(board: &'a Board,
                            params: MoveGenParams,
                            capture_sort: fn(&Board, ChessMove) -> SortResult,
                            quiet_sort: fn(&Board, ChessMove) -> SortResult,
                            search_first: &'a Vec<ChessMove>,
                            buffer1: &'a mut [ChessMove; 256],
                            buffer2: &'a mut [ChessMove; 256]) -> MoveGen<'a> {
        let mut result = MoveGen::new_without_search_buckets(board, capture_sort, quiet_sort, search_first, buffer1, buffer2);

        if params & MOVEGEN_FIRST == MOVEGEN_FIRST && search_first.len() > 0 {
            result.search_buckets |= HIGH_BIT >> SEARCH_FIRST;
        }
        if params & MOVEGEN_GOOD_CAPTURES == MOVEGEN_GOOD_CAPTURES {
            result.search_buckets |= HIGH_BIT >> GENERATE_CAPTURES;
            result.search_buckets |= HIGH_BIT >> GOOD_CAPTURES;
        }
        if params & MOVEGEN_CHECKS  == MOVEGEN_CHECKS {
            result.search_buckets |= HIGH_BIT >> GENERATE_CHECKS;
            result.search_buckets |= HIGH_BIT >> CHECKS;
        }
        if params & MOVEGEN_QUIETS == MOVEGEN_QUIETS {
            result.search_buckets |= HIGH_BIT >> GENERATE_QUIETS;
            result.search_buckets |= HIGH_BIT >> QUIETS;
        }
        if params & MOVEGEN_BAD_MOVES == MOVEGEN_BAD_MOVES {
            result.search_buckets |= HIGH_BIT >> BAD_MOVES;
        }

        result
    }

    fn search_first(&mut self) -> Option<ChessMove> {
        if self.search_first_index + 1 == self.search_first.len() {
            self.search_buckets -= HIGH_BIT >> SEARCH_FIRST;
        }
        self.search_first_index += 1;
        Some(self.search_first[self.search_first_index - 1])
    }

    fn legal_search_first(&mut self) -> Option<ChessMove> {
        while self.search_first_index < self.search_first.len() &&
              !self.board.legal(self.search_first[self.search_first_index]) {
            self.search_first_index += 1;
        }
        if self.search_first_index == self.search_first.len() {
            self.search_buckets -= HIGH_BIT >> SEARCH_FIRST;
            self.next()
        } else if self.search_first_index + 1 == self.search_first.len() {
            self.search_first_index += 1;
            self.search_buckets -= HIGH_BIT >> SEARCH_FIRST;
            Some(self.search_first[self.search_first_index - 1])
        } else {
            self.search_first_index += 1;
            Some(self.search_first[self.search_first_index - 1])
        }
    }

    fn generate_captures(&mut self) {
        let their_pieces = self.board.color_combined(!self.board.side_to_move());
        self.num_moves = 0;
        self.current_move_index = 0;
        let pawn_mask = their_pieces | 
            ((get_rank(Rank::First) | get_rank(Rank::Eighth)) &
             !self.board.color_combined(self.board.side_to_move()));
        enumerate_moves!(self.board, self.moves, self.num_moves, their_pieces, true, pawn_mask);
    }

    fn legal_generate_captures(&mut self) {
        let their_pieces = self.board.color_combined(!self.board.side_to_move());
        self.num_moves = 0;
        self.current_move_index = 0;
        let pawn_mask = their_pieces | 
            ((get_rank(Rank::First) | get_rank(Rank::Eighth)) &
             !self.board.color_combined(self.board.side_to_move()));
        self.search_buckets -= HIGH_BIT >> LEGAL_GENERATE_CAPTURES;
        enumerate_moves!(self.board, self.moves, self.num_moves, their_pieces, false, pawn_mask);
    }

    fn generate_checks(&mut self) {
        let not_my_pieces = !self.board.combined();
        self.num_moves = 0;
        self.current_move_index = 0;
        let pawn_mask = not_my_pieces |
                            (get_rank(Rank::Second) |
                             get_rank(Rank::Third) |
                             get_rank(Rank::Fourth) |
                             get_rank(Rank::Fifth) |
                             get_rank(Rank::Sixth) |
                             get_rank(Rank::Seventh));
        panic!();
    }

    fn legal_generate_checks(&mut self) {
        self.search_buckets -= HIGH_BIT >> LEGAL_GENERATE_CHECKS;
        self.current_move_index = 0;
        self.num_moves = 0;
    }


    fn generate_quiets(&mut self) {
        let not_my_pieces = !self.board.combined();
        self.num_moves = 0;
        self.current_move_index = 0;
        let pawn_mask = not_my_pieces |
                            (get_rank(Rank::Second) |
                             get_rank(Rank::Third) |
                             get_rank(Rank::Fourth) |
                             get_rank(Rank::Fifth) |
                             get_rank(Rank::Sixth) |
                             get_rank(Rank::Seventh));
        self.search_buckets -= HIGH_BIT >> GENERATE_QUIETS;
        panic!();
    }

    fn legal_generate_quiets(&mut self) {
        let not_my_pieces = !self.board.combined();
        self.num_moves = 0;
        self.current_move_index = 0;
        let pawn_mask = not_my_pieces |
                            (get_rank(Rank::Second) |
                             get_rank(Rank::Third) |
                             get_rank(Rank::Fourth) |
                             get_rank(Rank::Fifth) |
                             get_rank(Rank::Sixth) |
                             get_rank(Rank::Seventh));
        self.search_buckets -= HIGH_BIT >> LEGAL_GENERATE_QUIETS;
        enumerate_moves!(self.board, self.moves, self.num_moves, not_my_pieces, false, pawn_mask);
    }

    fn no_sort(&mut self, bucket: u32) -> Option<ChessMove> {
        self.current_move_index += 1;
        if self.current_move_index > self.num_moves {
            self.search_buckets -= HIGH_BIT >> bucket;
            self.next()
        } else if self.current_move_index == self.num_moves {
            self.search_buckets -= HIGH_BIT >> bucket;
            Some(self.moves[self.current_move_index - 1])
        } else {
            Some(self.moves[self.current_move_index - 1])
        }
    }

    fn good_capture(&mut self) -> Option<ChessMove> {
        self.no_sort(GOOD_CAPTURES)
    }

    fn quiet(&mut self) -> Option<ChessMove> {
        self.no_sort(QUIETS)
    }

    fn check(&mut self) -> Option<ChessMove> {
        self.no_sort(CHECKS)
    }

    fn bad_move(&mut self) -> Option<ChessMove> {
        None
    }

    fn legal_good_capture(&mut self) -> Option<ChessMove> {
        self.no_sort(LEGAL_GOOD_CAPTURES)
    }

    fn legal_quiet(&mut self) -> Option<ChessMove> {
        self.no_sort(LEGAL_QUIETS)
    }

    fn legal_check(&mut self) -> Option<ChessMove> {
        self.no_sort(LEGAL_CHECKS)
    }

    fn legal_bad_move(&mut self) -> Option<ChessMove> {
        None
    }
}

impl<'a> Iterator for MoveGen<'a> {
    type Item = ChessMove;

    fn next(&mut self) -> Option<ChessMove> {
        match self.search_buckets.leading_zeros() {
            // The user told me to search these moves first
            SEARCH_FIRST => self.search_first(),

            GENERATE_CAPTURES => {
                self.generate_captures();
                self.next()
            },

            GOOD_CAPTURES => {
                self.good_capture()
            },

            GENERATE_CHECKS => {
                self.generate_checks();
                self.next()
            },

            CHECKS =>self.check(),

            GENERATE_QUIETS => {
                self.generate_quiets();
                self.next()
            }

            QUIETS => self.quiet(),

            BAD_MOVES => self.bad_move(),

            LEGAL_SEARCH_FIRST => self.legal_search_first(),

            LEGAL_GENERATE_CAPTURES => {
                self.legal_generate_captures();
                self.next()
            },

            LEGAL_GOOD_CAPTURES => self.legal_good_capture(),

            LEGAL_GENERATE_CHECKS => {
                self.legal_generate_checks();
                self.next()
            },

            LEGAL_CHECKS => self.legal_check(),

            LEGAL_GENERATE_QUIETS => {
                self.legal_generate_quiets();
                self.next()
            },

            LEGAL_QUIETS => self.legal_quiet(),

            LEGAL_BAD_MOVES => self.legal_bad_move(),

            _ => None
        }
    }
}

fn sorter(board: &Board, chess_move: ChessMove) -> SortResult {
    SortResult::SearchMe(1)
}

fn internal_movegen_perft_test(board: Board, depth: usize) -> usize {
    let buffer1 = unsafe { &mut [ChessMove::new(Square::new(0), Square::new(0), None); 256] };
    let buffer2 = unsafe { &mut [ChessMove::new(Square::new(0), Square::new(0), None); 256] };
    let search_first = vec!{};

    if !board.is_sane() {
        println!("Insane Board!");
        println!("{}", board);
        return 0;
    }

    let iterable = MoveGen::new_legal(&board,
                                      MOVEGEN_GOOD_CAPTURES | MOVEGEN_CHECKS | MOVEGEN_QUIETS,
                                      sorter,
                                      sorter,
                                      &search_first,
                                      buffer1,
                                      buffer2);

    let mut result: usize = 0;
    if depth == 1 {
        let mut v: Vec<ChessMove> = iterable.collect();
        let count = v.len();
        if count != (board.perft(1) as usize) {
            v.sort();
            v.dedup();
            if v.len() != count {
                println!("Found Duplicates");
            } else {
                println!("no duplicates");
            }

        }
        count
    } else {
        for m in iterable {
            let cur = internal_movegen_perft_test(board.make_move(m), depth - 1);
            result += cur;
        }
        result
    }
}

use construct;
use square::Square;

fn movegen_perft_test(board: String, depth: usize, result: usize) {
     construct::construct();

     assert_eq!(internal_movegen_perft_test(Board::from_fen(board).unwrap(), depth), result);
}

#[test]
fn movegen_perft_kiwipete() {
    movegen_perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(), 5, 193690690);
}

#[test]
fn movegen_perft_1() {
    movegen_perft_test("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064); // Invalid FEN
}

#[test]
fn movegen_perft_2() {
    movegen_perft_test("8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064); // Invalid FEN
}

#[test]
fn movegen_perft_3() {
    movegen_perft_test("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467);
}

#[test]
fn movegen_perft_4() {
    movegen_perft_test("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467);
}

#[test]
fn movegen_perft_5() {
    movegen_perft_test("5k2/8/8/8/8/8/8/4K2R w K - 0 1".to_owned(), 6, 661072);
}

#[test]
fn movegen_perft_6() {
    movegen_perft_test("4k2r/8/8/8/8/8/8/5K2 b k - 0 1".to_owned(), 6, 661072);
}

#[test]
fn movegen_perft_7() {
    movegen_perft_test("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn movegen_perft_8() {
    movegen_perft_test("r3k3/8/8/8/8/8/8/3K4 b q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn movegen_perft_9() {
    movegen_perft_test("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(), 4, 1274206);
}

