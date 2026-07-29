//! FORK TEST: verify writes past the upstream 50 MB free-tier cap succeed.
//! Opens a COPY of a real .mv2 file and appends 1 MB chunks until the file
//! is well past 50 MB.
//!
//! Run with: cargo run --release --example fork_capacity_test -- /path/to/copy.mv2

use std::env;
use std::path::PathBuf;

use memvid_core::{Memvid, PutOptions, Result};

fn main() -> Result<()> {
    let path = PathBuf::from(env::args().nth(1).expect("usage: fork_capacity_test <copy.mv2>"));

    let size_before = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!("file before: {:.1} MB", size_before as f64 / 1_048_576.0);

    let mut mem = Memvid::open(&path)?;
    println!("get_capacity(): {:.1} GB", mem.get_capacity() as f64 / 1_073_741_824.0);

    // 1 MB payload of semi-varied text (compressible payload would undercount).
    let mut chunk = String::with_capacity(1_048_576);
    while chunk.len() < 1_048_576 {
        chunk.push_str("fork capacity test lorem ipsum dolor sit amet 0123456789 abcdefgh\n");
    }
    let chunk = chunk.into_bytes();

    for i in 1..=20 {
        let opts = PutOptions::builder()
            .title(format!("fork-capacity-test-{i:02}"))
            .uri(format!("mv2://fork-test/chunk-{i:02}"))
            .build();
        match mem.put_bytes_with_options(&chunk, opts) {
            Ok(seq) => {
                mem.commit()?;
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                println!("put {i:2} ok (seq {seq}) — file now {:.1} MB", size as f64 / 1_048_576.0);
            }
            Err(e) => {
                println!("put {i:2} FAILED: {e}");
                std::process::exit(1);
            }
        }
    }

    let size_after = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!(
        "\nSUCCESS: grew {:.1} MB -> {:.1} MB with no CapacityExceeded",
        size_before as f64 / 1_048_576.0,
        size_after as f64 / 1_048_576.0
    );
    Ok(())
}
