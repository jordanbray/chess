use crate::bitboard::{BitBoard, EMPTY};
use crate::board::Board;
use crate::chess_move::ChessMove;
use crate::magic::between;
use crate::movegen::piece_type::*;
use crate::piece::{Piece, NUM_PROMOTION_PIECES, PROMOTION_PIECES};
use crate::square::Square;
use arrayvec::ArrayVec;
use nodrop::NoDrop;
use std::iter::ExactSizeIterator;
use std::mem;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool,
}

impl SquareAndBitBoard {
    pub fn new(sq: Square, bb: BitBoard, promotion: bool) -> SquareAndBitBoard {
        SquareAndBitBoard {
            square: sq,
            bitboard: bb,
            promotion: promotion,
        }
    }
}

pub type MoveList = NoDrop<ArrayVec<[SquareAndBitBoard; 18]>>;

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
/// iterable.set_iterator_mask(*targets);
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
    moves: MoveList,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}

impl MoveGen {
    #[inline(always)]
    fn enumerate_moves(board: &Board) -> MoveList {
        let checkers = *board.checkers();
        let mask = !board.color_combined(board.side_to_move());
        let mut movelist = NoDrop::new(ArrayVec::<[SquareAndBitBoard; 18]>::new());

        if checkers == EMPTY {
            PawnType::legals::<NotInCheckType>(&mut movelist, &board, mask);
            KnightType::legals::<NotInCheckType>(&mut movelist, &board, mask);
            BishopType::legals::<NotInCheckType>(&mut movelist, &board, mask);
            RookType::legals::<NotInCheckType>(&mut movelist, &board, mask);
            QueenType::legals::<NotInCheckType>(&mut movelist, &board, mask);
            KingType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        } else if checkers.popcnt() == 1 {
            PawnType::legals::<InCheckType>(&mut movelist, &board, mask);
            KnightType::legals::<InCheckType>(&mut movelist, &board, mask);
            BishopType::legals::<InCheckType>(&mut movelist, &board, mask);
            RookType::legals::<InCheckType>(&mut movelist, &board, mask);
            QueenType::legals::<InCheckType>(&mut movelist, &board, mask);
            KingType::legals::<InCheckType>(&mut movelist, &board, mask);
        } else {
            KingType::legals::<InCheckType>(&mut movelist, &board, mask);
        }

        movelist
    }

    /// Create a new `MoveGen` structure, only generating legal moves
    #[inline(always)]
    pub fn new_legal(board: &Board) -> MoveGen {
        MoveGen {
            moves: MoveGen::enumerate_moves(board),
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0,
        }
    }

    /// Never, ever, iterate any moves that land on the following squares
    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.moves.len() {
            self.moves[x].bitboard &= !mask;
        }
    }

    /// Never, ever, iterate this move
    /// If this move was to be iterated, returns true; otherwise returns false.
    pub fn remove_move(&mut self, chess_move: ChessMove) -> bool {
        for x in 0..self.moves.len() {
            if self.moves[x].square == chess_move.get_source() {
                let dest_bb = BitBoard::from_square(chess_move.get_dest());
                if self.moves[x].bitboard & dest_bb != EMPTY {
                    self.moves[x].bitboard &= !dest_bb;
                    return true;
                } else {
                    //return false;
                }
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
        while i < self.moves.len() && self.moves[i].bitboard & self.iterator_mask != EMPTY {
            i += 1;
        }

        // next, find each element past i where the moves are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in (i + 1)..self.moves.len() {
            if self.moves[j].bitboard & self.iterator_mask != EMPTY {
                // self.moves.swap(i, j);
                let backup = self.moves[i];
                self.moves[i] = self.moves[j];
                self.moves[j] = backup;
                i += 1;
            }
        }
    }

    /// This function checks the legality *only for moves generated by `MoveGen`*.
    ///
    /// Calling this function for moves not generated by `MoveGen` will result in possibly
    /// incorrect results, and making that move on the `Board` will result in undefined behavior.
    /// This function may panic! if these rules are not followed.
    ///
    /// If you are validating a move from a user, you should call the .legal() function.
    pub fn legal_quick(board: &Board, chess_move: ChessMove) -> bool {
        let piece = board.piece_on(chess_move.get_source()).unwrap();
        match piece {
            Piece::Rook => true,
            Piece::Bishop => true,
            Piece::Knight => true,
            Piece::Queen => true,
            Piece::Pawn => {
                if chess_move.get_source().get_file() != chess_move.get_dest().get_file()
                    && board.piece_on(chess_move.get_dest()).is_none()
                {
                    // en-passant
                    PawnType::legal_ep_move(board, chess_move.get_source(), chess_move.get_dest())
                } else {
                    true
                }
            }
            Piece::King => {
                let bb = between(chess_move.get_source(), chess_move.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !KingType::legal_king_move(board, bb.to_square()) {
                        false
                    } else {
                        KingType::legal_king_move(board, chess_move.get_dest())
                    }
                } else {
                    KingType::legal_king_move(board, chess_move.get_dest())
                }
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
            iterable.set_iterator_mask(*targets);
            result += iterable.len();
            iterable.set_iterator_mask(!targets);
            result += iterable.len();
            result
        } else {
            iterable.set_iterator_mask(*targets);
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
}

impl ExactSizeIterator for MoveGen {
    /// Give the exact length of this iterator
    fn len(&self) -> usize {
        let mut result = 0;
        for i in 0..self.moves.len() {
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
        if self.index >= self.moves.len()
            || self.moves[self.index].bitboard & self.iterator_mask == EMPTY
        {
            // are we done?
            None
        } else if self.moves[self.index].promotion {
            let moves = &mut self.moves[self.index];

            let dest = (moves.bitboard & self.iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = ChessMove::new(
                moves.square,
                dest,
                Some(PROMOTION_PIECES[self.promotion_index]),
            );
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                moves.bitboard ^= BitBoard::from_square(dest);
                self.promotion_index = 0;
                if moves.bitboard & self.iterator_mask == EMPTY {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let moves = &mut self.moves[self.index];
            let dest = (moves.bitboard & self.iterator_mask).to_square();

            moves.bitboard ^= BitBoard::from_square(dest);
            if moves.bitboard & self.iterator_mask == EMPTY {
                self.index += 1;
            }
            Some(ChessMove::new(moves.square, dest, None))
        }
    }
}

#[cfg(test)]
use crate::board_builder::BoardBuilder;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::convert::TryInto;
#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
fn movegen_perft_test(fen: String, depth: usize, result: usize) {
    let board: Board = BoardBuilder::from_str(&fen).unwrap().try_into().unwrap();

    assert_eq!(MoveGen::movegen_perft_test(&board, depth), result);
    assert_eq!(MoveGen::movegen_perft_test_piecewise(&board, depth), result);
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

#[test]
fn movegen_issue_15() {
    let board =
        BoardBuilder::from_str("rnbqkbnr/ppp2pp1/4p3/3N4/3PpPp1/8/PPP3PP/R1B1KBNR b KQkq f3 0 1")
            .unwrap()
            .try_into()
            .unwrap();
    let _ = MoveGen::new_legal(&board);
}

#[cfg(test)]
fn move_of(m: &str) -> ChessMove {
    let promo = if m.len() > 4 {
        Some(match m.as_bytes()[4] {
            b'q' => Piece::Queen,
            b'r' => Piece::Rook,
            b'b' => Piece::Bishop,
            b'n' => Piece::Knight,
            _ => panic!("unrecognized uci move: {}", m),
        })
    } else {
        None
    };
    ChessMove::new(
        Square::from_string(m[..2].to_string()).unwrap(),
        Square::from_string(m[2..4].to_string()).unwrap(),
        promo,
    )
}

#[test]
fn test_masked_move_gen() {
    let board =
        Board::from_str("r1bqkb1r/pp3ppp/5n2/2ppn1N1/4pP2/1BN1P3/PPPP2PP/R1BQ1RK1 w kq - 0 9")
            .unwrap();

    let mut capture_moves = MoveGen::new_legal(&board);
    let targets = *board.color_combined(!board.side_to_move());
    capture_moves.set_iterator_mask(targets);

    let expected = vec![
        move_of("f4e5"),
        move_of("b3d5"),
        move_of("g5e4"),
        move_of("g5f7"),
        move_of("g5h7"),
        move_of("c3e4"),
        move_of("c3d5"),
    ];

    assert_eq!(
        capture_moves.collect::<HashSet<_>>(),
        expected.into_iter().collect()
    );
}
