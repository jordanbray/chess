use crate::board::{Board, BoardStatus};
use crate::board_builder::BoardBuilder;
use crate::chess_move::ChessMove;
use crate::color::Color;
use crate::movegen::MoveGen;
use crate::piece::Piece;
use std::convert::TryInto;
use std::str::FromStr;

/// Contains all actions supported within the game
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq)]
pub enum Action {
    MakeMove(ChessMove),
    OfferDraw(Color),
    AcceptDraw,
    DeclareDraw,
    Resign(Color),
}

/// What was the result of this game?
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum GameResult {
    WhiteCheckmates,
    WhiteResigns,
    BlackCheckmates,
    BlackResigns,
    Stalemate,
    DrawAccepted,
    DrawDeclared,
}

/// For UI/UCI Servers, store a game object which allows you to determine
/// draw by 3 fold repitition, draw offers, resignations, and moves.
///
/// This structure is slow compared to using `Board` directly, so it is
/// not recommended for engines.
#[derive(Clone, Debug)]
pub struct Game {
    start_pos: Board,
    moves: Vec<Action>,
}

impl Game {
    /// Create a new `Game` with the initial position.
    ///
    /// ```
    /// use chess::{Game, Board};
    ///
    /// let game = Game::new();
    /// assert_eq!(game.current_position(), Board::default());
    /// ```
    pub fn new() -> Game {
        Game {
            start_pos: Board::default(),
            moves: vec![],
        }
    }

    /// Get all actions made in this game (moves, draw offers, resignations, etc.)
    ///
    /// ```
    /// use chess::{Game, MoveGen, Color};
    ///
    /// let mut game = Game::new();
    /// let mut movegen = MoveGen::new_legal(&game.current_position());
    ///
    /// game.make_move(movegen.next().expect("At least one valid move"));
    /// game.resign(Color::Black);
    /// assert_eq!(game.actions().len(), 2);
    /// ```
    pub fn actions(&self) -> &Vec<Action> {
        &self.moves
    }

    /// What is the status of this game?
    ///
    /// ```
    /// use chess::Game;
    ///
    /// let game = Game::new();
    /// assert!(game.result().is_none());
    /// ```
    pub fn result(&self) -> Option<GameResult> {
        match self.current_position().status() {
            BoardStatus::Checkmate => {
                if self.side_to_move() == Color::White {
                    Some(GameResult::BlackCheckmates)
                } else {
                    Some(GameResult::WhiteCheckmates)
                }
            }
            BoardStatus::Stalemate => Some(GameResult::Stalemate),
            BoardStatus::Ongoing => {
                if self.moves.len() == 0 {
                    None
                } else if self.moves[self.moves.len() - 1] == Action::AcceptDraw {
                    Some(GameResult::DrawAccepted)
                } else if self.moves[self.moves.len() - 1] == Action::DeclareDraw {
                    Some(GameResult::DrawDeclared)
                } else if self.moves[self.moves.len() - 1] == Action::Resign(Color::White) {
                    Some(GameResult::WhiteResigns)
                } else if self.moves[self.moves.len() - 1] == Action::Resign(Color::Black) {
                    Some(GameResult::BlackResigns)
                } else {
                    None
                }
            }
        }
    }

    /// Create a new `Game` object from an FEN string.  Note: this function will be changed to
    /// return Result<Game, Error> in 4.0.0.
    ///
    /// ```
    /// use chess::{Game, Board};
    ///
    /// let game = Game::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN");
    /// let game2 = Game::new_from_fen("Invalid FEN");
    /// assert!(game2.is_none());
    /// ```
    pub fn new_from_fen(fen: &str) -> Option<Game> {
        if let Ok(fen) = BoardBuilder::from_str(fen) {
            if let Ok(board) = fen.try_into() {
                Some(Game {
                    start_pos: board,
                    moves: vec![],
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the current position on the board from the `Game` object.
    ///
    /// ```
    /// use chess::{Game, Board};
    ///
    /// let game = Game::new();
    /// assert_eq!(game.current_position(), Board::default());
    /// ```
    pub fn current_position(&self) -> Board {
        let mut copy = self.start_pos;

        for x in self.moves.iter() {
            match *x {
                Action::MakeMove(m) => {
                    copy = copy.make_move_new(m);
                }
                _ => {}
            }
        }

        copy
    }

    /// Determine if a player can legally declare a draw by 3-fold repetition or 50-move rule.
    ///
    /// ```
    /// use chess::{Game, Square, ChessMove};
    ///
    /// let b1c3 = ChessMove::new(Square::B1, Square::C3, None);
    /// let c3b1 = ChessMove::new(Square::C3, Square::B1, None);
    ///
    /// let b8c6 = ChessMove::new(Square::B8, Square::C6, None);
    /// let c6b8 = ChessMove::new(Square::C6, Square::B8, None);
    ///
    /// let mut game = Game::new();
    /// assert_eq!(game.can_declare_draw(), false);
    ///
    /// game.make_move(b1c3);
    /// game.make_move(b8c6);
    /// game.make_move(c3b1);
    /// game.make_move(c6b8);
    ///
    /// assert_eq!(game.can_declare_draw(), false); // position has shown up twice
    ///
    /// game.make_move(b1c3);
    /// game.make_move(b8c6);
    /// game.make_move(c3b1);
    /// game.make_move(c6b8);
    /// assert_eq!(game.can_declare_draw(), true); // position has shown up three times
    /// ```
    pub fn can_declare_draw(&self) -> bool {
        if self.result().is_some() {
            return false;
        }

        let mut legal_moves_per_move: Vec<Vec<ChessMove>> = vec![];

        let mut board = self.start_pos;
        let mut reversible_moves = 0;
        legal_moves_per_move.push(MoveGen::new_legal(&board).collect());
        for x in self.moves.iter() {
            match *x {
                Action::MakeMove(m) => {
                    let white_castle_rights = board.castle_rights(Color::White);
                    let black_castle_rights = board.castle_rights(Color::Black);
                    if board.piece_on(m.get_source()) == Some(Piece::Pawn) {
                        reversible_moves = 0;
                    } else if board.piece_on(m.get_dest()).is_some() {
                        reversible_moves = 0;
                    } else {
                        reversible_moves += 1;
                    }
                    board = board.make_move_new(m);

                    if board.castle_rights(Color::White) != white_castle_rights
                        || board.castle_rights(Color::Black) != black_castle_rights
                    {
                        reversible_moves = 0;
                    }
                    legal_moves_per_move.push(MoveGen::new_legal(&board).collect());
                }
                _ => {}
            }
        }

        if reversible_moves >= 100 {
            return true;
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

    /// Declare a draw by 3-fold repitition or 50-move rule.
    ///
    /// ```
    /// use chess::{Game, Square, ChessMove};
    ///
    /// let b1c3 = ChessMove::new(Square::B1, Square::C3, None);
    /// let c3b1 = ChessMove::new(Square::C3, Square::B1, None);
    ///
    /// let b8c6 = ChessMove::new(Square::B8, Square::C6, None);
    /// let c6b8 = ChessMove::new(Square::C6, Square::B8, None);
    ///
    /// let mut game = Game::new();
    /// assert_eq!(game.can_declare_draw(), false);
    ///
    /// game.make_move(b1c3);
    /// game.make_move(b8c6);
    /// game.make_move(c3b1);
    /// game.make_move(c6b8);
    ///
    /// assert_eq!(game.can_declare_draw(), false); // position has shown up twice
    ///
    /// game.make_move(b1c3);
    /// game.make_move(b8c6);
    /// game.make_move(c3b1);
    /// game.make_move(c6b8);
    /// assert_eq!(game.can_declare_draw(), true); // position has shown up three times
    /// game.declare_draw();
    /// ```
    pub fn declare_draw(&mut self) -> bool {
        if self.can_declare_draw() {
            self.moves.push(Action::DeclareDraw);
            true
        } else {
            false
        }
    }

    /// Make a chess move on the board
    ///
    /// ```
    /// use chess::{Game, MoveGen};
    ///
    /// let mut game = Game::new();
    ///
    /// let mut movegen = MoveGen::new_legal(&game.current_position());
    ///
    /// game.make_move(movegen.next().expect("At least one legal move"));
    /// ```
    pub fn make_move(&mut self, chess_move: ChessMove) -> bool {
        if self.result().is_some() {
            return false;
        }
        if self.current_position().legal(chess_move) {
            self.moves.push(Action::MakeMove(chess_move));
            true
        } else {
            false
        }
    }

    /// Who's turn is it to move?
    ///
    /// ```
    /// use chess::{Game, Color};
    ///
    /// let game = Game::new();
    /// assert_eq!(game.side_to_move(), Color::White);
    /// ```
    pub fn side_to_move(&self) -> Color {
        let move_count = self
            .moves
            .iter()
            .filter(|m| match *m {
                Action::MakeMove(_) => true,
                _ => false,
            })
            .count()
            + if self.start_pos.side_to_move() == Color::White {
                0
            } else {
                1
            };

        if move_count % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Offer a draw to my opponent.  `color` is the player who offered the draw.  The draw must be
    /// accepted before my opponent moves.
    ///
    /// ```
    /// use chess::{Game, Color};
    ///
    /// let mut game = Game::new();
    /// game.offer_draw(Color::White);
    /// ```
    pub fn offer_draw(&mut self, color: Color) -> bool {
        if self.result().is_some() {
            return false;
        }
        self.moves.push(Action::OfferDraw(color));
        return true;
    }

    /// Accept a draw offer from my opponent.
    ///
    /// ```
    /// use chess::{Game, MoveGen, Color};
    ///
    /// let mut game = Game::new();
    /// game.offer_draw(Color::Black);
    /// assert_eq!(game.accept_draw(), true);
    ///
    /// let mut game2 = Game::new();
    /// let mut movegen = MoveGen::new_legal(&game2.current_position());
    /// game2.offer_draw(Color::Black);
    /// game2.make_move(movegen.next().expect("At least one legal move"));
    /// assert_eq!(game2.accept_draw(), false);
    /// ```
    pub fn accept_draw(&mut self) -> bool {
        if self.result().is_some() {
            return false;
        }
        if self.moves.len() > 0 {
            if self.moves[self.moves.len() - 1] == Action::OfferDraw(Color::White)
                || self.moves[self.moves.len() - 1] == Action::OfferDraw(Color::Black)
            {
                self.moves.push(Action::AcceptDraw);
                return true;
            }
        }

        if self.moves.len() > 1 {
            if self.moves[self.moves.len() - 2] == Action::OfferDraw(!self.side_to_move()) {
                self.moves.push(Action::AcceptDraw);
                return true;
            }
        }

        false
    }

    /// `color` resigns the game
    ///
    /// ```
    /// use chess::{Game, Color};
    ///
    /// let mut game = Game::new();
    /// game.resign(Color::White);
    /// ```
    pub fn resign(&mut self, color: Color) -> bool {
        if self.result().is_some() {
            return false;
        }
        self.moves.push(Action::Resign(color));
        return true;
    }
}
