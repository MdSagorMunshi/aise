use aise_core::constants::{M_COL, M_ROW};
use aise_core::{field8, field16};
use std::fs::File;
use std::io::Write;

fn main() {
    let mut out = File::create("crates/aise-core/src/field_b_avx512_tables.rs").unwrap();
    writeln!(out, "pub const M_COL_T_LO: [[[u8; 16]; 16]; 16] = [").unwrap();
    for i in 0..16 {
        writeln!(out, "    [").unwrap();
        for j in 0..16 {
            let c = M_COL[i][j];
            write!(out, "        [").unwrap();
            for n in 0..16 {
                write!(out, "{}, ", field8::mul(c, n as u8)).unwrap();
            }
            writeln!(out, "],").unwrap();
        }
        writeln!(out, "    ],").unwrap();
    }
    writeln!(out, "];\n").unwrap();

    writeln!(out, "pub const M_COL_T_HI: [[[u8; 16]; 16]; 16] = [").unwrap();
    for i in 0..16 {
        writeln!(out, "    [").unwrap();
        for j in 0..16 {
            let c = M_COL[i][j];
            write!(out, "        [").unwrap();
            for n in 0..16 {
                write!(out, "{}, ", field8::mul(c, (n as u8) << 4)).unwrap();
            }
            writeln!(out, "],").unwrap();
        }
        writeln!(out, "    ],").unwrap();
    }
    writeln!(out, "];\n").unwrap();

    writeln!(out, "pub const M_ROW_T: [[[[u16; 16]; 4]; 8]; 8] = [").unwrap();
    for i in 0..8 {
        writeln!(out, "    [").unwrap();
        for j in 0..8 {
            let c = M_ROW[i][j];
            writeln!(out, "        [").unwrap();
            for k in 0..4 {
                write!(out, "            [").unwrap();
                for n in 0..16 {
                    write!(out, "{}, ", field16::mul(c, (n as u16) << (4 * k))).unwrap();
                }
                writeln!(out, "],").unwrap();
            }
            writeln!(out, "        ],").unwrap();
        }
        writeln!(out, "    ],").unwrap();
    }
    writeln!(out, "];\n").unwrap();
}
