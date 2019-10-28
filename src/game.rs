use crate::board::{Board, BoardStatus};
use crate::chess_move::ChessMove;
use crate::color::Color;
use crate::error::Error;
use crate::movegen::MoveGen;
use crate::piece::Piece;
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

    /// Create a new `Game` with a specific starting position.
    ///
    /// ```
    /// use chess::{Game, Board};
    ///
    /// let game = Game::new_with_board(Board::default());
    /// assert_eq!(game.current_position(), Board::default());
    /// ```
    pub fn new_with_board(board: Board) -> Game {
        Game {
            start_pos: board,
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

    /// Create a new `Game` object from an FEN string.
    ///
    /// ```
    /// use chess::{Game, Board};
    ///
    /// // This is the better way:
    /// # {
    /// use std::str::FromStr;
    /// let game: Game = Game::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN");
    /// let game2: Result<Game, _> = Game::from_str("Invalid FEN");
    /// assert!(game2.is_err());
    /// # }
    ///
    /// // This still works
    /// # {
    /// let game = Game::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Valid FEN");
    /// let game2 = Game::new_from_fen("Invalid FEN");
    /// assert!(game2.is_none());
    /// # }
    /// ```
    #[deprecated(since = "3.1.0", note = "Please use Game::from_str(fen)? instead.")]
    pub fn new_from_fen(fen: &str) -> Option<Game> {
        Game::from_str(fen).ok()
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

        let mut legal_moves_per_turn: Vec<(u64, Vec<ChessMove>)> = vec![];

        let mut board = self.start_pos;
        let mut reversible_moves = 0;

        // Loop over each move, counting the reversible_moves for draw by 50 move rule,
        // and filling a list of legal_moves_per_turn list for 3-fold repitition
        legal_moves_per_turn.push((board.get_hash(), MoveGen::new_legal(&board).collect()));
        for x in self.moves.iter() {
            match *x {
                Action::MakeMove(m) => {
                    let white_castle_rights = board.castle_rights(Color::White);
                    let black_castle_rights = board.castle_rights(Color::Black);
                    if board.piece_on(m.get_source()) == Some(Piece::Pawn) {
                        reversible_moves = 0;
                        legal_moves_per_turn.clear();
                    } else if board.piece_on(m.get_dest()).is_some() {
                        reversible_moves = 0;
                        legal_moves_per_turn.clear();
                    } else {
                        reversible_moves += 1;
                    }
                    board = board.make_move_new(m);

                    if board.castle_rights(Color::White) != white_castle_rights
                        || board.castle_rights(Color::Black) != black_castle_rights
                    {
                        reversible_moves = 0;
                        legal_moves_per_turn.clear();
                    }
                    legal_moves_per_turn
                        .push((board.get_hash(), MoveGen::new_legal(&board).collect()));
                }
                _ => {}
            }
        }

        if reversible_moves >= 100 {
            return true;
        }

        // Detect possible draw by 3 fold repitition
        let last_moves = legal_moves_per_turn[legal_moves_per_turn.len() - 1].clone();

        for i in 1..(legal_moves_per_turn.len() - 1) {
            for j in 0..i {
                if legal_moves_per_turn[i] == last_moves && legal_moves_per_turn[j] == last_moves {
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

impl FromStr for Game {
    type Err = Error;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        Ok(Game::new_with_board(Board::from_str(fen)?))
    }
}

#[cfg(test)]
pub fn fake_pgn_parser(moves: &str) -> Game {
    moves
        .split_whitespace()
        .filter(|s| !s.ends_with("."))
        .fold(Game::new(), |mut g, m| {
            g.make_move(ChessMove::from_san(&g.current_position(), m).expect("Valid SAN Move"));
            g
        })
}

#[test]
pub fn test_can_declare_draw() {
    let game = fake_pgn_parser(
        "1. Nc3 d5 2. e3 Nc6 3. Nf3 Nf6 4. Bb5 a6 5. Bxc6+ bxc6 6. Ne5 Qd6 7. d4 Nd7
                8. f4 Nxe5 9. dxe5 Qg6 10. O-O Bf5 11. e4 Bxe4 12. Nxe4 Qxe4 13. Re1 Qb4
                14. e6 f6 15. Be3 g6 16. Qd4 Qxd4 17. Bxd4 Bh6 18. g3 g5 19. f5 g4 20. Rad1
                Rg8 21. b3 Rb8 22. c4 dxc4 23. bxc4 Rd8 24. Kg2 Rc8 25. Bc5 Rg5 26. Rd7 Bf8
                27. Rf1 a5 28. Kg1 a4 29. Bb4 Rh5 30. Rf4 Rg5 31. Rf1 Rh5 32. Rf4 Rg5 33.
                Ba5",
    );
    assert!(!game.can_declare_draw());

    // three fold
    let game = fake_pgn_parser("1. Nc3 Nf6 2. Nb1 Ng8 3. Nc3 Nf6 4. Nb1 Ng8 5. Nc3 Nf6 6. Nb1 Ng8");
    assert!(game.can_declare_draw());

    // three fold (again)
    let game = fake_pgn_parser("1. Nc3 Nf6 2. Nb1 Ng8 3. Nc3 Nf6 4. Nb1 Ng8 5. Nc3 Nf6 6. Nb1");
    assert!(game.can_declare_draw());

    // three fold, but with a move at the end that breaks the draw cycle
    let game =
        fake_pgn_parser("1. Nc3 Nf6 2. Nb1 Ng8 3. Nc3 Nf6 4. Nb1 Ng8 5. Nc3 Nf6 6. Nb1 Ng8 7. e4");
    assert!(!game.can_declare_draw());

    // three fold, but with a move at the end that breaks the draw cycle
    let game =
        fake_pgn_parser("1. Nc3 Nf6 2. Nb1 Ng8 3. Nc3 Nf6 4. Nb1 Ng8 5. Nc3 Nf6 6. Nb1 Ng8 7. e4");
    assert!(!game.can_declare_draw());

    // fifty move rule
    let game = fake_pgn_parser("1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. Nd2 a5 10. Rb1 Nd7 11. a3 f5 12. b4 Kh8 13. f3 Ng8 14. Qc2 Ngf6 15. Nb5 axb4 16. axb4 Nh5 17. g3 Ndf6 18. c5 Bd7 19. Rb3 Nxg3 20. hxg3 Nh5 21. f4 exf4 22. c6 bxc6 23. dxc6 Nxg3 24. Rxg3 fxg3 25. cxd7 g2 26. Rf3 Qxd7 27. Bb2 fxe4 28. Rxf8+ Rxf8 29. Bxg7+ Qxg7 30. Qxe4 Qf6 31. Nf3 Qf4 32. Qe7 Rf7 33. Qe6 Rf6 34. Qe8+ Rf8 35. Qe7 Rf7 36. Qe6 Rf6 37. Qb3 g5 38. Nxc7 g4 39. Nd5 Qc1+ 40. Qd1 Qxd1+ 41. Bxd1 Rf5 42. Ne3 Rf4 43. Ne1 Rxb4 44. Bxg4 h5 45. Bf3 d5 46. N3xg2 h4 47. Nd3 Ra4 48. Ngf4 Kg7 49. Kg2 Kf6 50. Bxd5 Ra5 51. Bc6 Ra6 52. Bb7 Ra3 53. Be4 Ra4 54. Bd5 Ra5 55. Bc6 Ra6 56. Bf3 Kg5 57. Bb7 Ra1 58. Bc8 Ra4 59. Kf3 Rc4 60. Bd7 Kf6 61. Kg4 Rd4 62. Bc6 Rd8 63. Kxh4 Rg8 64. Be4 Rg1 65. Nh5+ Ke6 66. Ng3 Kf6 67. Kg4 Ra1 68. Bd5 Ra5 69. Bf3 Ra1 70. Kf4 Ke6 71. Nc5+ Kd6 72. Nge4+ Ke7 73. Ke5 Rf1 74. Bg4 Rg1 75. Be6 Re1 76. Bc8 Rc1 77. Kd4 Rd1+ 78. Nd3 Kf7 79. Ke3 Ra1 80. Kf4 Ke7 81. Nb4 Rc1 82. Nd5+ Kf7 83. Bd7 Rf1+ 84. Ke5 Ra1 85. Ng5+ Kg6 86. Nf3 Kg7 87. Bg4 Kg6 88. Nf4+ Kg7 89. Nd4 Re1+ 90. Kf5 Rc1 91. Be2 Re1 92. Bh5 Ra1 93. Nfe6+ Kh6 94. Be8 Ra8 95. Bc6 Ra1 96. Kf6 Kh7 97. Ng5+ Kh8 98. Nde6 Ra6 99. Be8 Ra8 100. Bh5 Ra1 101. Bg6 Rf1+ 102. Ke7 Ra1 103. Nf7+ Kg8 104. Nh6+ Kh8 105. Nf5 Ra7+ 106. Kf6 Ra1 107. Ne3 Re1 108. Nd5 Rg1 109. Bf5 Rf1 110. Ndf4 Ra1 111. Ng6+ Kg8 112. Ne7+ Kh8 113. Ng5");
    assert!(game.can_declare_draw());

    let game = fake_pgn_parser("1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2 e5 7. O-O Nc6 8. d5 Ne7 9. Nd2 a5 10. Rb1 Nd7 11. a3 f5 12. b4 Kh8 13. f3 Ng8 14. Qc2 Ngf6 15. Nb5 axb4 16. axb4 Nh5 17. g3 Ndf6 18. c5 Bd7 19. Rb3 Nxg3 20. hxg3 Nh5 21. f4 exf4 22. c6 bxc6 23. dxc6 Nxg3 24. Rxg3 fxg3 25. cxd7 g2 26. Rf3 Qxd7 27. Bb2 fxe4 28. Rxf8+ Rxf8 29. Bxg7+ Qxg7 30. Qxe4 Qf6 31. Nf3 Qf4 32. Qe7 Rf7 33. Qe6 Rf6 34. Qe8+ Rf8 35. Qe7 Rf7 36. Qe6 Rf6 37. Qb3 g5 38. Nxc7 g4 39. Nd5 Qc1+ 40. Qd1 Qxd1+ 41. Bxd1 Rf5 42. Ne3 Rf4 43. Ne1 Rxb4 44. Bxg4 h5 45. Bf3 d5 46. N3xg2 h4 47. Nd3 Ra4 48. Ngf4 Kg7 49. Kg2 Kf6 50. Bxd5 Ra5 51. Bc6 Ra6 52. Bb7 Ra3 53. Be4 Ra4 54. Bd5 Ra5 55. Bc6 Ra6 56. Bf3 Kg5 57. Bb7 Ra1 58. Bc8 Ra4 59. Kf3 Rc4 60. Bd7 Kf6 61. Kg4 Rd4 62. Bc6 Rd8 63. Kxh4 Rg8 64. Be4 Rg1 65. Nh5+ Ke6 66. Ng3 Kf6 67. Kg4 Ra1 68. Bd5 Ra5 69. Bf3 Ra1 70. Kf4 Ke6 71. Nc5+ Kd6 72. Nge4+ Ke7 73. Ke5 Rf1 74. Bg4 Rg1 75. Be6 Re1 76. Bc8 Rc1 77. Kd4 Rd1+ 78. Nd3 Kf7 79. Ke3 Ra1 80. Kf4 Ke7 81. Nb4 Rc1 82. Nd5+ Kf7 83. Bd7 Rf1+ 84. Ke5 Ra1 85. Ng5+ Kg6 86. Nf3 Kg7 87. Bg4 Kg6 88. Nf4+ Kg7 89. Nd4 Re1+ 90. Kf5 Rc1 91. Be2 Re1 92. Bh5 Ra1 93. Nfe6+ Kh6 94. Be8 Ra8 95. Bc6 Ra1 96. Kf6 Kh7 97. Ng5+ Kh8 98. Nde6 Ra6 99. Be8 Ra8 100. Bh5 Ra1 101. Bg6 Rf1+ 102. Ke7 Ra1 103. Nf7+ Kg8 104. Nh6+ Kh8 105. Nf5 Ra7+ 106. Kf6 Ra1 107. Ne3 Re1 108. Nd5 Rg1 109. Bf5 Rf1 110. Ndf4 Ra1 111. Ng6+ Kg8 112. Ne7+ Kh8");
    assert!(!game.can_declare_draw());
}
