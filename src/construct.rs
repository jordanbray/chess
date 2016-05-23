use magic::Magic;
use bitboard::BitBoard;
use std::sync::{Once, ONCE_INIT};

static SETUP: Once = ONCE_INIT;
pub fn construct() {
    unsafe {
        SETUP.call_once(|| {
            BitBoard::construct();
            Magic::construct();
        });
    }
}
