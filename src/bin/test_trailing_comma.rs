use aws_smithy_json::deserialize::json_token_iter;

fn test_json(name: &str, input: &str) {
    let mut iter = json_token_iter(input.as_bytes());
    let mut result = "OK".to_string();
    
    loop {
        match iter.next() {
            Some(Ok(_)) => continue,
            Some(Err(e)) => {
                result = format!("ERROR: {:?}", e);
                break;
            }
            None => break,
        }
    }
    
    println!("{:30} - {}", name, result);
}

fn main() {
    println!("Testing AWS Smithy JSON with trailing commas:\n");
    
    // Test trailing commas in arrays
    test_json("Array with trailing comma", r#"[1, 2, 3,]"#);
    test_json("Array without trailing comma", r#"[1, 2, 3]"#);
    
    // Test trailing commas in objects
    test_json("Object with trailing comma", r#"{"a": 1,}"#);
    test_json("Object without trailing comma", r#"{"a": 1}"#);
    
    // Test nested trailing commas
    test_json("Nested with trailing comma", r#"{"a": [1, 2,],}"#);
    test_json("Nested without trailing comma", r#"{"a": [1, 2]}"#);
    
    // Test multiple trailing commas
    test_json("Multiple trailing commas", r#"{"a": [1,], "b": 2,}"#);
    
    // Test whitespace after trailing comma
    test_json("Trailing comma + space", r#"[1, 2, 3, ]"#);
    test_json("Trailing comma + newline", "[1, 2, 3,\n]");
}
