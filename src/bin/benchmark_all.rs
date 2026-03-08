/// High-performance benchmark comparing Python vs Rust JSON repair
/// 
/// This benchmark runs entirely in Rust for maximum performance:
/// - Python: spawns subprocess (unavoidable)
/// - Rust (val): native single-pass with serde_json validation
/// - Rust (no val): native single-pass without final validation

use std::time::Instant;
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;

// Single-pass with validation
mod rust_val {
    use serde::de::IgnoredAny;

    fn is_valid_json(input: &[u8]) -> bool {
        serde_json::from_slice::<IgnoredAny>(input).is_ok()
    }

    pub fn repair_json(input: &[u8]) -> Result<Vec<u8>, String> {
        if is_valid_json(input) {
            return Ok(input.to_vec());
        }

        let mut output = Vec::with_capacity(input.len());
        let bytes = input.iter().copied();
        let mut container_stack: Vec<bool> = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut string_quote: Option<u8> = None;
        let mut last_comma_pos: Option<usize> = None;

        for c in bytes {
            if escape_next {
                if c == b'\'' && string_quote == Some(b'\'') {
                    output.truncate(output.len().saturating_sub(1));
                    output.push(c);
                } else {
                    output.push(c);
                }
                escape_next = false;
                continue;
            }

            if in_string {
                if c == b'\\' {
                    escape_next = true;
                    output.push(c);
                } else if Some(c) == string_quote {
                    in_string = false;
                    string_quote = None;
                    output.push(b'"');
                } else if c == b'"' && string_quote == Some(b'\'') {
                    output.push(b'\\');
                    output.push(b'"');
                } else {
                    output.push(c);
                }
                continue;
            }

            match c {
                b'\'' => {
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
                b'{' => {
                    output.push(c);
                    container_stack.push(false);
                    last_comma_pos = None;
                }
                b'}' => {
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

        if let Some(pos) = last_comma_pos.take() {
            output.truncate(pos);
        }

        if !container_stack.is_empty() {
            return Err(format!("Unclosed containers: {}", container_stack.len()));
        }

        if !is_valid_json(&output) {
            return Err("Invalid JSON after repair".to_string());
        }

        Ok(output)
    }
}

// Single-pass without validation
mod rust_no_val {
    use serde::de::IgnoredAny;

    fn is_valid_json(input: &[u8]) -> bool {
        serde_json::from_slice::<IgnoredAny>(input).is_ok()
    }

    pub fn repair_json(input: &[u8]) -> Result<Vec<u8>, String> {
        if is_valid_json(input) {
            return Ok(input.to_vec());
        }

        let mut output = Vec::with_capacity(input.len());
        let bytes = input.iter().copied();
        let mut container_stack: Vec<bool> = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut string_quote: Option<u8> = None;
        let mut last_comma_pos: Option<usize> = None;

        for c in bytes {
            if escape_next {
                if c == b'\'' && string_quote == Some(b'\'') {
                    output.truncate(output.len().saturating_sub(1));
                    output.push(c);
                } else {
                    output.push(c);
                }
                escape_next = false;
                continue;
            }

            if in_string {
                if c == b'\\' {
                    escape_next = true;
                    output.push(c);
                } else if Some(c) == string_quote {
                    in_string = false;
                    string_quote = None;
                    output.push(b'"');
                } else if c == b'"' && string_quote == Some(b'\'') {
                    output.push(b'\\');
                    output.push(b'"');
                } else {
                    output.push(c);
                }
                continue;
            }

            match c {
                b'\'' => {
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
                b'{' => {
                    output.push(c);
                    container_stack.push(false);
                    last_comma_pos = None;
                }
                b'}' => {
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

        if let Some(pos) = last_comma_pos.take() {
            output.truncate(pos);
        }

        if !container_stack.is_empty() {
            return Err(format!("Unclosed containers: {}", container_stack.len()));
        }

        Ok(output)
    }
}

fn generate_test_data(size: usize, error_type: &str) -> Vec<u8> {
    let mut s = String::with_capacity(size);
    
    match error_type {
        "trailing_comma" => {
            s.push_str("{'items':[");
            let mut i = 0;
            while s.len() < size - 20 {
                if i > 0 { s.push(','); }
                s.push_str(&format!("{}", i));
                i += 1;
            }
            s.push_str("],}");
        }
        "single_quotes" => {
            s.push_str("{'key':'value'");
            let mut depth = 1;
            while s.len() < size - 30 {
                s.push_str(",'nested':{'a':'b'");
                depth += 1;
            }
            s.push_str(&"}".repeat(depth));
            s.push('}');
        }
        "mixed" => {
            s.push_str("{'items':[1,2,3,],'name':'test'");
            while s.len() < size - 50 {
                s.push_str(",'extra':{'a':[1,]}");
            }
            s.push('}');
        }
        "valid" => {
            s.push_str("{\"items\":[");
            let mut i = 0;
            while s.len() < size - 20 {
                if i > 0 { s.push(','); }
                s.push_str(&format!("{}", i));
                i += 1;
            }
            s.push_str("]}");
        }
        _ => {}
    }
    
    s.into_bytes()
}

fn benchmark_rust<F>(func: F, data: &[u8], warmup: usize, iterations: usize) -> (f64, usize) 
where
    F: Fn(&[u8]) -> Result<Vec<u8>, String>
{
    for _ in 0..warmup {
        let _ = func(data);
    }
    
    let start = Instant::now();
    let mut success = 0;
    for _ in 0..iterations {
        if func(data).is_ok() {
            success += 1;
        }
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() / iterations as f64 * 1000.0;
    (avg_ms, success)
}

fn benchmark_python(data: &[u8], warmup: usize, iterations: usize) -> (f64, usize) {
    let project_dir = env!("CARGO_MANIFEST_DIR");
    let python_path = format!("{}/.venv/bin/python", project_dir);
    
    let script = format!(r#"
import sys
sys.path.insert(0, r"{project_dir}")
from json_repair import repair_json
data = sys.stdin.buffer.read().decode('utf-8')
warmup = int(sys.argv[1])
iters = int(sys.argv[2])
for _ in range(warmup):
    repair_json(data)
import time
start = time.perf_counter()
success = 0
for _ in range(iters):
    try:
        if repair_json(data):
            success += 1
    except:
        pass
elapsed = time.perf_counter() - start
print(f"{{elapsed*1000/iters:.6}},{{success}}")
"#);
    
    // Write script to temp file
    let script_path = "/tmp/py_bench.py";
    fs::write(script_path, script).ok();
    
    // Run Python benchmark with venv python
    let output = Command::new(&python_path)
        .arg(script_path)
        .arg(warmup.to_string())
        .arg(iterations.to_string())
        .current_dir(project_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(data).ok();
            }
            child.wait_with_output().ok()
        });
    
    // Cleanup
    fs::remove_file(script_path).ok();
    
    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = stdout.trim().split(',').collect();
        if parts.len() >= 2 {
            let avg_ms: f64 = parts[0].parse().unwrap_or(f64::INFINITY);
            let success: usize = parts[1].parse().unwrap_or(0);
            return (avg_ms, success);
        }
    }
    
    (f64::INFINITY, 0)
}

fn main() {
    let warmup = 100;
    let iterations = 1000;
    
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║         Python vs Rust Single-Pass Performance Benchmark (Rust native)       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Warmup: {} | Iterations: {}", warmup, iterations);
    println!();
    
    let test_cases = [
        ("trailing_comma", "Trailing Commas"),
        ("single_quotes", "Single Quotes"),
        ("mixed", "Mixed Issues"),
        ("valid", "Valid JSON"),
    ];
    
    let sizes = [
        (1000, "1KB"),
        (10000, "10KB"),
        (100000, "100KB"),
        (500000, "500KB"),
    ];
    
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ By Error Type (10KB)                                                            │");
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Error Type          │ Python         │ Rust (val)     │ Rust (no val)  │ Speedup│");
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    
    for (error_type, display_name) in &test_cases {
        let data = generate_test_data(10000, error_type);
        
        let (py_time, py_success) = benchmark_python(&data, warmup, iterations);
        let (rust_val_time, rust_val_success) = benchmark_rust(rust_val::repair_json, &data, warmup, iterations);
        let (rust_nv_time, rust_nv_success) = benchmark_rust(rust_no_val::repair_json, &data, warmup, iterations);
        
        let speedup_val = if rust_val_time > 0.0 { py_time / rust_val_time } else { f64::INFINITY };
        let speedup_nv = if rust_nv_time > 0.0 { py_time / rust_nv_time } else { f64::INFINITY };
        
        let py_status = if py_success == iterations { "✓" } else { "✗" };
        let rust_val_status = if rust_val_success == iterations { "✓" } else { "✗" };
        let rust_nv_status = if rust_nv_success == iterations { "✓" } else { "✗" };
        
        println!("│ {:<20}│ {:8.4}ms {} │ {:8.4}ms {} │ {:8.4}ms {} │ {:5.1}x/{:4.1}x │", 
                 display_name, py_time, py_status, 
                 rust_val_time, rust_val_status, 
                 rust_nv_time, rust_nv_status,
                 speedup_val, speedup_nv);
    }
    
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ By Size (Mixed Errors)                                                          │");
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Size                │ Python         │ Rust (val)     │ Rust (no val)  │ Speedup│");
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    
    for (size, display_name) in &sizes {
        let data = generate_test_data(*size, "mixed");
        
        let (py_time, py_success) = benchmark_python(&data, warmup, iterations);
        let (rust_val_time, rust_val_success) = benchmark_rust(rust_val::repair_json, &data, warmup, iterations);
        let (rust_nv_time, rust_nv_success) = benchmark_rust(rust_no_val::repair_json, &data, warmup, iterations);
        
        let speedup_val = if rust_val_time > 0.0 { py_time / rust_val_time } else { f64::INFINITY };
        let speedup_nv = if rust_nv_time > 0.0 { py_time / rust_nv_time } else { f64::INFINITY };
        
        let py_status = if py_success == iterations { "✓" } else { "✗" };
        let rust_val_status = if rust_val_success == iterations { "✓" } else { "✗" };
        let rust_nv_status = if rust_nv_success == iterations { "✓" } else { "✗" };
        
        println!("│ {:<20}│ {:8.4}ms {} │ {:8.4}ms {} │ {:8.4}ms {} │ {:5.1}x/{:4.1}x │", 
                 display_name, py_time, py_status, 
                 rust_val_time, rust_val_status, 
                 rust_nv_time, rust_nv_status,
                 speedup_val, speedup_nv);
    }
    
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    // Overall summary
    let mut total_py = 0.0;
    let mut total_rust_val = 0.0;
    let mut total_rust_nv = 0.0;
    
    for (error_type, _) in &test_cases {
        let data = generate_test_data(10000, error_type);
        let (py, _) = benchmark_python(&data, 10, 100);
        let (rv, _) = benchmark_rust(rust_val::repair_json, &data, 10, 100);
        let (rnv, _) = benchmark_rust(rust_no_val::repair_json, &data, 10, 100);
        total_py += py;
        total_rust_val += rv;
        total_rust_nv += rnv;
    }
    
    let overall_speedup_val = if total_rust_val > 0.0 { total_py / total_rust_val } else { f64::INFINITY };
    let overall_speedup_nv = if total_rust_nv > 0.0 { total_py / total_rust_nv } else { f64::INFINITY };
    
    println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ Overall Summary (by error type)                                                 │");
    println!("├─────────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Python:         {:8.4}ms total                                                   │", total_py);
    println!("│ Rust (val):     {:8.4}ms total                                                   │", total_rust_val);
    println!("│ Rust (no val):  {:8.4}ms total                                                   │", total_rust_nv);
    println!("│ Speedup (val):  {:5.1}x                                                          │", overall_speedup_val);
    println!("│ Speedup (no val): {:5.1}x                                                          │", overall_speedup_nv);
    println!("└─────────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("Key Findings:");
    println!("• Python uses two-pass: repair + orjson validation");
    println!("• Rust (val) uses single-pass + serde_json validation");
    println!("• Rust (no val) uses pure single-pass with structural validation only");
    println!("• All benchmarks run natively in Rust (no subprocess overhead for Rust)");
}
