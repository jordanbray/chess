# A Fast Chess Library In Rust

This library handles the process of move generation within a chess engine or chess UI.

This library is still a work in progress, but, it does work well.  Expect API changes.  I'm hoping to get this into a stable state for release relatively soon.

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

