use bitboard;
use zobrist::Zobrist;
use std::sync::{Once, ONCE_INIT};

static SETUP: Once = ONCE_INIT;

/// Call before using any objects within this library.
/// Must be called at least once.
/// Can be called more than once, and is thread safe.
pub fn construct() {
    SETUP.call_once(|| {
        bitboard::construct();
        Zobrist::construct();
    });
}
