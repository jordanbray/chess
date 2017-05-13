#[macro_use]
extern crate bencher;
extern crate chess;

use bencher::Bencher;
use chess::{Board, ChessMove, Color, ALL_SQUARES};

const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const MIDDLEGAME_FEN: &str = "rn1qkb1r/pbp2ppp/1p2p3/3n4/8/2N2NP1/PP1PPPBP/R1BQ1RK1 b kq - 1 7";

fn perft_4(bench: &mut Bencher) {
    let pos = Board::from_fen(INITIAL_FEN.to_owned()).expect("valid fen");
    bench.iter(|| assert_eq!(pos.perft(4), 197281));
}

fn perft_5(bench: &mut Bencher) {
    let pos = Board::from_fen(INITIAL_FEN.to_owned()).expect("valid fen");
    bench.iter(|| assert_eq!(pos.perft(5), 4865609));
}

fn enumerate_moves(bench: &mut Bencher) {
    let pos = Board::from_fen(MIDDLEGAME_FEN.to_owned()).expect("valid fen");
    bench.iter(|| {
        let mut moves = [ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256];
        assert_eq!(pos.enumerate_moves(&mut moves), 39);
    });
}

fn make_move(bench: &mut Bencher) {
    let pos = Board::from_fen(MIDDLEGAME_FEN.to_owned()).expect("valid fen");
    let m = ChessMove::new(ALL_SQUARES[61], ALL_SQUARES[52], None);
    bench.iter(|| {
       let after = pos.make_move(m);
       assert_eq!(after.side_to_move(), Color::White);
    });
}

benchmark_group!(benches, perft_4, perft_5, enumerate_moves, make_move);
benchmark_main!(benches);
