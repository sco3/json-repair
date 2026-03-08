use serde::de::IgnoredAny;
use std::fs;

/// Repairs broken JSON by removing trailing commas in arrays and objects.
/// Uses a single-pass streaming approach with integrated structural validation.
fn repair_json(input: &str) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    // Stack to track container types: true = array, false = object
    let mut container_stack: Vec<bool> = Vec::new();

    // Track if we're inside a string
    let mut in_string = false;
    let mut escape_next = false;
    // Track which quote type opened the string
    let mut string_quote: Option<char> = None;

    // Track position of last comma - only remove if followed by ] or }
    let mut last_comma_pos: Option<usize> = None;

    while let Some(c) = chars.next() {
        if in_string {
            // Inside a string - handle escape sequences and quotes
            if escape_next {
                // Previous character was a backslash
                if c == '\'' && string_quote == Some('\'') {
                    // Escaped single quote in single-quoted string: \'
                    // In output (double-quoted), single quotes don't need escaping
                    // Remove the backslash from output
                    output.truncate(output.len().saturating_sub(1));
                    output.push(c); // Keep the single quote as-is
                } else {
                    // Other escape sequences - keep as-is
                    output.push(c);
                }
                escape_next = false;
                continue;
            }

            if c == '\\' {
                escape_next = true;
                output.push(c);
            } else if Some(c) == string_quote {
                // End of string - convert to double quote in output
                in_string = false;
                string_quote = None;
                output.push('"');
            } else if c == '"' && string_quote == Some('\'') {
                // Double quote inside single-quoted string - escape it
                output.push('\\');
                output.push('"');
            } else {
                output.push(c);
            }
            continue;
        }

        match c {
            '\'' => {
                // Start of string with single quote - convert to double quote
                output.push('"');
                in_string = true;
                string_quote = Some('\'');
                last_comma_pos = None;
            }
            '"' => {
                output.push(c);
                in_string = true;
                string_quote = Some('"');
                last_comma_pos = None;
            }
            '[' => {
                output.push(c);
                container_stack.push(true);
                last_comma_pos = None;
            }
            ']' => {
                if container_stack.last() != Some(&true) {
                    continue;
                }
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                container_stack.pop();
                last_comma_pos = None;
            }
            '{' => {
                output.push(c);
                container_stack.push(false);
                last_comma_pos = None;
            }
            '}' => {
                if container_stack.last() != Some(&false) {
                    continue;
                }
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                container_stack.pop();
                last_comma_pos = None;
            }
            ',' => {
                // Always record comma position - will be removed if next non-whitespace is ] or }
                last_comma_pos = Some(output.len());
                output.push(c);
            }
            ':' => {
                // Colon means key-value separator - clear trailing comma marker
                last_comma_pos = None;
                output.push(c);
            }
            _ => {
                // Whitespace or value characters
                if !c.is_whitespace() {
                    // Non-whitespace value character - clear trailing comma marker
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

    // Final structural validation: all containers must be closed
    if !container_stack.is_empty() {
        return Err(format!(
            "Unclosed containers: {} unclosed brackets",
            container_stack.len()
        ));
    }

    // Validate the repaired JSON using serde_json (fast check)
    if !is_valid_json(&output) {
        return Err("Invalid JSON structure after repair".to_string());
    }

    Ok(output)
}

/// Fast validation using serde_json with IgnoredAny (doesn't build full values)
fn is_valid_json(input: &str) -> bool {
    serde_json::from_str::<IgnoredAny>(input).is_ok()
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
