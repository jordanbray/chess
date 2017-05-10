#[macro_use]
extern crate bencher;
extern crate chess;

use bencher::Bencher;
use chess::Board;

const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn perft_4(bench: &mut Bencher) {
    let pos = Board::from_fen(INITIAL_FEN.to_owned()).expect("valid fen");
    bench.iter(|| assert_eq!(pos.perft(4), 197281));
}

fn perft_5(bench: &mut Bencher) {
    let pos = Board::from_fen(INITIAL_FEN.to_owned()).expect("valid fen");
    bench.iter(|| assert_eq!(pos.perft(5), 4865609));
}

benchmark_group!(benches, perft_4, perft_5);
benchmark_main!(benches);
