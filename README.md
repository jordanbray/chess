# A Fast Chess Library In Rust

[![Build Status](https://travis-ci.org/jordanbray/chess.svg?branch=master)](https://travis-ci.org/jordanbray/chess)
[![crates.io](https://img.shields.io/crates/v/chess.svg)](https://crates.io/crates/chess)

This library handles the process of move generation within a chess engine or chess UI.

This library is follows semver for version numbering in the format MAJOR.MINOR.PATCH.  That means:

* Any change to the API that breaks existing code will involve a MAJOR version number change.
* Any added functionality or features that do not break existing applications will involve a MINOR version number change.
* Any bug fixes or performance improvements that do not affect users will involve a PATCH version change.

# Shakmaty

Another rust chess library is in the 'shakmaty' crate.  This is a great library, with many more features than this one.  It supports various chess variants, as well as the UCI protocol.  However, those features come at a cost, and this library performs consistently faster in all test cases I can throw at it.  To compare the two, I have added 'shakmaty' support to the 'chess_perft' application, and moved a bunch of benchmarks to that crate.  You can view the results at
https://github.com/jordanbray/chess_perft.

## What It Does

This library allows you to create a chess board from a FEN-formatted string, list all legal moves for the chess board and make moves.

This library also allows you to view various pieces of board-state information such as castle rights.

This library has very fast move generation (the primary purposes of its existance), which will be optimized more.  All the tricks to make chess move generation fast are used.

## What It Does Not Do

This is not a chess engine, just the move generator.  This is not a chess UI, just the move generator.  This is not a chess PGN parser, database, UCI communicator, XBOARD/WinBoard protocol, website or grandmaster.  Just a humble move generator.

## Compile-time Options

When compiling, I definitely recommend using RUSTFLAGS="-C target-cpu=native", specifically to gain access to the popcnt and ctzl instruction available on almost all modern CPUs.  This is used internally to figure out how many pieces are on a bitboard, and what square a piece is on respectively.  Because of the type system used here, these tasks become literally a single instruction.

## API Documentation

... is available at http://jordanbray.github.io/chess/.

## Anything Else

Nope.  Have fun.

