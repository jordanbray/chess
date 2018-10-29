use bitboard::{BitBoard, EMPTY};
use board::Board;
use chess_move::ChessMove;
use movegen::piece_type::*;
use piece::{Piece, NUM_PROMOTION_PIECES, PROMOTION_PIECES};
use square::Square;
use std::iter::ExactSizeIterator;
use std::mem;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool,
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
    ($movegen:expr, $board:expr, $mask:expr, $skip_legal_check:expr) => {{
        let checkers = $board.checkers();
        let combined = $board.combined();
        let color = $board.side_to_move();
        let my_pieces = $board.color_combined(color);
        let ksq = ($board.pieces(Piece::King) & my_pieces).to_square();
        let board = $board;

        if checkers == EMPTY {
            PawnType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            KnightType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            BishopType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            RookType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            QueenType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            KingType::legals::<NotInCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
        } else if checkers.popcnt() == 1 {
            PawnType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            KnightType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            BishopType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            RookType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            QueenType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
            KingType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
        } else {
            KingType::legals::<InCheckType, _>(
                &board,
                $mask,
                combined,
                my_pieces,
                color,
                ksq,
                |x, y, z| $movegen.push(x, y, z),
            );
        }
    }};
}

/// An incremental move generator
///
/// This structure enumerates moves slightly slower than board.enumerate_moves(...),
/// but has some extra features, such as:
///
/// * Being an iterator
/// * Not requiring you to create a buffer
/// * Only iterating moves that match a certain pattern
/// * Being iterable multiple times (such as, iterating once for all captures, then iterating again
///   for all quiets)
/// * Doing as little work early on as possible, so that if you are not going to look at every move, the
///   struture moves faster
/// * Being able to iterate pseudo legal moves, while keeping the (nearly) free legality checks in
///   place
///
/// # Examples
///
/// ```
/// use chess::MoveGen;
/// use chess::Board;
/// use chess::EMPTY;
/// use chess::construct;
///
/// // create a board with the initial position
/// let board = Board::default();
///
/// // create an iterable
/// let mut iterable = MoveGen::new_legal(&board);
///
/// // make sure .len() works.
/// assert_eq!(iterable.len(), 20); // the .len() function does *not* consume the iterator
///
/// // lets iterate over targets.
/// let targets = board.color_combined(!board.side_to_move());
/// iterable.set_iterator_mask(targets);
///
/// // count the number of targets
/// let mut count = 0;
/// for _ in &mut iterable {
///     count += 1;
///     // This move captures one of my opponents pieces (with the exception of en passant)
/// }
///
/// // now, iterate over the rest of the moves
/// iterable.set_iterator_mask(!EMPTY);
/// for _ in &mut iterable {
///     count += 1;
///     // This move does not capture anything
/// }
///
/// // make sure it works
/// assert_eq!(count, 20);
///
/// ```
pub struct MoveGen {
    moves: [SquareAndBitBoard; 18],
    pieces: usize,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}

impl MoveGen {
    /// Create a new `MoveGen` structure, only generating legal moves
    pub fn new_legal(board: &Board) -> MoveGen {
        let mut result = MoveGen {
            moves: unsafe { mem::uninitialized() },
            pieces: 0,
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0,
        };
        let mask = !board.color_combined(board.side_to_move());
        enumerate_moves!(result, board, mask, false);
        result
    }

    /// Create a new `MoveGen` structure, generating all legal moves, and some pseudo-legal moves.
    ///
    /// Note the board.legal_quick() function, which checks the legality of pseudo-legal
    /// moves generated specifically by this structure.  That way, if you call
    /// `MoveGen::new_pseudo_legal(&board)`, but you want to check legality later,
    /// you can call `board.legal_quick(...)` on that chess move, without the full
    /// expense of the `board.legal(...)` function.
     pub fn new_pseudo_legal(board: &Board) -> MoveGen {
        let mut result = MoveGen {
            moves: unsafe { mem::uninitialized() },
            pieces: 0,
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0,
        };
        let mask = !board.color_combined(board.side_to_move());
        enumerate_moves!(result, board, mask, true);
        result
    }

    fn push(&mut self, square: Square, bitboard: BitBoard, promotion: bool) {
        if bitboard != EMPTY {
            self.moves[self.pieces] = SquareAndBitBoard {
                square: square,
                bitboard: bitboard,
                promotion: promotion,
            };
            self.pieces += 1;
        }
    }

    /// Never, ever, iterate any moves that land on the following squares
    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.pieces {
            self.moves[x].bitboard &= !mask;
        }
    }

    /// Never, ever, iterate this move
    pub fn remove_move(&mut self, chess_move: ChessMove) -> bool {
        for x in 0..self.pieces {
            if self.moves[x].square == chess_move.get_source() {
                self.moves[x].bitboard &= !BitBoard::from_square(chess_move.get_dest());
                return true;
            }
        }
        false
    }

    /// For now, Only iterate moves that land on the following squares
    /// Note: Once iteration is completed, you can pass in a mask of ! `EMPTY`
    ///       to get the remaining moves, or another mask
    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;
        self.index = 0;

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
        for j in (i + 1)..self.pieces {
            if self.moves[j].bitboard & self.iterator_mask != EMPTY {
                let backup = self.moves[i];
                self.moves[i] = self.moves[j];
                self.moves[j] = backup;
                i += 1;
            }
        }
    }

    /// Fastest perft test with this structure
    pub fn movegen_perft_test(board: &Board, depth: usize) -> usize {
        let iterable = MoveGen::new_legal(board);

        let mut result: usize = 0;
        if depth == 1 {
            iterable.len()
        } else {
            for m in iterable {
                let mut bresult = unsafe { mem::uninitialized() };
                board.make_move(m, &mut bresult);
                let cur = MoveGen::movegen_perft_test(&bresult, depth - 1);
                result += cur;
            }
            result
        }
    }

    #[cfg(test)]
    /// Do a perft test after splitting the moves up into two groups
    pub fn movegen_perft_test_piecewise(board: &Board, depth: usize) -> usize {
        let mut iterable = MoveGen::new_legal(board);

        let targets = board.color_combined(!board.side_to_move());
        let mut result: usize = 0;

        if depth == 1 {
            iterable.set_iterator_mask(targets);
            result += iterable.len();
            iterable.set_iterator_mask(!targets);
            result += iterable.len();
            result
        } else {
            iterable.set_iterator_mask(targets);
            for x in &mut iterable {
                let mut bresult = unsafe { mem::uninitialized() };
                board.make_move(x, &mut bresult);
                result += MoveGen::movegen_perft_test_piecewise(&bresult, depth - 1);
            }
            iterable.set_iterator_mask(!EMPTY);
            for x in &mut iterable {
                let mut bresult = unsafe { mem::uninitialized() };
                board.make_move(x, &mut bresult);
                result += MoveGen::movegen_perft_test_piecewise(&bresult, depth - 1);
            }
            result
        }
    }

    #[cfg(test)]
    pub fn movegen_perft_test_legality(board: &Board, depth: usize) -> usize {
        let iterable = MoveGen::new_pseudo_legal(board);

        let mut result: usize = 0;
        if depth == 1 {
            iterable.filter(|x| board.legal(*x)).count()
        } else {
            for m in iterable {
                if board.legal_quick(m) {
                    let mut bresult = unsafe { mem::uninitialized() };
                    board.make_move(m, &mut bresult);
                    let cur = MoveGen::movegen_perft_test_legality(&bresult, depth - 1);
                    result += cur;
                }
            }
            result
        }
    }
}

impl ExactSizeIterator for MoveGen {
    /// Give the exact length of this iterator
    fn len(&self) -> usize {
        let mut result = 0;
        for i in 0..self.pieces {
            if self.moves[i].bitboard & self.iterator_mask == EMPTY {
                break;
            }
            if self.moves[i].promotion {
                result += ((self.moves[i].bitboard & self.iterator_mask).popcnt() as usize)
                    * NUM_PROMOTION_PIECES;
            } else {
                result += (self.moves[i].bitboard & self.iterator_mask).popcnt() as usize;
            }
        }
        result
    }
}

impl Iterator for MoveGen {
    type Item = ChessMove;

    /// Give a size_hint to some functions that need it
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    /// Find the next chess move.
    fn next(&mut self) -> Option<ChessMove> {
        if self.index >= self.pieces
            || self.moves[self.index].bitboard & self.iterator_mask == EMPTY
        {
            // are we done?
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

#[cfg(test)]
use construct;

#[cfg(test)]
fn movegen_perft_test(board: String, depth: usize, result: usize) {
    construct::construct();

    let board = Board::from_fen(board).unwrap();

    assert_eq!(MoveGen::movegen_perft_test(&board, depth), result);
    assert_eq!(MoveGen::movegen_perft_test_piecewise(&board, depth), result);
    assert_eq!(MoveGen::movegen_perft_test_legality(&board, depth), result);
}

#[test]
fn movegen_perft_kiwipete() {
    movegen_perft_test(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(),
        5,
        193690690,
    );
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
    movegen_perft_test(
        "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn movegen_perft_10() {
    movegen_perft_test(
        "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn movegen_perft_11() {
    movegen_perft_test(
        "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn movegen_perft_12() {
    movegen_perft_test(
        "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn movegen_perft_13() {
    movegen_perft_test("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn movegen_perft_14() {
    movegen_perft_test("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn movegen_perft_15() {
    movegen_perft_test("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn movegen_perft_16() {
    movegen_perft_test("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn movegen_perft_17() {
    movegen_perft_test("4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn movegen_perft_18() {
    movegen_perft_test("8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn movegen_perft_19() {
    movegen_perft_test("8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn movegen_perft_20() {
    movegen_perft_test("8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn movegen_perft_21() {
    movegen_perft_test("K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn movegen_perft_22() {
    movegen_perft_test("8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn movegen_perft_23() {
    movegen_perft_test("8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn movegen_perft_24() {
    movegen_perft_test("8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn movegen_perft_25() {
    movegen_perft_test("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

#[test]
fn movegen_perft_26() {
    movegen_perft_test("8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}
