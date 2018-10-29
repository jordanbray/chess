# A Fast Chess Library In Rust

[![Build Status](https://travis-ci.org/jordanbray/chess.svg?branch=master)](https://travis-ci.org/jordanbray/chess)
[![crates.io](https://img.shields.io/crates/v/chess.svg)](https://crates.io/crates/chess)

This library handles the process of move generation within a chess engine or chess UI.

This library is follows semver for version numbering in the format MAJOR.MINOR.PATCH.  That means:

* Any change to the API that breaks existing code will involve a MAJOR version number change.
* Any added functionality or features that do not break existing applications will involve a MINOR version number change.
* Any bug fixes or performance improvements that do not affect users will involve a PATCH version change.

## Requires Rust 1.27 or Greater

This library requires rust version 1.27 or greater in order to check for the BMI2 instruction-set at compile-time.

## Examples

### Simple Move Generation

This puts all moves into a static array.  An array is used instead of a Vec to keep the moves on the stack, and to allow reuse of the array (which has a big impact on performance).

```rust
  use chess::{Board, ChessMove};

  let board = Board::default();
  let mut moves = [ChessMove::default(); 256];
  let count = board.enumerate_moves(&mut moves);
  assert_eq!(count, 20);
```

### Incremental Move Generation With Capture/Non-Capture Sorting

Here we iterate over all moves with incremental move generation.  The iterator below will generate moves as you are going through the list, which is ideal for situations where not all moves will be looked at (such as in an engine search function).

```rust
  use chess::MoveGen;
  use chess::Board;
  use chess::EMPTY;

  // create a board with the initial position
  let board = Board::default();

  // create an iterable
  let mut iterable = MoveGen::new(board, true);

  // make sure .len() works.
  assert_eq!(iterable.len(), 20); // the .len() function does *not* consume the iterator

  // lets iterate over targets.
  let targets = board.color_combined(!board.side_to_move());
  iterable.set_iterator_mask(targets);

  // count the number of targets
  let mut count = 0;
  for _ in &mut iterable {
      count += 1;
      // This move captures one of my opponents pieces (with the exception of en passant)
  }

  // now, iterate over the rest of the moves
  iterable.set_iterator_mask(!EMPTY);
  for _ in &mut iterable {
      count += 1;
      // This move does not capture anything
  }

  // make sure it works
  assert_eq!(count, 20);
```

### Making a Move

Here we make a move on the chess board.  The board is a copy-on-make structure, meaning every time you make a move, you create a new chess board.  The board structure is optimized for size to reduce copy-time.

```rust
  use chess::{Board, ChessMove, Square, Rank, File, Color};

  let m = ChessMove::new(Square::make_square(Rank::Second, File::D),
                         Square::make_square(Rank::Fourth, File::D),
                         None);

  let board = Board::default();
  assert_eq!(board.make_move_new(m).side_to_move(), Color::Black);
```

## Compile-time Options

When compiling, I definitely recommend using RUSTFLAGS="-C target-cpu=native", specifically to gain access to the popcnt and ctzl instruction available on almost all modern CPUs.  This is used internally to figure out how many pieces are on a bitboard, and what square a piece is on respectively.  Because of the type system used here, these tasks become literally a single instruction.  Additionally, BMI2 is enabled on machines with the instructions by using this flag.

## BMI2

As of version 1.0.3 of this library, the BMI2 instruction-set is used on machines that support it.  This speeds up the logic in two ways:
* It uses built-in instructions to do the same logic that magic bitboards do.
* It reduces cache load by storing moves in a u16 rather than a u64, which can be decompressed to a u64 with a single instruction.

On targets without BMI2, the library falls back on magic bitboards.  This is checked at compile-time.

## Shakmaty

Another rust chess library is in the 'shakmaty' crate.  This is a great library, with many more features than this one.  It supports various chess variants, as well as the UCI protocol.  However, those features come at a cost, and this library performs consistently faster in all test cases I can throw at it.  To compare the two, I have added 'shakmaty' support to the 'chess_perft' application, and moved a bunch of benchmarks to that crate.  You can view the results at
https://github.com/jordanbray/chess_perft.

## What It Does

This library allows you to create a chess board from a FEN-formatted string, list all legal moves for the chess board and make moves.

This library also allows you to view various pieces of board-state information such as castle rights.

This library has very fast move generation (the primary purposes of its existance), which will be optimized more.  All the tricks to make chess move generation fast are used.

## What It Does Not Do

This is not a chess engine, just the move generator.  This is not a chess UI, just the move generator.  This is not a chess PGN parser, database, UCI communicator, XBOARD/WinBoard protocol, website or grandmaster.  Just a humble move generator.

## API Documentation

... is available at http://jordanbray.github.io/chess/.

## Anything Else

Nope.  Have fun.

