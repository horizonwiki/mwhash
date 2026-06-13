//! Throughput benchmark for `mwhash`.
//!
//! Run with:
//! ```text
//! cargo run --release 
//! ```
//! 
//! This measures raw throughput (bytes hashed per second) across a few
//! buffer sizes that land in different parts of the memory hierarchy:
//!
//! - 64 KB  — fits in L1/L2 cache on most CPUs, shows the "compute bound" speed.
//! - 1 MB   — typically spills into L3, mixes cache and memory bandwidth.
//! - 1 GB   — far exceeds any cache, dominated by main memory bandwidth.
//!
//! Numbers will vary across CPUs, memory speed, and OS. Treat the result as
//! an order-of-magnitude indicator, not a guaranteed benchmark.

use mwhash::mwhash_seeded;
use std::hint::black_box;
use std::time::{Duration, Instant};

const SEED: u32 = 0x1234_5678;

fn bench(label: &str, data: &[u8], iterations: u32) {
    // cold-cache effects.
    for _ in 0..3 {
        black_box(mwhash_seeded(black_box(data), SEED));
    }

    let start = Instant::now();
    for _ in 0..iterations {
        black_box(mwhash_seeded(black_box(data), SEED));
    }
    let elapsed = start.elapsed();

    print_result(label, data.len(), iterations, elapsed);
}

fn print_result(label: &str, size: usize, iterations: u32, elapsed: Duration) {
    let total_bytes = size as u64 * iterations as u64;
    let total_gb = total_bytes as f64 / 1_000_000_000.0;
    let throughput_gb_s = total_gb / elapsed.as_secs_f64();
    let per_call = elapsed / iterations;

    println!(
        "{label:<10} | size: {size_kb:>10.2} KB | iters: {iterations:>6} | \
total: {elapsed:>9.3?} | per call: {per_call:>10.3?} | throughput: {throughput_gb_s:>6.2} GB/s",
        size_kb = size as f64 / 1024.0,
    );
}

fn main() {
    println!("mwhash throughput benchmark");
    println!("seed = {SEED:#010x}");
    println!("{}", "-".repeat(100));

    let cases: &[(&str, usize, u32)] = &[
        ("64 KB", 64 * 1024, 20_000),
        ("1 MB", 1024 * 1024, 2_000),
        ("1 GB", 1024 * 1024 * 1024, 3),
    ];

    for &(label, size, iterations) in cases {
        
        // closer to real-world input.
        let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        bench(label, &data, iterations);
    }

    println!("{}", "-".repeat(100));
}
