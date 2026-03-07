use repair_json::repair_json_aws_smithy;

fn main() {
    let test_cases = vec![
        (r#"{"asdf":"asdf"}"#, "Already valid"),
        (r#"{"asdf":'asdf'}"#, "Single quote value"),
        (r#"{'asdf':'asdf'}"#, "Single quotes everywhere"),
        (r#"{'asdf':"asdf"}"#, "Single quote key"),
        (r#"[1, 2, 3,]"#, "Trailing comma"),
        (r#"{"a": [1,2,],}"#, "Both issues"),
    ];

    println!("Testing Rust AWS Smithy repair with single quotes:");
    println!("============================================================");

    for (test, description) in test_cases {
        match repair_json_aws_smithy(test.as_bytes()) {
            Ok(result) => {
                let result_str = String::from_utf8_lossy(&result);
                let status = if result_str == test { "=" } else { "->" };
                println!("{:<25}: {:<25} {} {:<25} [OK]", description, test, status, result_str);
            }
            Err(e) => {
                println!("{:<25}: {:<25} = {:<25} [Error: {}]", description, test, test, e);
            }
        }
    }
}
