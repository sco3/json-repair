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
