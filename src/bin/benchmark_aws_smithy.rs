/// Simple benchmark that reads from stdin and repairs JSON
/// Used for Python vs Rust comparison benchmarks

use repair_json::repair_json;
use std::env;
use std::io::{self, Read, Write};
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If arguments provided, use file/benchmark mode
    if args.len() >= 3 {
        let input_file = &args[1];
        let warmup_iters: usize = args.get(2).and_then(|s: &String| s.parse().ok()).unwrap_or(100);
        let benchmark_iters: usize = args.get(3).and_then(|s: &String| s.parse().ok()).unwrap_or(1000);

        let input = match std::fs::read(input_file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", input_file, e);
                std::process::exit(1);
            }
        };

        // Warmup
        for _ in 0..warmup_iters {
            let _ = repair_json(&input);
        }

        // Benchmark
        let start = Instant::now();
        let mut success = 0;
        for _ in 0..benchmark_iters {
            if repair_json(&input).is_ok() {
                success += 1;
            }
        }
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_secs_f64() / benchmark_iters as f64 * 1000.0;

        println!("RUST|{}|{}|{}|{:.6}|{}",
                 input.len(), warmup_iters, benchmark_iters, avg_ms, success);
        return;
    }

    // Otherwise, read from stdin and repair (single operation mode)
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("Failed to read stdin");

    match repair_json(&input) {
        Ok(output) => {
            io::stdout().write_all(&output).expect("Failed to write output");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
