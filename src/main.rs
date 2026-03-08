use std::fs;
use repair_json::repair_json;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.json> [output.json]", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  input.json   - Path to the broken JSON file");
        eprintln!("  output.json  - Optional path for output (defaults to stdout)");
        std::process::exit(1);
    }

    // Read input file as bytes
    let input = match fs::read(&args[1]) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args[1], e);
            std::process::exit(1);
        }
    };

    let repaired = match repair_json(&input) {
        Ok(repaired) => repaired,
        Err(e) => {
            eprintln!("Failed to repair JSON: {}", e);
            std::process::exit(1);
        }
    };

    // Write output
    if args.len() >= 3 {
        // Write to output file
        match fs::write(&args[2], &repaired) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error writing to file '{}': {}", args[2], e);
                std::process::exit(1);
            }
        }
    } else {
        // Write to stdout
        print!("{}", String::from_utf8_lossy(&repaired));
    }
}
