use crate::{CastleType, File};
use crate::bitboard::{BitBoard, EMPTY};
use crate::board::Board;
use crate::color::Color;
use crate::movegen::{MoveList, SquareAndBitBoard};
use crate::piece::Piece;
use crate::square::Square;

use crate::magic::{
    between, get_adjacent_files, get_bishop_moves, get_bishop_rays, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_pawn_moves, get_rank, get_rook_moves, get_rook_rays,
    line,
};

pub trait PieceType {
    fn is(piece: Piece) -> bool;
    fn into_piece() -> Piece;
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard;
    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, mask) & check_mask;
            if moves != EMPTY {
                unsafe {
                    movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, *combined, mask) & line(src, ksq);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        }
    }
}

pub struct PawnType;
pub struct BishopType;
pub struct KnightType;
pub struct RookType;
pub struct QueenType;
pub struct KingType;

pub trait CheckType {
    const IN_CHECK: bool;
}

pub struct InCheckType;
pub struct NotInCheckType;

impl CheckType for InCheckType {
    const IN_CHECK: bool = true;
}

impl CheckType for NotInCheckType {
    const IN_CHECK: bool = false;
}

impl PawnType {
    /// Is a particular en-passant capture legal?
    pub fn legal_ep_move(board: &Board, source: Square, dest: Square) -> bool {
        let combined = board.combined()
            ^ BitBoard::from_square(board.en_passant().unwrap())
            ^ BitBoard::from_square(source)
            ^ BitBoard::from_square(dest);

        let ksq =
            (board.pieces(Piece::King) & board.color_combined(board.side_to_move())).to_square();

        let rooks = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        if (get_rook_rays(ksq) & rooks) != EMPTY {
            if (get_rook_moves(ksq, combined) & rooks) != EMPTY {
                return false;
            }
        }

        let bishops = (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        if (get_bishop_rays(ksq) & bishops) != EMPTY {
            if (get_bishop_moves(ksq, combined) & bishops) != EMPTY {
                return false;
            }
        }

        return true;
    }
}

impl PieceType for PawnType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Pawn
    }

    fn into_piece() -> Piece {
        Piece::Pawn
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_pawn_moves(src, color, combined) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *combined, mask) & check_mask;
            if moves != EMPTY {
                unsafe {
                    movelist.push_unchecked(SquareAndBitBoard::new(
                        src,
                        moves,
                        src.get_rank() == color.to_seventh_rank(),
                    ));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, *combined, mask) & line(ksq, src);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            moves,
                            src.get_rank() == color.to_seventh_rank(),
                        ));
                    }
                }
            }
        }

        if board.en_passant().is_some() {
            let ep_sq = board.en_passant().unwrap();
            let rank = get_rank(ep_sq.get_rank());
            let files = get_adjacent_files(ep_sq.get_file());
            for src in rank & files & pieces {
                let dest = ep_sq.uforward(color);
                if PawnType::legal_ep_move(board, src, dest) {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(
                            src,
                            BitBoard::from_square(dest),
                            false,
                        ));
                    }
                }
            }
        }
    }
}

impl PieceType for BishopType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Bishop
    }

    fn into_piece() -> Piece {
        Piece::Bishop
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_bishop_moves(src, combined) & mask
    }
}

impl PieceType for KnightType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Knight
    }

    fn into_piece() -> Piece {
        Piece::Knight
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let my_pieces = board.color_combined(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        if T::IN_CHECK {
            let check_mask = between(checkers.to_square(), ksq) ^ checkers;

            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *combined, mask & check_mask);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        } else {
            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *combined, mask);
                if moves != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        };
    }
}

impl PieceType for RookType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Rook
    }

    fn into_piece() -> Piece {
        Piece::Rook
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_rook_moves(src, combined) & mask
    }
}

impl PieceType for QueenType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Queen
    }

    fn into_piece() -> Piece {
        Piece::Queen
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, combined) ^ get_bishop_moves(src, combined)) & mask
    }
}

impl KingType {
    /// Is a particular king move legal?
    #[inline(always)]
    pub fn legal_king_move(board: &Board, dest: Square) -> bool {
        let combined = board.combined()
            ^ (board.pieces(Piece::King) & board.color_combined(board.side_to_move()))
            | BitBoard::from_square(dest);

        let mut attackers = EMPTY;

        let rooks = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        attackers |= get_rook_moves(dest, combined) & rooks;

        let bishops = (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
            & board.color_combined(!board.side_to_move());

        attackers |= get_bishop_moves(dest, combined) & bishops;

        let knight_rays = get_knight_moves(dest);
        attackers |=
            knight_rays & board.pieces(Piece::Knight) & board.color_combined(!board.side_to_move());

        let king_rays = get_king_moves(dest);
        attackers |=
            king_rays & board.pieces(Piece::King) & board.color_combined(!board.side_to_move());

        attackers |= get_pawn_attacks(
            dest,
            board.side_to_move(),
            board.pieces(Piece::Pawn) & board.color_combined(!board.side_to_move()),
        );

        return attackers == EMPTY;
    }
}

impl PieceType for KingType {
    fn is(piece: Piece) -> bool {
        piece == Piece::King
    }

    fn into_piece() -> Piece {
        Piece::King
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut MoveList, board: &Board, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let color = board.side_to_move();
        let ksq = board.king_square(color);

        let mut moves = Self::pseudo_legals(ksq, color, *combined, mask);

        let copy = moves;
        for dest in copy {
            if !KingType::legal_king_move(board, dest) {
                moves ^= BitBoard::from_square(dest);
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `Board` structure says we can.
        //  * the squares between my king and king dest are empty (except for the castling rook).
        //  * the square between the castling rook and the rook dest are empty (except for the king).
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !T::IN_CHECK {
            for castle_type in [CastleType::Kingside, CastleType::Queenside] {
                if let Some(rook_file) = board.my_castle_rights().get(castle_type) {
                    let backrank = color.to_my_backrank();
                    let rook_sq = Square::make_square(backrank, rook_file);
                    let rook_sq_bb = BitBoard::from_square(rook_sq);
                    let king_sq_bb = BitBoard::from_square(ksq);
                    let king_journey_squares = castle_type.king_journey_squares(ksq);
                    if combined & !rook_sq_bb & !king_sq_bb & king_journey_squares == EMPTY {
                        let rook_dest_sq = castle_type.rook_dest(color);
                        let rook_path = between(rook_sq, rook_dest_sq) | BitBoard::from_square(rook_dest_sq);

                        let rook_path_clear = combined & !rook_sq_bb & !king_sq_bb & rook_path == EMPTY;
                        if rook_path_clear {

                            let mut journey_squares_not_attacked = true;
                            for sq in king_journey_squares {
                                if !KingType::legal_king_move(board, sq) {
                                    journey_squares_not_attacked = false;
                                    break;
                                }
                            }
                            // check that there are no enemy heavy pieces waiting for the king on the other side!
                            if castle_type == CastleType::Kingside {
                                if rook_file != File::H {
                                    let their_heavy_pieces = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen)) & board.color_combined(!color);
                                    if their_heavy_pieces & BitBoard::set(backrank, File::H) != EMPTY {
                                        continue;
                                    }
                                }
                            } else {
                                if rook_file != File::A {
                                    let their_heavy_pieces = (board.pieces(Piece::Rook) | board.pieces(Piece::Queen)) & board.color_combined(!color);
                                    if their_heavy_pieces & BitBoard::set(backrank, File::B) != EMPTY {
                                        continue;
                                    }
                                    if their_heavy_pieces & BitBoard::set(backrank, File::A) != EMPTY &&
                                       combined & !rook_sq_bb & BitBoard::set(backrank, File::B) == EMPTY {
                                        continue;
                                    }
                                }
                            }
                            if journey_squares_not_attacked {
                                moves ^= rook_sq_bb;
                            }
                        }
                    }
                }
            }
        }
        if moves != EMPTY {
            unsafe {
                movelist.push_unchecked(SquareAndBitBoard::new(ksq, moves, false));
            }
        }
    }
}
