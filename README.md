# JSON Repair

A lightweight Python module that attempts to repair nearly-invalid JSON strings into valid JSON.

## Features

- **Conservative approach**: Only applies transformations when confidently fixable
- **High performance**: Uses `orjson` for fast JSON parsing
- **Common fixes**:
  - Trailing commas before `}` or `]`
  - Single quotes instead of double quotes
  - Missing braces for object-like text

## Installation

```bash
pip install repair-json
```

Or with uv:

```bash
uv add repair-json
```

## Usage

```python
from json_repair import repair_json

# Remove trailing commas
repair_json('{"a": 1,}')  # '{"a": 1}'

# Convert single quotes to double quotes
repair_json("{'a': 1}")  # '{"a": 1}'

# Wrap raw object-like text missing braces
repair_json('a: 1')  # '{"a": 1}'

# Valid JSON passes through unchanged
repair_json('{"valid": true}')  # '{"valid": true}'
```

## API

### `repair_json(s: str) -> str`

Attempts to repair an invalid JSON string.

**Args:**
- `s`: Potentially invalid JSON string

**Returns:**
- Repaired JSON string, or original string if unrepairable

## Performance

Benchmark comparison against Rust implementations (actson and aws-smithy-json) across various JSON sizes and error types:

| Size | Type | Python (ms) | Actson (ms) | AWS Smithy (ms) | vs Actson | vs AWS Smithy |
|------|------|-------------|-------------|-----------------|-----------|---------------|
| 55 | valid_json | 0.0004 | 0.0001 | 0.0001 | 6.81x | 3.57x |
| 42 | trailing_comma | 0.0049 | 0.0006 | 0.0005 | 8.23x | 10.19x |
| 32 | nested | 0.0045 | 0.0004 | 0.0004 | 10.13x | 12.80x |
| 39 | mixed | 0.0053 | 0.0006 | 0.0005 | 8.28x | 11.05x |
| 319 | valid_json | 0.0019 | 0.0003 | 0.0005 | 7.36x | 4.03x |
| 312 | trailing_comma | 0.0092 | 0.0018 | 0.0024 | 5.21x | 3.74x |
| 473 | nested | 0.0115 | 0.0048 | 0.0036 | 2.42x | 3.19x |
| 335 | mixed | 0.0127 | 0.0036 | 0.0030 | 3.56x | 4.16x |
| 3391 | valid_json | 0.0140 | 0.0043 | 0.0042 | 3.27x | 3.32x |
| 3562 | trailing_comma | 0.0477 | 0.0191 | 0.0257 | 2.49x | 1.85x |
| 5187 | nested | 0.0445 | 0.0413 | 0.0353 | 1.08x | 1.26x |
| 3858 | mixed | 0.0478 | 0.0415 | 0.0182 | 1.15x | 2.63x |
| 37261 | valid_json | 0.1269 | 0.0268 | 0.0444 | 4.73x | 2.86x |
| 39672 | trailing_comma | 0.2900 | 0.1945 | 0.1430 | 1.49x | 2.03x |
| 54865 | nested | 0.2677 | 0.2256 | 0.2125 | 1.19x | 1.26x |
| 43303 | mixed | 0.6033 | 0.2492 | 0.1950 | 2.42x | 3.09x |
| 407461 | valid_json | 1.5866 | 0.2328 | 0.2223 | 6.82x | 7.14x |
| 438572 | trailing_comma | 3.5597 | 2.1658 | 1.5196 | 1.64x | 2.34x |
| 576863 | nested | 2.7495 | 1.3600 | 2.1338 | 2.02x | 1.29x |
| 479768 | mixed | 6.8474 | 2.6287 | 1.9773 | 2.60x | 3.46x |
| 4424461 | valid_json | 31.0216 | 2.5779 | 2.6047 | 12.03x | 11.91x |
| 4805572 | trailing_comma | 60.9054 | 24.1378 | 15.6485 | 2.52x | 3.89x |
| 6048861 | nested | 38.3782 | 12.9313 | 22.2813 | 2.97x | 1.72x |
| 5264433 | mixed | 118.2079 | 28.2360 | 21.6915 | 4.19x | 5.45x |

**Overall Summary:**

| Implementation | Avg Time (ms) | Success Rate | Speedup vs Python |
|----------------|---------------|--------------|-------------------|
| Python | 11.0312 | 0% | - |
| Rust Actson | 3.1285 | 87.5% | 3.53x |
| Rust AWS Smithy | 2.8653 | 100% | 3.85x |

*Benchmark: 100 warmup iterations, 100 measured iterations per test case*

## Development

This project uses [uv](https://github.com/astral-sh/uv) for dependency management.

```bash
# Install dependencies
uv sync

# Run the module directly
python json_repair.py

# Run tests
python -m pytest
```

## Requirements

- Python >= 3.14
- orjson >= 3.0.0
