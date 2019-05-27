use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid FEN string: {}", fen)]
    InvalidFen { fen: String },
    #[fail(
        display = "The board specified did not pass sanity checks.  Are you sure the kings exist and the side to move cannot capture the opposing king?"
    )]
    InvalidBoard,
}
