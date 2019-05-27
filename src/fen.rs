use crate::board::Board;
use crate::castle_rights::CastleRights;
use crate::color::Color;
use crate::error::Error;
use crate::file::{File, ALL_FILES};
use crate::piece::Piece;
use crate::rank::{Rank, ALL_RANKS};
use crate::square::{Square, ALL_SQUARES};

use std::fmt;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(Copy, Clone)]
pub struct Fen {
    pieces: [Option<(Piece, Color)>; 64],
    side_to_move: Color,
    castle_rights: [CastleRights; 2],
    en_passant: Option<Square>,
}

impl Fen {
    pub fn new() -> Fen {
        Fen {
            pieces: [None; 64],
            side_to_move: Color::White,
            castle_rights: [CastleRights::NoRights, CastleRights::NoRights],
            en_passant: None,
        }
    }

    pub fn setup(
        pieces: impl IntoIterator<Item = (Square, Piece, Color)>,
        side_to_move: Color,
        white_castle_rights: CastleRights,
        black_castle_rights: CastleRights,
        en_passant: Option<File>,
    ) -> Fen {
        let mut result = Fen {
            pieces: [None; 64],
            side_to_move: side_to_move,
            castle_rights: [white_castle_rights, black_castle_rights],
            en_passant: en_passant
                .map(|f| Square::make_square((!side_to_move).to_fourth_rank(), f)),
        };

        for piece in pieces.into_iter() {
            result.pieces[piece.0.to_index()] = Some((piece.1, piece.2));
        }

        result
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn set_side_to_move(&mut self, color: Color) {
        self.side_to_move = color;
    }

    pub fn castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color.to_index()]
    }

    pub fn set_castle_rights(&mut self, color: Color, castle_rights: CastleRights) {
        self.castle_rights[color.to_index()] = castle_rights;
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    pub fn set_en_passant(&mut self, ep: Option<Square>) {
        self.en_passant = ep;
    }
}

impl Index<Square> for Fen {
    type Output = Option<(Piece, Color)>;

    fn index<'a>(&'a self, index: Square) -> &'a Self::Output {
        &self.pieces[index.to_index()]
    }
}

impl IndexMut<Square> for Fen {
    fn index_mut<'a>(&'a mut self, index: Square) -> &'a mut Self::Output {
        &mut self.pieces[index.to_index()]
    }
}

impl fmt::Display for Fen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut count = 0;
        for rank in ALL_RANKS.iter().rev() {
            for file in ALL_FILES.iter() {
                let square = Square::make_square(*rank, *file).to_index();

                if self.pieces[square].is_some() && count != 0 {
                    write!(f, "{}", count)?;
                    count = 0;
                }

                if let Some((piece, color)) = self.pieces[square] {
                    write!(f, "{}", piece.to_string(color))?;
                } else {
                    count += 1;
                }
            }

            if count != 0 {
                write!(f, "{}", count)?;
            }

            if *rank != Rank::First {
                write!(f, "/")?;
            }
            count = 0;
        }

        write!(f, " ")?;

        if self.side_to_move == Color::White {
            write!(f, "w ")?;
        } else {
            write!(f, "b ")?;
        }

        write!(
            f,
            "{}",
            self.castle_rights[Color::White.to_index()].to_string(Color::White)
        )?;
        write!(
            f,
            "{}",
            self.castle_rights[Color::Black.to_index()].to_string(Color::Black)
        )?;
        if self.castle_rights[0] == CastleRights::NoRights
            && self.castle_rights[1] == CastleRights::NoRights
        {
            write!(f, "-")?;
        }

        write!(f, " ")?;
        if let Some(sq) = self.en_passant {
            write!(f, "{}", sq)?;
        } else {
            write!(f, "-")?;
        }

        write!(f, " 0 1")
    }
}

#[test]
fn check_initial_position() {
    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen: Fen = Board::default().into();
    let computed_initial_fen = format!("{}", fen);
    assert_eq!(computed_initial_fen, initial_fen);

    let pass_through = format!("{}", Fen::default());
    assert_eq!(pass_through, initial_fen);
}

impl Default for Fen {
    fn default() -> Fen {
        Fen::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl FromStr for Fen {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut cur_rank = Rank::Eighth;
        let mut cur_file = File::A;
        let mut fen: Fen = Fen::new();

        let tokens: Vec<&str> = value.split(' ').collect();
        if tokens.len() < 4 {
            return Err(Error::InvalidFen {
                fen: value.to_string(),
            });
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let ep = tokens[3];

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
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Rook, Color::Black));
                    cur_file = cur_file.right();
                }
                'R' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Rook, Color::White));
                    cur_file = cur_file.right();
                }
                'n' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Knight, Color::Black));
                    cur_file = cur_file.right();
                }
                'N' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Knight, Color::White));
                    cur_file = cur_file.right();
                }
                'b' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Bishop, Color::Black));
                    cur_file = cur_file.right();
                }
                'B' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Bishop, Color::White));
                    cur_file = cur_file.right();
                }
                'p' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Pawn, Color::Black));
                    cur_file = cur_file.right();
                }
                'P' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Pawn, Color::White));
                    cur_file = cur_file.right();
                }
                'q' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Queen, Color::Black));
                    cur_file = cur_file.right();
                }
                'Q' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::Queen, Color::White));
                    cur_file = cur_file.right();
                }
                'k' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::King, Color::Black));
                    cur_file = cur_file.right();
                }
                'K' => {
                    fen[Square::make_square(cur_rank, cur_file)] =
                        Some((Piece::King, Color::White));
                    cur_file = cur_file.right();
                }
                _ => {
                    return Err(Error::InvalidFen {
                        fen: value.to_string(),
                    });
                }
            }
        }
        match side {
            "w" | "W" => fen.set_side_to_move(Color::White),
            "b" | "B" => fen.set_side_to_move(Color::Black),
            _ => {
                return Err(Error::InvalidFen {
                    fen: value.to_string(),
                })
            }
        }

        if castles.contains("K") && castles.contains("Q") {
            fen.castle_rights[Color::White.to_index()] = CastleRights::Both;
        } else if castles.contains("K") {
            fen.castle_rights[Color::White.to_index()] = CastleRights::KingSide;
        } else if castles.contains("Q") {
            fen.castle_rights[Color::White.to_index()] = CastleRights::QueenSide;
        } else {
            fen.castle_rights[Color::White.to_index()] = CastleRights::NoRights;
        }

        if castles.contains("k") && castles.contains("q") {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::Both;
        } else if castles.contains("k") {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::KingSide;
        } else if castles.contains("q") {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::QueenSide;
        } else {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::NoRights;
        }

        if let Some(sq) = Square::from_string(ep.to_owned()) {
            fen.set_en_passant(Some(sq.ubackward(fen.side_to_move())));
        }

        Ok(fen)
    }
}

impl From<&Board> for Fen {
    fn from(board: &Board) -> Self {
        let mut pieces = vec![];
        for sq in ALL_SQUARES.iter() {
            if let Some(piece) = board.piece_on(*sq) {
                let color = board.color_on(*sq).unwrap();
                pieces.push((*sq, piece, color));
            }
        }

        Fen::setup(
            pieces,
            board.side_to_move(),
            board.castle_rights(Color::White),
            board.castle_rights(Color::Black),
            board.en_passant().map(|sq| sq.get_file()),
        )
    }
}

impl From<Board> for Fen {
    fn from(board: Board) -> Self {
        (&board).into()
    }
}
