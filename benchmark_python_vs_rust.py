#!/usr/bin/env python3
"""
Benchmark: Python vs Rust Single-Pass vs Rust Single-Pass (no validation)

This benchmark compares the actual repair performance by:
1. Running Python repair directly
2. Running Rust benchmarks via their native binaries (no subprocess per iteration)
"""

import subprocess
import time
import sys
import os
import csv

def generate_test_data(size: int, error_type: str) -> str:
    """Generate test JSON data with specific error types."""
    s = []
    
    if error_type == "trailing_comma":
        s.append('{"items":[')
        i = 0
        while len(''.join(s)) < size - 20:
            if i > 0:
                s.append(',')
            s.append(str(i))
            i += 1
        s.append('],}')
    elif error_type == "single_quotes":
        s.append("{'key':'value'")
        depth = 1
        while len(''.join(s)) < size - 30:
            s.append(",'nested':{'a':'b'")
            depth += 1
        s.append('}' * depth + '}')
    elif error_type == "mixed":
        s.append("{'items':[1,2,3,],'name':'test'")
        while len(''.join(s)) < size - 50:
            s.append(",'extra':{'a':[1,]}")
        s.append('}')
    elif error_type == "valid":
        s.append('{"items":[')
        i = 0
        while len(''.join(s)) < size - 20:
            if i > 0:
                s.append(',')
            s.append(str(i))
            i += 1
        s.append(']}')
    
    return ''.join(s)

def benchmark_python(data: str, warmup: int, iterations: int) -> tuple:
    """Benchmark Python json_repair."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from json_repair import repair_json as py_repair
    
    # Warmup
    for _ in range(warmup):
        try:
            py_repair(data)
        except:
            pass
    
    # Benchmark
    start = time.perf_counter()
    success = 0
    for _ in range(iterations):
        try:
            result = py_repair(data)
            if result:
                success += 1
        except:
            pass
    elapsed = time.perf_counter() - start
    avg_ms = (elapsed / iterations) * 1000
    return avg_ms, success

def run_rust_benchmark(data: str, warmup: int, iterations: int, binary: str) -> tuple:
    """Run Rust benchmark binary and parse output."""
    project_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Write test data to temp file
    test_file = os.path.join(project_dir, 'test_bench_temp.json')
    with open(test_file, 'w') as f:
        f.write(data)
    
    try:
        binary_path = os.path.join(project_dir, 'target', 'release', binary)
        
        if not os.path.exists(binary_path):
            subprocess.run(['cargo', 'build', '--release'], cwd=project_dir, check=True)
        
        result = subprocess.run(
            [binary_path, test_file, str(warmup), str(iterations)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Parse output: RUST_ACTSON|size|warmup|iters|avg_ms|success
        output = result.stdout.strip()
        if '|' in output:
            parts = output.split('|')
            if len(parts) >= 5:
                avg_ms = float(parts[4])
                success = int(parts[5]) if len(parts) > 5 else iterations
                return avg_ms, success
        
        return float('inf'), 0
    except Exception as e:
        print(f"Rust benchmark error: {e}", file=sys.stderr)
        return float('inf'), 0
    finally:
        if os.path.exists(test_file):
            os.remove(test_file)

def main():
    warmup = 100
    iterations = 1000
    
    print("╔══════════════════════════════════════════════════════════════════════════════╗")
    print("║     Python vs Rust Single-Pass vs Rust Single-Pass (no validation)           ║")
    print("╚══════════════════════════════════════════════════════════════════════════════╝")
    print()
    print(f"Warmup: {warmup} | Iterations: {iterations}")
    print()
    
    test_cases = [
        ("trailing_comma", "Trailing Commas"),
        ("single_quotes", "Single Quotes"),
        ("mixed", "Mixed Issues"),
        ("valid", "Valid JSON"),
    ]
    
    sizes = [
        (1000, "1KB"),
        (10000, "10KB"),
        (100000, "100KB"),
        (500000, "500KB"),
    ]
    
    all_results = []
    
    print("┌─────────────────────────────────────────────────────────────────────────────────┐")
    print("│ By Error Type (10KB)                                                            │")
    print("├─────────────────────────────────────────────────────────────────────────────────┤")
    print("│ Error Type          │ Python         │ Rust (val)     │ Rust (no val)  │ Speedup│")
    print("├─────────────────────────────────────────────────────────────────────────────────┤")
    
    for error_type, display_name in test_cases:
        data = generate_test_data(10000, error_type)
        
        py_time, py_success = benchmark_python(data, warmup, iterations)
        rust_time, rust_success = run_rust_benchmark(data, warmup, iterations, 'benchmark_actson')
        rust_nv_time, rust_nv_success = run_rust_benchmark(data, warmup, iterations, 'benchmark_aws_smithy')
        
        speedup_val = py_time / rust_time if rust_time > 0 else float('inf')
        speedup_nv = py_time / rust_nv_time if rust_nv_time > 0 else float('inf')
        
        py_status = "✓" if py_success == iterations else "✗"
        rust_status = "✓" if rust_success == iterations else "✗"
        rust_nv_status = "✓" if rust_nv_success == iterations else "✗"
        
        print(f"│ {display_name:<20}│ {py_time:>8.4f}ms {py_status} │ {rust_time:>8.4f}ms {rust_status} │ {rust_nv_time:>8.4f}ms {rust_nv_status} │ {speedup_val:>5.1f}x/{speedup_nv:>4.1f}x │")
        
        all_results.append({
            'size': 10000,
            'type': error_type,
            'python_ms': py_time,
            'rust_val_ms': rust_time,
            'rust_no_val_ms': rust_nv_time,
            'python_success': py_success,
            'rust_success': rust_success,
            'rust_no_val_success': rust_nv_success,
            'iterations': iterations,
        })
    
    print("└─────────────────────────────────────────────────────────────────────────────────┘")
    print()
    
    print("┌─────────────────────────────────────────────────────────────────────────────────┐")
    print("│ By Size (Mixed Errors)                                                          │")
    print("├─────────────────────────────────────────────────────────────────────────────────┤")
    print("│ Size                │ Python         │ Rust (val)     │ Rust (no val)  │ Speedup│")
    print("├─────────────────────────────────────────────────────────────────────────────────┤")
    
    for size, display_name in sizes:
        data = generate_test_data(size, "mixed")
        
        py_time, py_success = benchmark_python(data, warmup, iterations)
        rust_time, rust_success = run_rust_benchmark(data, warmup, iterations, 'benchmark_actson')
        rust_nv_time, rust_nv_success = run_rust_benchmark(data, warmup, iterations, 'benchmark_aws_smithy')
        
        speedup_val = py_time / rust_time if rust_time > 0 else float('inf')
        speedup_nv = py_time / rust_nv_time if rust_nv_time > 0 else float('inf')
        
        py_status = "✓" if py_success == iterations else "✗"
        rust_status = "✓" if rust_success == iterations else "✗"
        rust_nv_status = "✓" if rust_nv_success == iterations else "✗"
        
        print(f"│ {display_name:<20}│ {py_time:>8.4f}ms {py_status} │ {rust_time:>8.4f}ms {rust_status} │ {rust_nv_time:>8.4f}ms {rust_nv_status} │ {speedup_val:>5.1f}x/{speedup_nv:>4.1f}x │")
        
        all_results.append({
            'size': size,
            'type': 'mixed',
            'python_ms': py_time,
            'rust_val_ms': rust_time,
            'rust_no_val_ms': rust_nv_time,
            'python_success': py_success,
            'rust_success': rust_success,
            'rust_no_val_success': rust_nv_success,
            'iterations': iterations,
        })
    
    print("└─────────────────────────────────────────────────────────────────────────────────┘")
    print()
    
    # Overall summary
    total_py_time = sum(r['python_ms'] for r in all_results[:len(test_cases)])
    total_rust_time = sum(r['rust_val_ms'] for r in all_results[:len(test_cases)])
    total_rust_nv_time = sum(r['rust_no_val_ms'] for r in all_results[:len(test_cases)])
    
    total_py_success = sum(r['python_success'] for r in all_results[:len(test_cases)])
    total_rust_success = sum(r['rust_success'] for r in all_results[:len(test_cases)])
    total_rust_nv_success = sum(r['rust_no_val_success'] for r in all_results[:len(test_cases)])
    
    total_tests = len(test_cases) * iterations
    
    overall_speedup_val = total_py_time / total_rust_time if total_rust_time > 0 else float('inf')
    overall_speedup_nv = total_py_time / total_rust_nv_time if total_rust_nv_time > 0 else float('inf')
    
    print("┌─────────────────────────────────────────────────────────────────────────────────┐")
    print("│ Overall Summary (by error type)                                                 │")
    print("├─────────────────────────────────────────────────────────────────────────────────┤")
    print(f"│ Python:         {total_py_time:>8.4f}ms total ({total_py_success}/{total_tests})                              │")
    print(f"│ Rust (val):     {total_rust_time:>8.4f}ms total ({total_rust_success}/{total_tests})                              │")
    print(f"│ Rust (no val):  {total_rust_nv_time:>8.4f}ms total ({total_rust_nv_success}/{total_tests})                              │")
    print(f"│ Speedup (val):  {overall_speedup_val:>5.1f}x                                                           │")
    print(f"│ Speedup (no val): {overall_speedup_nv:>5.1f}x                                                           │")
    print("└─────────────────────────────────────────────────────────────────────────────────┘")
    print()
    
    # Save results to CSV
    csv_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'benchmark_python_vs_rust.csv')
    with open(csv_file, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=['size', 'type', 'python_ms', 'rust_val_ms', 'rust_no_val_ms', 
                                                'python_success', 'rust_success', 'rust_no_val_success', 'iterations'])
        writer.writeheader()
        writer.writerows(all_results)
    
    print(f"Results saved to: {csv_file}")
    print()
    
    print("Key Findings:")
    print("• Python uses two-pass: repair + orjson validation")
    print("• Rust (val) uses single-pass + serde_json validation")
    print("• Rust (no val) uses pure single-pass with structural validation only")
    print("• Rust benchmarks run natively without subprocess overhead per iteration")
    print()

if __name__ == "__main__":
    main()
