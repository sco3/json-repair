#!/usr/bin/env python3
"""Benchmark comparing Python and Rust JSON repair implementations."""

from __future__ import annotations

import subprocess
import time
from pathlib import Path
from dataclasses import dataclass

# Test case generators
def generate_valid_json(size: int) -> str:
    """Generate valid JSON of approximately given size with unique keys."""
    # Each item is ~60 bytes, calculate needed count upfront
    item_count = max(1, size // 60)
    # Use unique keys for each item to prevent orjson key caching
    items = ','.join([f'{{"key_{i}":"value_{i}","num_{i}":{i},"flag_{i}":true}}' for i in range(item_count)])
    return f'{{"items":[{items}]}}'


def generate_broken_trailing_comma(size: int) -> str:
    """Generate broken JSON with trailing commas and unique keys."""
    # Each item is ~40 bytes, calculate needed count upfront
    item_count = max(1, size // 40)
    items = ','.join([f'{{"id_{i}":{i},"name_{i}":"item_{i}",}}' for i in range(item_count)])
    return f'{{"items":[{items},]}}'


def generate_broken_nested(size: int) -> str:
    """Generate broken JSON with nested trailing commas and unique keys."""
    # Each level adds ~15 bytes, calculate depth upfront
    depth = max(1, (size - 20) // 15)
    result = '{"level_0":{' + ''.join([f'"level_{i}_key":{{' for i in range(1, depth)])
    result += '"final_value":42,}' + ',}' * depth
    return result


def generate_broken_mixed(size: int) -> str:
    """Generate broken JSON with mixed issues and unique keys."""
    # Each item is ~45 bytes, calculate needed count upfront
    item_count = max(1, size // 45)
    items = ','.join([f'{{"arr_{i}":[{i},],"obj_{i}":{{"val_{i}":{i},}},}}' for i in range(item_count)])
    return f'[{items},]'


@dataclass
class BenchmarkResult:
    size: int
    test_type: str
    python_ms: float
    rust_actson_ms: float
    rust_aws_smithy_ms: float
    speedup_actson: float
    speedup_aws_smithy: float
    python_success: int
    rust_actson_success: int
    rust_aws_smithy_success: int


def benchmark_python(json_str: str, warmup: int = 1000, iterations: int = 3000) -> tuple[float, int]:
    """Benchmark Python repair_json function (in-memory, no I/O)."""
    from json_repair import repair_json
    
    # Warmup
    for _ in range(warmup):
        repair_json(json_str)
    
    # Benchmark
    start = time.perf_counter()
    success = 0
    for _ in range(iterations):
        result = repair_json(json_str)
        # Count as success if result is valid JSON
        try:
            json.loads(result)
            success += 1
        except:
            pass
    end = time.perf_counter()
    
    elapsed_ms = (end - start) * 1000
    avg_ms = elapsed_ms / iterations
    return avg_ms, success


def benchmark_rust(binary_name: str, json_str: str, warmup: int = 1000, iterations: int = 3000) -> tuple[float, int]:
    """Benchmark Rust repair-json binary (in-memory via file)."""
    rust_binary = Path(f"./target/release/{binary_name}")
    if not rust_binary.exists():
        raise FileNotFoundError(f"Rust benchmark binary not found. Run: cargo build --release --bin {binary_name}")
    
    # Write input to temp file
    input_file = Path("/tmp/bench_input.json")
    input_file.write_text(json_str)
    
    # Run Rust benchmark binary
    result = subprocess.run(
        [str(rust_binary), str(input_file), str(warmup), str(iterations)],
        capture_output=True,
        text=True,
    )
    
    # Cleanup
    input_file.unlink(missing_ok=True)
    
    if result.returncode != 0:
        raise RuntimeError(f"Rust benchmark failed: {result.stderr}")
    
    # Parse output: NAME|size|warmup|iterations|success|avg_ms|errors
    parts = result.stdout.strip().split('|')
    if len(parts) >= 6:
        avg_ms = float(parts[5])
        success = int(parts[4])
        return avg_ms, success
    
    raise RuntimeError(f"Unexpected Rust output: {result.stdout}")


def run_benchmarks():
    """Run all benchmarks and print results."""
    sizes = [42, 420, 4200, 42000, 420000, 4200000]
    generators = [
        ("valid_json", generate_valid_json),
        ("trailing_comma", generate_broken_trailing_comma),
        ("nested", generate_broken_nested),
        ("mixed", generate_broken_mixed),
    ]

    warmup = 100
    iterations = 100

    print("=" * 120)
    print(f"JSON Repair Benchmark: Python vs Rust (actson) vs Rust (aws-smithy-json)")
    print(f"warmup={warmup}, iterations={iterations}")
    print("=" * 120)
    print()

    results = []

    for size in sizes:
        for name, generator in generators:
            json_str = generator(size)
            actual_size = len(json_str.encode('utf-8'))

            # Python benchmark
            try:
                python_time, python_success = benchmark_python(json_str, warmup, iterations)
            except Exception as e:
                python_time = float('inf')
                python_success = 0
                print(f"Python error ({name}, {size}): {e}")

            # Rust actson benchmark
            try:
                rust_actson_time, rust_actson_success = benchmark_rust("benchmark_actson", json_str, warmup, iterations)
            except Exception as e:
                rust_actson_time = float('inf')
                rust_actson_success = 0
                print(f"Rust actson error ({name}, {size}): {e}")

            # Rust aws_smithy benchmark
            try:
                rust_aws_smithy_time, rust_aws_smithy_success = benchmark_rust("benchmark_aws_smithy", json_str, warmup, iterations)
            except Exception as e:
                rust_aws_smithy_time = float('inf')
                rust_aws_smithy_success = 0
                print(f"Rust aws_smithy error ({name}, {size}): {e}")

            speedup_actson = python_time / rust_actson_time if rust_actson_time > 0 else float('inf')
            speedup_aws_smithy = python_time / rust_aws_smithy_time if rust_aws_smithy_time > 0 else float('inf')

            results.append(BenchmarkResult(
                size=actual_size,
                test_type=name,
                python_ms=python_time,
                rust_actson_ms=rust_actson_time,
                rust_aws_smithy_ms=rust_aws_smithy_time,
                speedup_actson=speedup_actson,
                speedup_aws_smithy=speedup_aws_smithy,
                python_success=python_success,
                rust_actson_success=rust_actson_success,
                rust_aws_smithy_success=rust_aws_smithy_success,
            ))

            status_actson = "✓" if rust_actson_success == iterations else "✗"
            status_aws = "✓" if rust_aws_smithy_success == iterations else "✗"
            print(f"Size: {actual_size:7d} | Type: {name:15s} | "
                  f"Python: {python_time:8.4f}ms | "
                  f"Actson: {rust_actson_time:8.4f}ms {status_actson} ({speedup_actson:6.2f}x) | "
                  f"AWS Smithy: {rust_aws_smithy_time:8.4f}ms {status_aws} ({speedup_aws_smithy:6.2f}x)")
    
    print()
    print("=" * 120)
    print("Summary by Test Type")
    print("=" * 120)

    # Group by type and calculate averages
    for name in sorted(set(r.test_type for r in results)):
        type_results = [r for r in results if r.test_type == name]
        avg_python = sum(r.python_ms for r in type_results) / len(type_results)
        avg_actson = sum(r.rust_actson_ms for r in type_results) / len(type_results)
        avg_aws_smithy = sum(r.rust_aws_smithy_ms for r in type_results) / len(type_results)
        avg_speedup_actson = avg_python / avg_actson if avg_actson > 0 else float('inf')
        avg_speedup_aws_smithy = avg_python / avg_aws_smithy if avg_aws_smithy > 0 else float('inf')
        total_python_success = sum(r.python_success for r in type_results)
        total_actson_success = sum(r.rust_actson_success for r in type_results)
        total_aws_success = sum(r.rust_aws_smithy_success for r in type_results)
        max_success = len(type_results) * iterations
        print(f"{name:15s}: Python: {avg_python:8.4f}ms | "
              f"Actson: {avg_actson:8.4f}ms ({avg_speedup_actson:6.2f}x) | "
              f"AWS Smithy: {avg_aws_smithy:8.4f}ms ({avg_speedup_aws_smithy:6.2f}x)")

    print()
    print("=" * 120)
    print("Summary by Size")
    print("=" * 120)

    # Group by size
    for size in sorted(set(r.size for r in results)):
        size_results = [r for r in results if r.size == size]
        avg_python = sum(r.python_ms for r in size_results) / len(size_results)
        avg_actson = sum(r.rust_actson_ms for r in size_results) / len(size_results)
        avg_aws_smithy = sum(r.rust_aws_smithy_ms for r in size_results) / len(size_results)
        avg_speedup_actson = avg_python / avg_actson if avg_actson > 0 else float('inf')
        avg_speedup_aws_smithy = avg_python / avg_aws_smithy if avg_aws_smithy > 0 else float('inf')
        total_python_success = sum(r.python_success for r in size_results)
        total_actson_success = sum(r.rust_actson_success for r in size_results)
        total_aws_success = sum(r.rust_aws_smithy_success for r in size_results)
        max_success = len(size_results) * iterations
        print(f"Size {size:7d}: Python: {avg_python:8.4f}ms | "
              f"Actson: {avg_actson:8.4f}ms ({avg_speedup_actson:6.2f}x) | "
              f"AWS Smithy: {avg_aws_smithy:8.4f}ms ({avg_speedup_aws_smithy:6.2f}x)")

    print()
    print("=" * 120)
    print("Overall Summary")
    print("=" * 120)

    # Overall averages
    avg_python = sum(r.python_ms for r in results) / len(results)
    avg_actson = sum(r.rust_actson_ms for r in results) / len(results)
    avg_aws_smithy = sum(r.rust_aws_smithy_ms for r in results) / len(results)
    overall_speedup_actson = avg_python / avg_actson if avg_actson > 0 else float('inf')
    overall_speedup_aws_smithy = avg_python / avg_aws_smithy if avg_aws_smithy > 0 else float('inf')
    total_python_success = sum(r.python_success for r in results)
    total_actson_success = sum(r.rust_actson_success for r in results)
    total_aws_success = sum(r.rust_aws_smithy_success for r in results)
    max_success = len(results) * iterations

    print(f"Python:          {avg_python:8.4f}ms ({total_python_success}/{max_success})")
    print(f"Rust Actson:     {avg_actson:8.4f}ms ({total_actson_success}/{max_success}) - Speedup: {overall_speedup_actson:6.2f}x")
    print(f"Rust AWS Smithy: {avg_aws_smithy:8.4f}ms ({total_aws_success}/{max_success}) - Speedup: {overall_speedup_aws_smithy:6.2f}x")

    # Save results to CSV
    csv_path = Path("benchmark_results.csv")
    with open(csv_path, 'w') as f:
        f.write("size,type,python_ms,rust_actson_ms,rust_aws_smithy_ms,speedup_actson,speedup_aws_smithy,python_success,actson_success,aws_smithy_success,iterations\n")
        for r in results:
            f.write(f"{r.size},{r.test_type},{r.python_ms:.6f},{r.rust_actson_ms:.6f},{r.rust_aws_smithy_ms:.6f},{r.speedup_actson:.2f},{r.speedup_aws_smithy:.2f},{r.python_success},{r.rust_actson_success},{r.rust_aws_smithy_success},{iterations}\n")
    print(f"\nResults saved to: {csv_path}")


if __name__ == "__main__":
    run_benchmarks()
