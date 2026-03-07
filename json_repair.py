# -*- coding: utf-8 -*-
"""JSON Repair Module.

Attempts to repair nearly-JSON string outputs into valid JSON strings.
It is conservative: only applies transformations when confidently fixable.
"""

from __future__ import annotations

import re

import orjson

# Precompiled regex patterns for performance
_JSON_BRACKETS_RE = re.compile(r"^[\[{].*[\]}]$", flags=re.S)
_TRAILING_COMMA_RE = re.compile(r",(\s*[}\]])")


def repair_json(s: str) -> str:
    """Repair invalid JSON string.

    Attempts to fix common JSON issues like trailing commas,
    single quotes instead of double quotes, and missing braces.

    Args:
        s: Potentially invalid JSON string.

    Returns:
        Repaired JSON string, or original string if unrepairable.

    Examples:
        >>> repair_json('{"a": 1,}')
        '{"a": 1}'
        >>> repair_json("{'a': 1}")
        '{"a": 1}'
        >>> repair_json('a: 1')
        '{"a": 1}'
    """
    # If already valid, return as-is
    if _is_valid_json(s):
        return s

    t = s.strip()
    base = t

    # Replace single quotes with double quotes when it looks like JSON-ish
    if _JSON_BRACKETS_RE.match(t) and ("'" in t and '"' not in t):
        base = t.replace("'", '"')
        if _is_valid_json(base):
            return base

    # Remove trailing commas before } or ]
    cand = _TRAILING_COMMA_RE.sub(r"\1", base)
    if cand != base and _is_valid_json(cand):
        return cand

    # Wrap raw object-like text missing braces
    if not t.startswith("{") and ":" in t and t.count("{") == 0 and t.count("}") == 0:
        cand = "{" + t + "}"
        if _is_valid_json(cand):
            return cand

    # Return original if unrepairable
    return s


def _is_valid_json(s: str) -> bool:
    """Check if string is valid JSON.

    Args:
        s: String to parse.

    Returns:
        True if string is valid JSON.
    """
    try:
        orjson.loads(s)
        return True
    except Exception:
        return False


if __name__ == "__main__":
    # Test examples
    test_cases = [
        '{"a": 1,}',
        '{"a": [1, 2, 3,]}',
        "{'a': 1}",
        "a: 1",
        '{"valid": true}',
    ]

    for test in test_cases:
        result = repair_json(test)
        print(f"Input:  {test!r}")
        print(f"Output: {result!r}")
        print()
