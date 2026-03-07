#!/usr/bin/env python3
"""Test single quote repair."""

from json_repair import repair_json
import json

test_cases = [
    ('{"asdf":"asdf"}', "Already valid"),
    ('{"asdf":\'asdf\'}', "Single quote value"),
    ("{'asdf':'asdf'}", "Single quotes everywhere"),
    ("{'asdf':\"asdf\"}", "Single quote key"),
    ('[1, 2, 3,]', "Trailing comma"),
    ('{"a": [1,2,],}', "Both issues"),
]

print("Testing Python json_repair with single quotes:")
print("=" * 60)

for test, description in test_cases:
    result = repair_json(test)
    try:
        json.loads(result)
        status = "✓ Valid"
    except:
        status = "✗ Invalid"
    
    changed = "→" if result != test else "="
    print(f"{description:25s}: {test!r:25s} {changed} {result!r:25s} [{status}]")
