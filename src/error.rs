use failure::Fail;

/// Sometimes, bad stuff happens.
#[derive(Clone, Debug, Fail)]
pub enum Error {
    /// The FEN string is invalid
    #[fail(display = "Invalid FEN string: {}", fen)]
    InvalidFen { fen: String },

    /// The board created from BoardBuilder was found to be invalid
    #[fail(
        display = "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"
    )]
    InvalidBoard,

    /// An attempt was made to create a square from an invalid string
    #[fail(display = "The string specified does not contain a valid algebraic notation square")]
    InvalidSquare,

    /// An attempt was made to create a move from an invalid SAN string
    #[fail(display = "The string specified does not contain a valid SAN notation move")]
    InvalidSanMove,

    /// An atempt was made to create a move from an invalid UCI string
    #[fail(display = "The string specified does not contain a valid UCI notation move")]
    InvalidUciMove,

    /// An attempt was made to convert a string not equal to "1"-"8" to a rank
    #[fail(display = "The string specified does not contain a valid rank")]
    InvalidRank,

    /// An attempt was made to convert a string not equal to "a"-"h" to a file
    #[fail(display = "The string specified does not contain a valid file")]
    InvalidFile,
}
