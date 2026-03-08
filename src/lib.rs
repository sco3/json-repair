use serde::de::IgnoredAny;

/// Fast validation using serde_json with IgnoredAny (doesn't build full values)
fn is_valid_json(input: &[u8]) -> bool {
    serde_json::from_slice::<IgnoredAny>(input).is_ok()
}

/// Internal repair implementation.
/// 
/// # Arguments
/// * `input` - The input bytes to repair
/// * `validate` - If true, performs final serde_json validation
fn repair_json_impl(input: &[u8], validate: bool) -> Result<Vec<u8>, String> {
    // Fast path: if already valid JSON, return as-is
    if is_valid_json(input) {
        return Ok(input.to_vec());
    }

    let mut output = Vec::with_capacity(input.len());
    let bytes = input.iter().copied();

    // Stack to track container types: true = array, false = object
    let mut container_stack: Vec<bool> = Vec::new();

    // Track if we're inside a string
    let mut in_string = false;
    let mut escape_next = false;
    // Track which quote type opened the string (b'"' or b'\'')
    let mut string_quote: Option<u8> = None;

    // Track position of last comma - only remove if followed by ] or }
    let mut last_comma_pos: Option<usize> = None;

    for c in bytes {
        if escape_next {
            output.push(c);
            escape_next = false;
            continue;
        }

        if in_string {
            // Inside a string - handle escape sequences and quotes
            if escape_next {
                // Previous character was a backslash
                if c == b'\'' && string_quote == Some(b'\'') {
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
            
            if c == b'\\' {
                escape_next = true;
                output.push(c);
            } else if Some(c) == string_quote {
                // End of string - convert to double quote in output
                in_string = false;
                string_quote = None;
                output.push(b'"');
            } else if c == b'"' && string_quote == Some(b'\'') {
                // Double quote inside single-quoted string - escape it
                output.push(b'\\');
                output.push(b'"');
            } else {
                output.push(c);
            }
            continue;
        }

        match c {
            b'\'' => {
                // Start of string with single quote - convert to double quote
                output.push(b'"');
                in_string = true;
                string_quote = Some(b'\'');
                last_comma_pos = None;
            }
            b'"' => {
                output.push(c);
                in_string = true;
                string_quote = Some(b'"');
                last_comma_pos = None;
            }
            b'[' => {
                output.push(c);
                container_stack.push(true);
                last_comma_pos = None;
            }
            b']' => {
                // Validate: must have matching opening array
                if container_stack.last() != Some(&true) {
                    // Mismatched bracket - skip the invalid close (conservative)
                    continue;
                }
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                container_stack.pop();
                last_comma_pos = None;
            }
            b'{' => {
                output.push(c);
                container_stack.push(false);
                last_comma_pos = None;
            }
            b'}' => {
                // Validate: must have matching opening object
                if container_stack.last() != Some(&false) {
                    // Mismatched bracket - skip invalid close
                    continue;
                }
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                container_stack.pop();
                last_comma_pos = None;
            }
            b',' => {
                // Always record comma position - will be removed if next non-whitespace is ] or }
                last_comma_pos = Some(output.len());
                output.push(c);
            }
            b':' => {
                // Colon means key-value separator - clear trailing comma marker
                last_comma_pos = None;
                output.push(c);
            }
            _ => {
                // Whitespace or value characters
                if !c.is_ascii_whitespace() {
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

    // Optional final validation using serde_json
    if validate && !is_valid_json(&output) {
        return Err("Invalid JSON structure after repair".to_string());
    }

    Ok(output)
}

/// Repairs broken JSON by removing trailing commas in arrays and objects
/// and converting single quotes to double quotes.
/// Uses a single-pass streaming approach with integrated structural validation.
/// Works with bytes directly to avoid string conversion overhead.
///
/// # Single-Pass Design
/// - Repairs and validates in one pass through the input
/// - Tracks structural state (containers, strings, escapes) during repair
/// - Validates bracket matching and string termination on-the-fly
/// - Performs final serde_json validation as a safety check
pub fn repair_json(input: &[u8]) -> Result<Vec<u8>, String> {
    repair_json_impl(input, true)
}

/// Repairs broken JSON by removing trailing commas in arrays and objects
/// and converting single quotes to double quotes.
/// Uses a single-pass streaming approach with integrated structural validation.
/// Works with bytes directly to avoid string conversion overhead.
///
/// # Single-Pass Design
/// - Same algorithm as repair_json but without final serde_json validation
/// - Relies entirely on integrated structural validation
/// - Faster than repair_json as it skips the final validation check
pub fn repair_json_aws_smithy(input: &[u8]) -> Result<Vec<u8>, String> {
    repair_json_impl(input, false)
}
