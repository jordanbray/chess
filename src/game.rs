use crate::board::Board;
use crate::chess_move::ChessMove;
use crate::color::Color;
use crate::movegen::MoveGen;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq)]
enum GameMove {
    MakeMove(ChessMove),
    OfferDraw(Color),
    AcceptDraw,
    DeclareDraw,
    Resign(Color),
}

pub struct Game {
    start_pos: Board,
    moves: Vec<GameMove>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            start_pos: Board::default(),
            moves: vec!(),
        }
    }

    pub fn new_from_fen(fen: &str) -> Option<Game> {
        let board = Board::from_fen(fen.to_string());
        match board {
            None => None,
            Some(b) => Some(Game {
                start_pos: b,
                moves: vec!(),
            })
        }
    }

    pub fn current_position(&self) -> Board {
        let mut copy = self.start_pos;

        for x in self.moves.iter() {
            match *x {
                GameMove::MakeMove(m) => { copy = copy.make_move_new(m); }
                _ => {}
            }
        }

        copy
    }

    pub fn can_declare_draw(&self) -> bool {
        let mut legal_moves_per_move: Vec<Vec<ChessMove>> = vec!();

        let mut board = self.start_pos;
        legal_moves_per_move.push(MoveGen::new_legal(&board).collect());
        for x in self.moves.iter() {
            match *x {
                GameMove::MakeMove(m) => {
                   board = board.make_move_new(m);
                   legal_moves_per_move.push(MoveGen::new_legal(&board).collect());
                },
                _ => {}
            }
        }

        let last_moves = legal_moves_per_move[legal_moves_per_move.len() - 1].clone();

        for i in 1..(legal_moves_per_move.len() - 1) {
            for j in 0..i {
                if legal_moves_per_move[i] == last_moves && legal_moves_per_move[j] == last_moves {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn declare_draw(&mut self) -> bool {
        if self.can_declare_draw() {
            self.moves.push(GameMove::DeclareDraw);
            true
        } else {
            false
        }
    }

    pub fn make_move(&mut self, chess_move: ChessMove) -> bool {
        if self.current_position().legal(chess_move) {
            self.moves.push(GameMove::MakeMove(chess_move));
            true
        } else {
            false
        }
    }

    pub fn side_to_move(&self) -> Color {
        let move_count = self.moves.iter().filter(
            |m| match *m {
                GameMove::MakeMove(_) => true,
                _ => false
            })
            .count() + if self.start_pos.side_to_move() == Color::White { 0 } else { 1 };

        if move_count % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn offer_draw(&mut self, color: Color) {
        self.moves.push(GameMove::OfferDraw(color));
    }

    pub fn accept_draw(&mut self) -> bool {
        if self.moves.len() > 1 {
            if self.moves[self.moves.len() - 1] == GameMove::OfferDraw(Color::White) ||
               self.moves[self.moves.len() - 1] == GameMove::OfferDraw(Color::Black) {
                self.moves.push(GameMove::AcceptDraw);
                return true;
            }
        }

        if self.moves.len() > 2 {
            if self.moves[self.moves.len() - 2] == GameMove::OfferDraw(!self.side_to_move()) {
                self.moves.push(GameMove::AcceptDraw);
                return true;
            }
        }

        false
    }

    pub fn resign(&mut self, color: Color) {
        self.moves.push(GameMove::Resign(color));
    }


}
