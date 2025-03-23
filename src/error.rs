use std::fmt;

/// Sometimes, bad stuff happens.
#[derive(Clone, Debug)]
pub enum InvalidError {
    /// The FEN string is invalid
    #[cfg(feature = "std")]
    FEN { fen: String },
    #[cfg(not(feature = "std"))]
    FEN,

    /// The board created from BoardBuilder was found to be invalid
    Board,

    /// An attempt was made to create a square from an invalid string
    Square,

    /// An attempt was made to create a move from an invalid SAN string
    SanMove,

    /// An atempt was made to create a move from an invalid UCI string
    UciMove,

    /// An attempt was made to convert a string not equal to "1"-"8" to a rank
    Rank,

    /// An attempt was made to convert a string not equal to "a"-"h" to a file
    File,
}

impl fmt::Display for InvalidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature="std")]
            Self::FEN{ fen: s } => write!(f, "Invalid FEN string: {}", s),
            #[cfg(not(feature="std"))]
            Self::FEN => write!(f, "Invalid FEN string."),
            Self::Board => write!(f, "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"),
            Self::Square => write!(f, "The string specified does not contain a valid algebraic notation square."),
            Self::SanMove => write!(f, "The string specified does not contain a valid SAN notation move"),
            Self::UciMove => write!(f, "The string specified does not contain a valid UCI notation move"),
            Self::Rank => write!(f, "The string specified does not contain a valid rank."),
            Self::File => write!(f, "The string specified does not contain a valid file.")
        }
    }
}
