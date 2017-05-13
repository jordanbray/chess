#[macro_use]
extern crate bencher;
extern crate chess;

use bencher::Bencher;
use chess::{Board, ChessMove, Color, ALL_SQUARES, MoveGen};

const MIDDLEGAME_FEN: &str = "rn1qkb1r/pbp2ppp/1p2p3/3n4/8/2N2NP1/PP1PPPBP/R1BQ1RK1 b kq - 1 7";

// This is a helper function to remove boilerplate code from all the perft_* _movegenbenchmarks
fn movegen_perft(bench: &mut Bencher, fen: String, depth: usize, count: usize) {
     let pos = Board::from_fen(fen).expect("Valid FEN");

     bench.iter(|| assert_eq!(MoveGen::movegen_perft_test(pos, depth), count));
}

// This is a helper function to remove boilerplate code from all the perft_* _boardbenchmarks
fn board_perft(bench: &mut Bencher, fen: String, depth: u64, count: u64) {
    let pos = Board::from_fen(fen).expect("Valid FEN");

    bench.iter(|| assert_eq!(pos.perft(depth), count));
}

fn board_enumerate_moves(bench: &mut Bencher) {
    let pos = Board::from_fen(MIDDLEGAME_FEN.to_owned()).expect("valid fen");
    bench.iter(|| {
        let mut moves = [ChessMove::new(ALL_SQUARES[0], ALL_SQUARES[0], None); 256];
        assert_eq!(pos.enumerate_moves(&mut moves), 39);
    });
}

fn board_make_move(bench: &mut Bencher) {
    let pos = Board::from_fen(MIDDLEGAME_FEN.to_owned()).expect("valid fen");
    let m = ChessMove::new(ALL_SQUARES[61], ALL_SQUARES[52], None);
    bench.iter(|| {
       let after = pos.make_move(m);
       assert_eq!(after.side_to_move(), Color::White);
    });
}

// These first two contain a technically invalid FEN position
// The position is completely valid, except it cannot be reached by any set of legal moves.
// Many chess move generators fail here due to a particular en-passant optimization.
// Should these two test ever fail, it should fail with an invaild FEN error, not an incorrect
// move count.

fn perft_01_board(bench: &mut Bencher) {
    board_perft(bench, "8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064);
}

fn perft_02_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064);
}

// These are all normal perft tests.

fn perft_03_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467);
}

fn perft_04_board(bench: &mut Bencher) {
    board_perft(bench, "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467);
}

fn perft_05_board(bench: &mut Bencher) {
    board_perft(bench, "5k2/8/8/8/8/8/8/4K2R w K - 0 1".to_owned(), 6, 661072);
}

fn perft_06_board(bench: &mut Bencher) {
    board_perft(bench, "4k2r/8/8/8/8/8/8/5K2 b k - 0 1".to_owned(), 6, 661072);
}

fn perft_07_board(bench: &mut Bencher) {
    board_perft(bench, "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1".to_owned(), 6, 803711);
}

fn perft_08_board(bench: &mut Bencher) {
    board_perft(bench, "r3k3/8/8/8/8/8/8/3K4 b q - 0 1".to_owned(), 6, 803711);
}

fn perft_09_board(bench: &mut Bencher) {
    board_perft(bench, "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(), 4, 1274206);
}

fn perft_10_board(bench: &mut Bencher) {
    board_perft(bench, "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(), 4, 1274206);
}

fn perft_11_board(bench: &mut Bencher) {
    board_perft(bench, "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(), 4, 1720476);
}

fn perft_12_board(bench: &mut Bencher) {
    board_perft(bench, "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(), 4, 1720476);
}

fn perft_13_board(bench: &mut Bencher) {
    board_perft(bench, "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

fn perft_14_board(bench: &mut Bencher) {
    board_perft(bench, "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

fn perft_15_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

fn perft_16_board(bench: &mut Bencher) {
    board_perft(bench, "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

fn perft_17_board(bench: &mut Bencher) {
    board_perft(bench, "4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

fn perft_18_board(bench: &mut Bencher) {
    board_perft(bench, "8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

fn perft_19_board(bench: &mut Bencher) {
    board_perft(bench, "8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

fn perft_20_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

fn perft_21_board(bench: &mut Bencher) {
    board_perft(bench, "K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

fn perft_22_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

fn perft_23_board(bench: &mut Bencher) {
    board_perft(bench, "8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

fn perft_24_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

fn perft_25_board(bench: &mut Bencher) {
    board_perft(bench, "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

fn perft_26_board(bench: &mut Bencher) {
    board_perft(bench, "8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}

fn perft_kiwipete_board(bench: &mut Bencher) {
    board_perft(bench, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(), 5, 193690690);
}

// Movegen Struct Tests.  Same as above

fn perft_01_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064); // Invalid FEN
}

fn perft_02_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064); // Invalid FEN
}

fn perft_03_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467);
}

fn perft_04_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467);
}

fn perft_05_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "5k2/8/8/8/8/8/8/4K2R w K - 0 1".to_owned(), 6, 661072);
}

fn perft_06_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "4k2r/8/8/8/8/8/8/5K2 b k - 0 1".to_owned(), 6, 661072);
}

fn perft_07_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1".to_owned(), 6, 803711);
}

fn perft_08_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k3/8/8/8/8/8/8/3K4 b q - 0 1".to_owned(), 6, 803711);
}

fn perft_09_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(), 4, 1274206);
}

fn perft_10_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(), 4, 1274206);
}

fn perft_11_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(), 4, 1720476);
}

fn perft_12_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(), 4, 1720476);
}

fn perft_13_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

fn perft_14_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

fn perft_15_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

fn perft_16_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

fn perft_17_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

fn perft_18_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

fn perft_19_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

fn perft_20_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

fn perft_21_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

fn perft_22_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

fn perft_23_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

fn perft_24_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

fn perft_25_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

fn perft_26_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}

fn perft_kiwipete_movegen(bench: &mut Bencher) {
    movegen_perft(bench, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(), 5, 193690690);
}


benchmark_group!(
    benches,
    perft_01_board,
    perft_02_board,
    perft_03_board,
    perft_04_board,
    perft_05_board,
    perft_06_board,
    perft_07_board,
    perft_08_board,
    perft_09_board,
    perft_10_board,
    perft_11_board,
    perft_12_board,
    perft_13_board,
    perft_14_board,
    perft_15_board,
    perft_16_board,
    perft_17_board,
    perft_18_board,
    perft_19_board,
    perft_20_board,
    perft_21_board,
    perft_22_board,
    perft_23_board,
    perft_24_board,
    perft_25_board,
    perft_26_board,
    perft_01_movegen,
    perft_02_movegen,
    perft_03_movegen,
    perft_04_movegen,
    perft_05_movegen,
    perft_06_movegen,
    perft_07_movegen,
    perft_08_movegen,
    perft_09_movegen,
    perft_10_movegen,
    perft_11_movegen,
    perft_12_movegen,
    perft_13_movegen,
    perft_14_movegen,
    perft_15_movegen,
    perft_16_movegen,
    perft_17_movegen,
    perft_18_movegen,
    perft_19_movegen,
    perft_20_movegen,
    perft_21_movegen,
    perft_22_movegen,
    perft_23_movegen,
    perft_24_movegen,
    perft_25_movegen,
    perft_26_movegen,
    perft_kiwipete_board,
    perft_kiwipete_movegen,
    board_enumerate_moves,
    board_make_move);

benchmark_main!(benches);
