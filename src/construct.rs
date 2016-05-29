use bitboard::BitBoard;
use zobrist::Zobrist;
use std::sync::{Once, ONCE_INIT};
use magic;

static SETUP: Once = ONCE_INIT;

/// Call before using any objects within this library.
/// Must be called at least once.
/// Can be called more than once, and is thread safe.
pub fn construct() {
    SETUP.call_once(|| {
        Zobrist::construct();
        BitBoard::construct();
        magic::construct();
    });
}
