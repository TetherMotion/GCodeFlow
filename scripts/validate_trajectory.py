#!/usr/bin/env python3
"""
Validation script to compare GCodeFlow (Rust/C++) output with Python reference.

Usage:
    python validate_trajectory.py test.gcode
    python validate_trajectory.py --gcode "G0 X10\\nG1 X50 F1000"
"""

import argparse
import csv
import os
import subprocess
import sys
import tempfile
from pathlib import Path

# Import reference implementation
from gcode_reference import GCodeInterpreter, generate_trajectory


def run_gcodeflow(gcode: str, resolution: float, feed_rate: float) -> list:
    """Run GCodeFlow CLI and parse output"""
    # Find gcodeflow binary
    script_dir = Path(__file__).parent
    project_dir = script_dir.parent
    
    # Try common locations
    binary_paths = [
        project_dir / "target" / "release" / "gcodeflow",
        project_dir / "target" / "debug" / "gcodeflow",
    ]
    
    gcodeflow_bin = None
    for path in binary_paths:
        if path.exists():
            gcodeflow_bin = path
            break
    
    if not gcodeflow_bin:
        print("Error: gcodeflow binary not found. Build with 'cargo build --release'", 
              file=sys.stderr)
        return None
    
    # Create temp file for gcode
    with tempfile.NamedTemporaryFile(mode='w', suffix='.gcode', delete=False) as f:
        f.write(gcode)
        gcode_file = f.name
    
    try:
        # Run gcodeflow
        result = subprocess.run(
            [str(gcodeflow_bin), "points", 
             "--file", gcode_file,
             "--resolution", str(resolution),
             "--feed-rate", str(feed_rate),
             "--format", "csv"],
            capture_output=True,
            text=True,
        )
        
        if result.returncode != 0:
            print(f"Error running gcodeflow: {result.stderr}", file=sys.stderr)
            return None
        
        # Parse CSV output
        lines = result.stdout.strip().split('\n')
        if not lines:
            return []
        
        reader = csv.DictReader(lines)
        points = []
        for row in reader:
            points.append({
                'time': float(row['time']),
                'x': float(row['x']),
                'y': float(row['y']),
                'z': float(row['z']),
            })
        
        return points
        
    finally:
        os.unlink(gcode_file)


def run_reference(gcode: str, resolution: float, feed_rate: float) -> list:
    """Run Python reference implementation"""
    interpreter = GCodeInterpreter(
        rapid_rate=6000.0,
        default_feed=feed_rate,
    )
    segments = interpreter.process_gcode(gcode)
    trajectory = generate_trajectory(segments, resolution)
    
    return [
        {
            'time': pt.time,
            'x': pt.position.x,
            'y': pt.position.y,
            'z': pt.position.z,
        }
        for pt in trajectory
    ]


def compare_points(rust_points: list, py_points: list, tolerance: float) -> dict:
    """Compare two point lists"""
    results = {
        'match': True,
        'rust_count': len(rust_points),
        'python_count': len(py_points),
        'max_diff': {'t': 0, 'x': 0, 'y': 0, 'z': 0},
        'mismatches': [],
    }
    
    # Use minimum length for comparison
    min_len = min(len(rust_points), len(py_points))
    
    if len(rust_points) != len(py_points):
        results['match'] = False
        results['count_mismatch'] = True
    
    for i in range(min_len):
        r = rust_points[i]
        p = py_points[i]
        
        dt = abs(r['time'] - p['time'])
        dx = abs(r['x'] - p['x'])
        dy = abs(r['y'] - p['y'])
        dz = abs(r['z'] - p['z'])
        
        results['max_diff']['t'] = max(results['max_diff']['t'], dt)
        results['max_diff']['x'] = max(results['max_diff']['x'], dx)
        results['max_diff']['y'] = max(results['max_diff']['y'], dy)
        results['max_diff']['z'] = max(results['max_diff']['z'], dz)
        
        if dt > tolerance or dx > tolerance or dy > tolerance or dz > tolerance:
            results['match'] = False
            if len(results['mismatches']) < 10:  # Limit stored mismatches
                results['mismatches'].append({
                    'index': i,
                    'rust': r,
                    'python': p,
                    'diff': {'t': dt, 'x': dx, 'y': dy, 'z': dz},
                })
    
    return results


def main():
    parser = argparse.ArgumentParser(description="Validate GCodeFlow against Python reference")
    parser.add_argument('file', nargs='?', help="GCode file to test")
    parser.add_argument('--gcode', '-g', help="GCode string to test")
    parser.add_argument('--resolution', type=float, default=0.01,
                       help="Time resolution in seconds")
    parser.add_argument('--feed-rate', type=float, default=1000.0,
                       help="Default feed rate (mm/min)")
    parser.add_argument('--tolerance', type=float, default=0.01,
                       help="Maximum acceptable difference")
    parser.add_argument('--verbose', '-v', action='store_true',
                       help="Show detailed output")
    
    args = parser.parse_args()
    
    # Get GCode content
    if args.gcode:
        gcode = args.gcode.replace('\\n', '\n')
    elif args.file:
        with open(args.file, 'r') as f:
            gcode = f.read()
    else:
        print("Error: Specify a file or --gcode", file=sys.stderr)
        sys.exit(1)
    
    print(f"Testing GCode ({len(gcode.strip().split(chr(10)))} lines)...")
    if args.verbose:
        print("---")
        print(gcode)
        print("---")
    
    # Run both implementations
    print("Running GCodeFlow (Rust/C++)...")
    rust_points = run_gcodeflow(gcode, args.resolution, args.feed_rate)
    
    if rust_points is None:
        print("Failed to run GCodeFlow")
        sys.exit(1)
    
    print(f"  Generated {len(rust_points)} points")
    
    print("Running Python reference...")
    py_points = run_reference(gcode, args.resolution, args.feed_rate)
    print(f"  Generated {len(py_points)} points")
    
    # Compare
    print("\nComparing trajectories...")
    results = compare_points(rust_points, py_points, args.tolerance)
    
    if results['match']:
        print(f"✓ PASS: All points match within tolerance {args.tolerance}")
        print(f"  Max differences: t={results['max_diff']['t']:.6f}s, "
              f"x={results['max_diff']['x']:.6f}, "
              f"y={results['max_diff']['y']:.6f}, "
              f"z={results['max_diff']['z']:.6f}")
        sys.exit(0)
    else:
        print(f"✗ FAIL: Trajectories differ")
        
        if 'count_mismatch' in results:
            print(f"  Point count: Rust={results['rust_count']}, Python={results['python_count']}")
        
        print(f"  Max differences: t={results['max_diff']['t']:.6f}s, "
              f"x={results['max_diff']['x']:.6f}, "
              f"y={results['max_diff']['y']:.6f}, "
              f"z={results['max_diff']['z']:.6f}")
        
        if results['mismatches']:
            print(f"\n  First {len(results['mismatches'])} mismatches:")
            for m in results['mismatches']:
                print(f"    Point {m['index']}:")
                print(f"      Rust:   ({m['rust']['x']:.4f}, {m['rust']['y']:.4f}, {m['rust']['z']:.4f}) @ t={m['rust']['time']:.4f}")
                print(f"      Python: ({m['python']['x']:.4f}, {m['python']['y']:.4f}, {m['python']['z']:.4f}) @ t={m['python']['time']:.4f}")
        
        sys.exit(1)


if __name__ == '__main__':
    main()
