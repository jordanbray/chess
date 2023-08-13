use std::fs::File;
use std::io::Write;
// we use the same types as the rest of the library.
use crate::color::NUM_COLORS;
use crate::file::NUM_FILES;
use crate::piece::NUM_PIECES;
use crate::square::NUM_SQUARES;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

// write the ZOBRIEST_* arrays to a file.  I don't generate it, because its just
// a bunch of random u64s
pub fn write_zobrist(f: &mut File) {
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF12345678);

    write!(f, "const SIDE_TO_MOVE: u64 = {};\n\n", rng.next_u64()).unwrap();

    writeln!(
        f,
        "const ZOBRIST_PIECES: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[["
    )
    .unwrap();
    for i in 0..NUM_COLORS {
        for j in 0..NUM_PIECES {
            for _ in 0..NUM_SQUARES {
                writeln!(f, "    {},", rng.next_u64()).unwrap();
            }
            if j != NUM_PIECES - 1 {
                writeln!(f, "   ], [").unwrap();
            }
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "  ]], [[").unwrap();
        }
    }
    write!(f, "]]];\n\n").unwrap();

    writeln!(f, "const ZOBRIST_CASTLES: [[u64; 4]; NUM_COLORS] = [[").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..4 {
            writeln!(f, "    {},", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "  ], [").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();

    writeln!(f, "const ZOBRIST_EP: [[u64; NUM_FILES]; NUM_COLORS] = [[").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..NUM_FILES {
            writeln!(f, "    {},", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "], [").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();
}
