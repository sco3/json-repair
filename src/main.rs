use actson::{feeder::SliceJsonFeeder, options::JsonParserOptionsBuilder, JsonParser};
use std::fs;

/// Repairs broken JSON by removing trailing commas in arrays and objects.
/// Uses a streaming approach to detect and remove trailing commas before parsing.
fn repair_json(input: &str) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    
    // Stack to track container types: true = array, false = object
    let mut container_stack: Vec<bool> = Vec::new();
    
    // Track if we're inside a string
    let mut in_string = false;
    let mut escape_next = false;
    
    // Track position of last comma - only remove if followed by ] or }
    let mut last_comma_pos: Option<usize> = None;
    
    while let Some(c) = chars.next() {
        if escape_next {
            output.push(c);
            escape_next = false;
            continue;
        }
        
        if in_string {
            output.push(c);
            if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        
        match c {
            '"' => {
                output.push(c);
                in_string = true;
                // Comma before a key is valid (between key-value pairs)
                last_comma_pos = None;
            }
            '[' => {
                output.push(c);
                container_stack.push(true); // true = array
                last_comma_pos = None;
            }
            ']' => {
                // Remove trailing comma before closing bracket
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&true) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            '{' => {
                output.push(c);
                container_stack.push(false); // false = object
                last_comma_pos = None;
            }
            '}' => {
                // Remove trailing comma before closing brace
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&false) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            ',' => {
                // Mark this comma position - will be removed if next non-whitespace is ] or }
                last_comma_pos = Some(output.len());
                output.push(c);
            }
            ':' => {
                // Colon means we're seeing a key-value separator
                // Comma before colon in object is valid (between pairs)
                last_comma_pos = None;
                output.push(c);
            }
            _ => {
                // Whitespace or value characters (numbers, true, false, null)
                // If it's a value character, the comma before it is valid
                if !c.is_whitespace() {
                    last_comma_pos = None;
                }
                output.push(c);
            }
        }
    }
    
    // Final cleanup: remove any trailing comma at the end
    if let Some(pos) = last_comma_pos.take() {
        output.truncate(pos);
    }
    
    // Validate the repaired JSON using actson
    validate_json(&output)?;
    
    Ok(output)
}

/// Validate that the output is valid JSON using actson
fn validate_json(input: &str) -> Result<(), String> {
    let options = JsonParserOptionsBuilder::default().build();
    let feeder = SliceJsonFeeder::new(input.as_bytes());
    let mut parser = JsonParser::new_with_options(feeder, options);
    
    loop {
        match parser.next_event() {
            Ok(Some(_)) => continue,
            Ok(None) => break,
            Err(e) => return Err(format!("Invalid JSON: {:?}", e)),
        }
    }
    
    Ok(())
}

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

    // Read input file
    let input = match fs::read_to_string(&args[1]) {
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
        print!("{}", repaired);
    }
}
