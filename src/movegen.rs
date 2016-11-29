use bitboard::{BitBoard, EMPTY, get_rank, get_adjacent_files};
use piece::{Piece, NUM_PROMOTION_PIECES, PROMOTION_PIECES};
use magic::{get_rook_moves, get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_moves, between, line};
use chess_move::ChessMove;
use rank::Rank;
use board::Board;
use file::File;
use std::mem;

/// Never Call Directly!
///
/// Generate the pseudo-legal moves (moves that *may* leave you in check) for a particular piece
/// Do this as a macro for optimization purposes.
///
/// You must pass in the piece type, the source square, the color of the piece to move, and the
/// combined `BitBoard` which represents blocking pieces.
macro_rules! pseudo_legal_moves {
    ($piece_type:expr, $src:expr, $color:expr, $combined:expr, $mask:expr) => {
        match $piece_type {
            Piece::Pawn => SquareAndBitBoard { square: $src,
                                               bitboard: get_pawn_moves($src, $color, $combined) & $mask,
                                               promotion: BitBoard::from_square($src) & get_rank($color.to_seventh_rank()) != EMPTY },
            Piece::Knight => SquareAndBitBoard { square: $src, bitboard: get_knight_moves($src) & $mask, promotion: false },
            Piece::Bishop => SquareAndBitBoard { square: $src, bitboard: get_bishop_moves($src, $combined) & $mask, promotion: false },
            Piece::Rook => SquareAndBitBoard { square: $src, bitboard: get_rook_moves($src, $combined) & $mask, promotion: false },
            Piece::Queen => SquareAndBitBoard { square: $src,
                                                bitboard: (get_bishop_moves($src, $combined) ^ get_rook_moves($src, $combined)) & $mask,
                                                promotion: false },
            Piece::King => SquareAndBitBoard { square: $src, bitboard: get_king_moves($src) & $mask, promotion: false }
        }
    };
}

/// Never Call Directly!
///
/// Enumerate all legal moves for a particular board.
///
/// You must pass in:
///  * a `MoveGen`
///  * a whether or not you want to skip the legality checks.
///  Note: The pawn moves *must* be generated first due to assumptions made by the `MoveGen`
///        struct.
macro_rules! enumerate_moves {
    ($movegen:expr, $mask: expr, $skip_legal_check:expr) => { {
        let color = $movegen.board.side_to_move();
        if $movegen.board.checkers() == EMPTY {
            enumerate_moves_one_piece!($movegen, Piece::Pawn, false, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Knight, false, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Bishop, false, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Rook, false, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Queen, false, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::King, false, color, $mask, $skip_legal_check);
        } else if $movegen.board.checkers().popcnt() == 1 {
            enumerate_moves_one_piece!($movegen, Piece::Pawn, true, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Knight, true, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Bishop, true, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Rook, true, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::Queen, true, color, $mask, $skip_legal_check);
            enumerate_moves_one_piece!($movegen, Piece::King, true, color, $mask, $skip_legal_check);
        } else {
            enumerate_moves_one_piece!($movegen, Piece::King, true, color, $mask, $skip_legal_check);
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
///  * a `MoveGen`
///  * a `Piece` type
///  * whether or not you are currently in check (a boolean)
///  * your color
///  * a boolean to determine if any `legal_*` functions should be called to determine if a move is
///    legal
macro_rules! enumerate_moves_one_piece {
    ($movegen:expr, $piece_type:expr, $in_check:expr, $color:expr, $mask:expr, $skip_legal_check:expr) => { {
        let combined = $movegen.board.combined();
        let my_pieces = $movegen.board.color_combined($color);
        let pieces = $movegen.board.pieces($piece_type) & my_pieces;
        let pinned = $movegen.board.pinned();
        let checkers = $movegen.board.checkers();

        // if the piece is a king, iterate over all pseudo-legal moves, and check to see if it
        // leaves you in check with `legal_king_move`.
        if $piece_type == Piece::King {
            let ksq = ($movegen.board.pieces(Piece::King) & $movegen.board.color_combined($color)).to_square();
            $movegen.moves[$movegen.pieces] = pseudo_legal_moves!($piece_type, ksq, $color, combined, $mask);
            
            // maybe check the legality of these moves
            if !$skip_legal_check {
                let iter = $movegen.moves[$movegen.pieces].bitboard;
                for dest in iter {
                    if !$movegen.board.legal_king_move(dest) {
                        $movegen.moves[$movegen.pieces].bitboard ^= BitBoard::from_square(dest);
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
                if $movegen.board.my_castle_rights().has_kingside() && 
                    ($movegen.board.combined() & $movegen.board.my_castle_rights().kingside_squares($color)) == EMPTY {
                    if $skip_legal_check ||
                        ($movegen.board.legal_king_move(ksq.uright()) && $movegen.board.legal_king_move(ksq.uright().uright())) {
                        $movegen.moves[$movegen.pieces].bitboard ^= BitBoard::from_square(ksq.uright().uright());
                    }
                }

                if $movegen.board.my_castle_rights().has_queenside() &&
                    ($movegen.board.combined() & $movegen.board.my_castle_rights().queenside_squares($color)) == EMPTY {
                    if $skip_legal_check ||
                        ($movegen.board.legal_king_move(ksq.uleft()) && $movegen.board.legal_king_move(ksq.uleft().uleft())) {
                        $movegen.moves[$movegen.pieces].bitboard ^= BitBoard::from_square(ksq.uleft().uleft());
                    }
                }
            }

            // if we found any legal king moves, increment the number of pieces with moves
            if $movegen.moves[$movegen.pieces].bitboard != EMPTY {
                $movegen.pieces += 1;
            }
        } else {
            // Just a normal piece move.
            let ksq = ($movegen.board.pieces(Piece::King) & my_pieces).to_square();

            // if the piece is not pinned:
            //  * And I'm currently in check:
            //  ** I can move to any square between my king and the dude putting me in check.
            //  ** I can catpure the dude putting me in check
            //  ** I will not be at this section of code if I'm in double-check
            //  * And I'm currently NOT in check:
            //  ** I can move anywhere!
            for src in pieces & !pinned { 
                $movegen.moves[$movegen.pieces] = pseudo_legal_moves!($piece_type, src, $color, combined, $mask);
                $movegen.moves[$movegen.pieces].bitboard &= if $in_check {
                                                                between(checkers.to_square(), ksq) ^ checkers
                                                            } else {
                                                                !EMPTY
                                                            };
                if $piece_type == Piece::Pawn && $movegen.board.en_passant().is_some() { // passed pawn rule
                    let ep_sq = $movegen.board.en_passant().unwrap();
                    let rank = get_rank(ep_sq.get_rank());
                    let files = get_adjacent_files(ep_sq.get_file());
                    if rank & files & BitBoard::from_square(src) != EMPTY {
                        let dest = ep_sq.uforward($color);
                        if $skip_legal_check || $movegen.board.legal_ep_move(src, dest) {
                            $movegen.moves[$movegen.pieces].bitboard ^= BitBoard::from_square(dest);
                        }
                    }
                }
                if $movegen.moves[$movegen.pieces].bitboard != EMPTY {
                    $movegen.pieces += 1;
                }
            }

            // If I'm not in check AND I'm pinned
            //  * I can still move along the line between my pinner and my king
            //  * I can still capture my pinner
            //  * If I'm a knight, I cannot move at all due to the way knights move.
            if !$in_check && $piece_type != Piece::Knight {
                // for each pinned piece of this type
                for src in pieces & pinned {
                    // grab all the moves that put me between my pinner and my king, and
                    // possibly capture my attacker
                    // * Note: Due to how lines work, the line between my pinner and my king is
                    //         the same as the line between ME and my king.  So lets use the
                    //         second definition because it's easier to code.
                    $movegen.moves[$movegen.pieces] = pseudo_legal_moves!($piece_type, src, $color, combined, $mask);
                    $movegen.moves[$movegen.pieces].bitboard &= line(src, ksq);
                     if $piece_type == Piece::Pawn && $movegen.board.en_passant().is_some() { // passed pawn rule
                        let ep_sq = $movegen.board.en_passant().unwrap();
                        let rank = get_rank(ep_sq.get_rank());
                        let files = get_adjacent_files(ep_sq.get_file());
                        if rank & files & BitBoard::from_square(src) != EMPTY {
                            let dest = ep_sq.uforward($color);
                            if $skip_legal_check || $movegen.board.legal_ep_move(src, dest) {
                                $movegen.moves[$movegen.pieces].bitboard ^= BitBoard::from_square(dest);
                            }
                        }
                    }
                    if $movegen.moves[$movegen.pieces].bitboard != EMPTY {
                        $movegen.pieces += 1;
                    }
                }
            }

            // The astute among you will notice a missing invariant.
            // If I'm in check AND I'm pinned, I cannot move at all.
            // So, lets just do nothing in that case
        }
    } };
}

#[derive(Copy, Clone)]
struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool
}

/// The move generation iterator
pub struct MoveGen {
    board: Board,
    moves: [SquareAndBitBoard; 16],
    pieces: usize,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}

impl MoveGen {
    pub fn new(board: Board, legal: bool) -> MoveGen {
         let mut result = MoveGen {
            board: board,
            moves: unsafe { mem::uninitialized() },
            pieces: 0,
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0
         };
         let mask = !result.board.color_combined(result.board.side_to_move());
         if legal {
             enumerate_moves!(result, mask, false);
         } else {
             enumerate_moves!(result, mask, true);
         }
         result
    }

    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.pieces {
            self.moves[x].bitboard &= !mask;
        }
    }

    pub fn remove_move(&mut self, chess_move: ChessMove) -> bool {
        for x in 0..self.pieces {
            if self.moves[x].square == chess_move.get_source() {
                self.moves[x].bitboard &= !BitBoard::from_square(chess_move.get_dest());
                return true;
            }
        }
        false
    }

    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;

        // the iterator portion of this struct relies on the invariant that
        // the bitboards at the beginning of the moves[] array are the only
        // ones used.  As a result, we must partition the list such that the
        // assumption is true. 

        // first, find the first non-used moves index, and store that in i
        let mut i = 0;
        while i < self.pieces && self.moves[i].bitboard & self.iterator_mask != EMPTY {
            i += 1;
        }

        // next, find each element past i where the moves are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in i..self.pieces {
            if self.moves[j].bitboard & self.iterator_mask == EMPTY {
                continue;
            }
            let backup = self.moves[i];
            self.moves[i] = self.moves[j];
            self.moves[j] = backup;
            i += 1;
        }
    }
}

impl Iterator for MoveGen {
    type Item = ChessMove;

    fn next(&mut self) -> Option<ChessMove> {
        if self.index >= self.pieces || self.moves[self.index].bitboard & self.iterator_mask == EMPTY { // are we done?
            None
        } else if self.moves[self.index].promotion {
            let bb = &mut self.moves[self.index].bitboard;
            let src = self.moves[self.index].square;
            let dest = (*bb & self.iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = ChessMove::new(src, dest, Some(PROMOTION_PIECES[self.promotion_index]));
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                *bb ^= BitBoard::from_square(dest);
                self.promotion_index = 0;
                if *bb == EMPTY {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let bb = &mut self.moves[self.index].bitboard;
            let src = self.moves[self.index].square;
            let dest = (*bb & self.iterator_mask).to_square();

            *bb ^= BitBoard::from_square(dest);
            if *bb == EMPTY {
                self.index += 1;
            }
            Some(ChessMove::new(src, dest, None))
        }
    }
}

fn internal_movegen_perft_test(board: Board, depth: usize) -> usize {
    let iterable = MoveGen::new(board, true);

    let mut result: usize = 0;
    if depth == 1 {
        iterable.count()
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

#[test]
fn movegen_perft_10() {
    Board::perft_test("r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(), 4, 1274206);
}

#[test]
fn movegen_perft_11() {
    Board::perft_test("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(), 4, 1720476);
}

#[test]
fn movegen_perft_12() {
    Board::perft_test("r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(), 4, 1720476);
}

#[test]
fn movegen_perft_13() {
    Board::perft_test("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn movegen_perft_14() {
    Board::perft_test("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn movegen_perft_15() {
    Board::perft_test("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn movegen_perft_16() {
    Board::perft_test("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn movegen_perft_17() {
    Board::perft_test("4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn movegen_perft_18() {
    Board::perft_test("8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn movegen_perft_19() {
    Board::perft_test("8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn movegen_perft_20() {
    Board::perft_test("8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn movegen_perft_21() {
    Board::perft_test("K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn movegen_perft_22() {
    Board::perft_test("8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn movegen_perft_23() {
    Board::perft_test("8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn movegen_perft_24() {
    Board::perft_test("8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn movegen_perft_25() {
    Board::perft_test("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

#[test]
fn movegen_perft_26() {
    Board::perft_test("8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}


