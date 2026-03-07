use actson::{feeder::SliceJsonFeeder, options::JsonParserOptionsBuilder, JsonParser};
use serde::de::IgnoredAny;

/// Fast validation using serde_json with IgnoredAny (doesn't build full values)
fn is_valid_json(input: &[u8]) -> bool {
    serde_json::from_slice::<IgnoredAny>(input).is_ok()
}

/// Repairs broken JSON by removing trailing commas in arrays and objects
/// and converting single quotes to double quotes.
/// Uses a streaming approach to detect and remove trailing commas before parsing.
/// Validates using actson (streaming JSON parser).
/// Works with bytes directly to avoid string conversion overhead.
pub fn repair_json(input: &[u8]) -> Result<Vec<u8>, String> {
    // Fast path: if already valid JSON, return as-is
    if is_valid_json(input) {
        return Ok(input.to_vec());
    }

    let mut output = Vec::with_capacity(input.len());
    let bytes = input.iter().copied();

    // Stack to track container types: true = array, false = object
    let mut container_stack: Vec<bool> = Vec::new();

    // Track if we're inside a string (regardless of quote type)
    let mut in_string = false;
    let mut escape_next = false;

    // Track position of last comma - only remove if followed by ] or }
    let mut last_comma_pos: Option<usize> = None;

    for c in bytes {
        if escape_next {
            output.push(c);
            escape_next = false;
            continue;
        }

        if in_string {
            // Inside a string - convert single quotes to double quotes
            if c == b'\'' {
                output.push(b'"');
            } else {
                output.push(c);
            }

            if c == b'\\' {
                escape_next = true;
            } else if c == b'"' || c == b'\'' {
                // End of string (either quote type ends it)
                in_string = false;
            }
            continue;
        }

        match c {
            b'\'' => {
                // Start of string with single quote - convert to double quote
                output.push(b'"');
                in_string = true;
                last_comma_pos = None;
            }
            b'"' => {
                output.push(c);
                in_string = true;
                last_comma_pos = None;
            }
            b'[' => {
                output.push(c);
                container_stack.push(true);
                last_comma_pos = None;
            }
            b']' => {
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&true) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            b'{' => {
                output.push(c);
                container_stack.push(false);
                last_comma_pos = None;
            }
            b'}' => {
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&false) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            b',' => {
                last_comma_pos = Some(output.len());
                output.push(c);
            }
            b':' => {
                last_comma_pos = None;
                output.push(c);
            }
            _ => {
                if !c.is_ascii_whitespace() {
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

    // Validate using actson
    validate_json_actson(&output)?;

    Ok(output)
}

/// Validate that the output is valid JSON using actson
fn validate_json_actson(input: &[u8]) -> Result<(), String> {
    let options = JsonParserOptionsBuilder::default().build();
    let feeder = SliceJsonFeeder::new(input);
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

/// Repairs broken JSON by removing trailing commas in arrays and objects
/// and converting single quotes to double quotes.
/// Uses a streaming approach to detect and remove trailing commas before parsing.
/// Validates using aws-smithy-json (AWS SDK JSON parser).
/// Works with bytes directly to avoid string conversion overhead.
pub fn repair_json_aws_smithy(input: &[u8]) -> Result<Vec<u8>, String> {
    // Fast path: if already valid JSON, return as-is
    if is_valid_json(input) {
        return Ok(input.to_vec());
    }

    let mut output = Vec::with_capacity(input.len());
    let bytes = input.iter().copied();

    // Stack to track container types: true = array, false = object
    let mut container_stack: Vec<bool> = Vec::new();

    // Track if we're inside a string (regardless of quote type)
    let mut in_string = false;
    let mut escape_next = false;

    // Track position of last comma - only remove if followed by ] or }
    let mut last_comma_pos: Option<usize> = None;

    for c in bytes {
        if escape_next {
            output.push(c);
            escape_next = false;
            continue;
        }

        if in_string {
            // Inside a string - convert single quotes to double quotes
            if c == b'\'' {
                output.push(b'"');
            } else {
                output.push(c);
            }

            if c == b'\\' {
                escape_next = true;
            } else if c == b'"' || c == b'\'' {
                // End of string (either quote type ends it)
                in_string = false;
            }
            continue;
        }

        match c {
            b'\'' => {
                // Start of string with single quote - convert to double quote
                output.push(b'"');
                in_string = true;
                last_comma_pos = None;
            }
            b'"' => {
                output.push(c);
                in_string = true;
                last_comma_pos = None;
            }
            b'[' => {
                output.push(c);
                container_stack.push(true);
                last_comma_pos = None;
            }
            b']' => {
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&true) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            b'{' => {
                output.push(c);
                container_stack.push(false);
                last_comma_pos = None;
            }
            b'}' => {
                if let Some(pos) = last_comma_pos.take() {
                    output.truncate(pos);
                }
                output.push(c);
                if container_stack.last() == Some(&false) {
                    container_stack.pop();
                }
                last_comma_pos = None;
            }
            b',' => {
                last_comma_pos = Some(output.len());
                output.push(c);
            }
            b':' => {
                last_comma_pos = None;
                output.push(c);
            }
            _ => {
                if !c.is_ascii_whitespace() {
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

    // Validate using aws-smithy-json
    validate_json_aws_smithy(&output)?;

    Ok(output)
}

/// Validate that the output is valid JSON using aws-smithy-json
fn validate_json_aws_smithy(input: &[u8]) -> Result<(), String> {
    use aws_smithy_json::deserialize::json_token_iter;

    let mut iter = json_token_iter(input);
    loop {
        match iter.next() {
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(format!("Invalid JSON: {:?}", e)),
            None => break,
        }
    }

    Ok(())
}
