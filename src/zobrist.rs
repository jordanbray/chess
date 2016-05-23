use piece::{Piece, NUM_PIECES};
use square::{Square, NUM_SQUARES};
use color::{Color, NUM_COLORS};
use castle_rights::{CastleRights, NUM_CASTLE_RIGHTS};
use bitboard::{BitBoard, EMPTY};

static mut ZOBRIST_PIECES: [[[BitBoard; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[[EMPTY; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS];
static mut ZOBRIST_CASTLES: [[BitBoard; NUM_CASTLE_RIGHTS]; NUM_COLORS] = [[EMPTY; NUM_CASTLE_RIGHTS]; NUM_COLORS];


