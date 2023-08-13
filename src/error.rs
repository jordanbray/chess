use thiserror::Error;

/// Sometimes, bad stuff happens.
#[derive(Clone, Debug, Error)]
pub enum InvalidError {
    /// The FEN string is invalid
    #[error("Invalid FEN string: {fen}")]
    Fen { fen: String },

    /// The board created from BoardBuilder was found to be invalid
    #[error(
        "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"
    )]
    Board,

    /// An attempt was made to create a square from an invalid string
    #[error("The string specified does not contain a valid algebraic notation square")]
    Square,

    /// An attempt was made to create a move from an invalid SAN string
    #[error("The string specified does not contain a valid SAN notation move")]
    SanMove,

    /// An atempt was made to create a move from an invalid UCI string
    #[error("The string specified does not contain a valid UCI notation move")]
    UciMove,

    /// An attempt was made to convert a string not equal to "1"-"8" to a rank
    #[error("The string specified does not contain a valid rank")]
    Rank,

    /// An attempt was made to convert a string not equal to "a"-"h" to a file
    #[error("The string specified does not contain a valid file")]
    File,
}

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("invalid state")]
    Invalid(#[from] InvalidError),
}
