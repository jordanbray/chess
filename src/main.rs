mod color;
mod piece;
mod square;
mod chess_move;
mod bitboard;
mod castle_rights;
mod board;
mod magic;

extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate time;

fn main() {
    bitboard::BitBoard::construct();
    magic::Magic::construct();
    let board: board::Board = board::Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned());
    let start = time::precise_time_s();
    println!("Perft {}: {}", 6, board.perft(6));
    let end = time::precise_time_s();
    println!("Performed in {} seconds", end - start);
}
