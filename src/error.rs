use failure::Fail;

/// Sometimes, bad stuff happens.
#[derive(Debug, Fail)]
pub enum Error {
    /// The FEN string is invalid
    #[fail(display = "Invalid FEN string: {}", fen)]
    InvalidFen { fen: String },

    /// The board created from BoardBuilder was found to be invalid
    #[fail(
        display = "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"
    )]
    InvalidBoard,
}
