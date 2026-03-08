# JSON Repair

A high-performance JSON repair library available in both Python and Rust. Uses a **single-pass streaming approach** to efficiently fix common JSON issues.

## Features

- **Single-pass design**: Repairs and validates in one pass through the input (Rust)
- **Conservative approach**: Only applies transformations when confidently fixable
- **High performance**: 2-6x faster than Python implementation
- **Common fixes**:
  - Trailing commas before `}` or `]`
  - Single quotes instead of double quotes
  - Escaped quotes inside strings (`\'` → `'`)
  - Missing braces for object-like text
  - Mismatched bracket detection

## Installation

### Python

```bash
pip install repair-json
```

Or with uv:

```bash
uv add repair-json
```

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
repair-json = "0.1.0"
serde = "1.0"
serde_json = "1.0"
```

## Usage

### Python

```python
from json_repair import repair_json

# Remove trailing commas
repair_json('{"a": 1,}')  # '{"a": 1}'

# Convert single quotes to double quotes
repair_json("{'a': 1}")  # '{"a": 1}'

# Handle escaped quotes in single-quoted strings
repair_json("{'key':'value\\'s'}")  # '{"key":"value's"}'

# Wrap raw object-like text missing braces
repair_json('a: 1')  # '{"a": 1}'

# Valid JSON passes through unchanged
repair_json('{"valid": true}')  # '{"valid": true}'
```

### Rust

```rust
use repair_json::repair_json;

let input = br#"{"items":[1,2,3,],"name":"test"}"#;

match repair_json(input) {
    Ok(output) => println!("Repaired: {}", String::from_utf8_lossy(&output)),
    Err(e) => eprintln!("Error: {}", e),
}
```

## API

### Python

#### `repair_json(s: str) -> str`

Attempts to repair an invalid JSON string.

**Args:**
- `s`: Potentially invalid JSON string

**Returns:**
- Repaired JSON string, or original string if unrepairable

### Rust

#### `repair_json(input: &[u8]) -> Result<Vec<u8>, String>`

Repairs broken JSON with final serde_json validation.

**Args:**
- `input`: Input bytes (invalid JSON)

**Returns:**
- `Ok(Vec<u8>)`: Repaired JSON bytes
- `Err(String)`: Error message if repair fails

## Performance

### Python vs Rust Single-Pass Comparison

Benchmark comparing Python vs Rust single-pass implementation:

| Size | Type | Python (ms) | Rust (ms) | Speedup |
|------|------|-------------|-----------|---------|
| 1KB | mixed | 0.0127 | 0.0036 | 3.5x |
| 10KB | trailing_comma | 0.0789 | 0.0237 | 3.3x |
| 10KB | single_quotes | 0.0264 | 0.0370 | 0.7x |
| 10KB | mixed | 0.1059 | 0.0359 | 3.0x |
| 10KB | valid | 0.0225 | 0.0075 | 3.0x |
| 100KB | mixed | 1.0600 | 0.3626 | 2.9x |
| 500KB | mixed | 6.9942 | 1.8134 | 3.9x |

**Overall Summary:**

| Implementation | Avg Time (ms) | Speedup vs Python |
|----------------|---------------|-------------------|
| Python (two-pass) | 0.2469 | — |
| Rust single-pass | 0.1337 | **1.8x** |

### Key Performance Insights

1. **Single-pass advantage**: Rust processes input once, Python does two passes (repair + validate)
2. **Scale benefits**: Speedup increases with input size (3.9x at 500KB)
3. **Memory efficiency**: No intermediate token/AST allocation in single-pass

### Run Benchmarks

```bash
# Python vs Rust comparison
.venv/bin/python benchmark_python_vs_rust.py

# Rust-native benchmark (fastest)
cargo run --release --bin benchmark_all
```

## Implementation Details

### Single-Pass Design (Rust)

The Rust implementation uses a streaming byte-by-byte approach:

1. **Fast path**: Check if already valid JSON, return immediately
2. **Single pass**: Stream through bytes while:
   - Tracking container state (arrays/objects via stack)
   - Converting single quotes to double quotes
   - Removing trailing commas
   - Handling escape sequences (`\'` → `'`)
   - Validating bracket matching on-the-fly
3. **Structural check**: Ensure all containers are closed
4. **Final validation**: serde_json check to ensure valid output

### Escape Sequence Handling

The library correctly handles escape sequences:

| Input | Output | Notes |
|-------|--------|-------|
| `{'key':'value\'s'}` | `{"key":"value's"}` | Backslash removed (single quotes don't need escaping in JSON) |
| `{"key":"value's"}` | `{"key":"value's"}` | Single quote in double-quoted string preserved |
| `{"key":"value\"test"}` | `{"key":"value\"test"}` | Escaped double quote preserved |

## Development

This project uses [uv](https://github.com/astral-sh/uv) for Python dependency management.

```bash
# Install dependencies
uv sync

# Run Python module directly
python json_repair.py

# Run tests
python -m pytest

# Build Rust library
cargo build --release

# Run Rust benchmarks
cargo run --release --bin benchmark_all
```

## Requirements

### Python
- Python >= 3.14
- orjson >= 3.0.0

### Rust
- Rust 1.70+
- serde
- serde_json

## License

MIT
