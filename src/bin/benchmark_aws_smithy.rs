use repair_json::repair_json_aws_smithy;
use std::env;
use std::fs;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input.json> <warmup_iterations> <benchmark_iterations>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let warmup_iters: usize = args[2].parse().unwrap_or(1000);
    let benchmark_iters: usize = args[3].parse().unwrap_or(3000);

    // Read input file into memory as bytes
    let input = match fs::read(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_file, e);
            std::process::exit(1);
        }
    };

    // Warmup phase (not measured)
    for _ in 0..warmup_iters {
        let _ = repair_json_aws_smithy(&input);
    }

    // Benchmark phase (measured)
    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for _ in 0..benchmark_iters {
        match repair_json_aws_smithy(&input) {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() / benchmark_iters as f64 * 1000.0;
    let input_size = input.len();

    // Output results in a parseable format
    println!(
        "AWS_SMITHY|{}|{}|{}|{}|{:.6}|{}",
        input_size,
        warmup_iters,
        benchmark_iters,
        success_count,
        avg_ms,
        error_count
    );
}
