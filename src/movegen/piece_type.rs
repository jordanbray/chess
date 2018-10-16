use piece::Piece;
use bitboard::{BitBoard, EMPTY};
use square::Square;
use color::Color;
use board::Board;

use magic::{
    get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_moves, get_rook_moves, between, line,
    get_adjacent_files, get_rank
};

pub trait PieceType {
    fn is(piece: Piece) -> bool;
    fn into_piece() -> Piece;
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard;
    fn legals<T, F>(board: Board,
                 mask: BitBoard,
                 mut store_moves: F)
            where T: CheckType, F: FnMut(Square, BitBoard, bool) {
        let color = board.side_to_move();
        let combined = board.combined();
        let my_pieces = board.color_combined(color);
        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();
        let ksq = (board.pieces(Piece::King) & my_pieces).to_square();
        
        let check_mask = if T::IN_CHECK {
                   between(checkers.to_square(), ksq) ^ checkers
                } else {
                    !EMPTY
                };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, combined, mask) & check_mask;
            store_moves(src, moves, false);
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, combined, mask) & line(src, ksq);
                store_moves(src, moves, false);
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

impl PieceType for PawnType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Pawn
    }

    fn into_piece() -> Piece {
        Piece::Pawn
    }

    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_pawn_moves(src, color, combined) & mask
    }


    fn legals<T, F>(board: Board,
                 mask: BitBoard,
                 mut store_moves: F)
            where T: CheckType, F: FnMut(Square, BitBoard, bool) {
        let color = board.side_to_move();
        let combined = board.combined();
        let my_pieces = board.color_combined(color);
        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();
        let ksq = (board.pieces(Piece::King) & my_pieces).to_square();
        
        let check_mask = if T::IN_CHECK {
                   between(checkers.to_square(), ksq) ^ checkers
                } else {
                    !EMPTY
                };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, combined, mask) & check_mask;
            store_moves(src, moves, src.get_rank() == color.to_seventh_rank());
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, combined, mask) & line(ksq, src);
                store_moves(src, moves, src.get_rank() == color.to_seventh_rank());
            }
        }
         
        if board.en_passant().is_some() {
            let ep_sq = board.en_passant().unwrap();
            let rank = get_rank(ep_sq.get_rank());
            let files = get_adjacent_files(ep_sq.get_file());
            for src in rank & files & pieces {
                let dest = ep_sq.uforward(color);
                if board.legal_ep_move(src, dest) {
                    store_moves(src, BitBoard::from_square(dest), false);
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

    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }

    fn legals<T, F>(board: Board,
                 mask: BitBoard,
                 mut store_moves: F)
            where T: CheckType, F: FnMut(Square, BitBoard, bool) {
        let color = board.side_to_move();
        let combined = board.combined();
        let my_pieces = board.color_combined(color);
        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();
        let ksq = (board.pieces(Piece::King) & my_pieces).to_square();
        
        let check_mask = if T::IN_CHECK {
                   between(checkers.to_square(), ksq) ^ checkers
                } else {
                    !EMPTY
                };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, combined, mask) & check_mask;
            store_moves(src, moves, false);
        }
    }
}

impl PieceType for RookType {
    fn is(piece: Piece) -> bool {
        piece == Piece::Rook
    }

    fn into_piece() -> Piece {
        Piece::Rook
    }

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

    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, combined) ^ get_bishop_moves(src, combined)) & mask
    }
}

impl PieceType for KingType {
    fn is(piece: Piece) -> bool {
        piece == Piece::King
    }

    fn into_piece() -> Piece {
        Piece::King
    }

    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    fn legals<T, F>(board: Board,
                 mask: BitBoard,
                 mut store_moves: F)
            where T: CheckType, F: FnMut(Square, BitBoard, bool) {
        let color = board.side_to_move();
        let combined = board.combined();
        let my_pieces = board.color_combined(color);
        let pieces = board.pieces(Piece::King) & my_pieces;
        let ksq = pieces.to_square();
        let mut moves = Self::pseudo_legals(ksq, color, combined, mask);

        let copy = moves;
        for dest in copy {
            if !board.legal_king_move(dest) {
                moves ^= BitBoard::from_square(dest);
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
        if !T::IN_CHECK {
            if board.my_castle_rights().has_kingside() &&
                (combined & board.my_castle_rights().kingside_squares(color)) == EMPTY {
                if board.legal_king_move(ksq.uright()) && 
                   board.legal_king_move(ksq.uright().uright()) {
                    moves ^= BitBoard::from_square(ksq.uright().uright());
                }
            }

            if board.my_castle_rights().has_queenside() &&
                (combined & board.my_castle_rights().queenside_squares(color)) == EMPTY {
                if board.legal_king_move(ksq.uleft()) && 
                   board.legal_king_move(ksq.uleft().uleft()) {
                    moves ^= BitBoard::from_square(ksq.uleft().uleft());
                }
            }
        }

        store_moves(ksq, moves, false);
    }
}

