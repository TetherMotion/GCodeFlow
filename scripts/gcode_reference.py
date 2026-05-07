#!/usr/bin/env python3
"""
GCode Reference Implementation for Trajectory Validation

This script provides a pure Python implementation of GCode trajectory generation
that can be used to validate the C++/Rust implementation. It's intentionally
independent of the main codebase to serve as an oracle for testing.

Usage:
    python gcode_reference.py --gcode "G0 X10 Y20\\nG1 X30 Y40 F1000"
    python gcode_reference.py --file test.gcode --format csv
    python gcode_reference.py --file test.gcode --compare output.csv
"""

import argparse
import json
import math
import sys
from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Tuple


class MotionType(Enum):
    RAPID = 0
    LINEAR = 1
    CW_ARC = 2
    CCW_ARC = 3


class Plane(Enum):
    XY = 0  # G17
    XZ = 1  # G18
    YZ = 2  # G19


@dataclass
class Position:
    x: float = 0.0
    y: float = 0.0
    z: float = 0.0
    a: float = 0.0
    b: float = 0.0
    c: float = 0.0


@dataclass
class TrajectoryPoint:
    time: float
    position: Position
    velocity: Position
    motion_type: MotionType
    block_index: int


@dataclass
class MotionSegment:
    start: Position
    end: Position
    center: Optional[Position]  # For arcs
    feed_rate: float  # mm/min
    motion_type: MotionType
    arc_radius: float = 0.0
    arc_sweep: float = 0.0  # radians
    segment_length: float = 0.0
    segment_time: float = 0.0  # seconds
    block_index: int = 0
    plane: Plane = Plane.XY


@dataclass
class MachineState:
    """Current machine state for modal tracking"""
    position: Position = field(default_factory=Position)
    feed_rate: float = 1000.0  # mm/min
    rapid_rate: float = 6000.0  # mm/min
    motion_mode: MotionType = MotionType.RAPID
    plane: Plane = Plane.XY
    absolute: bool = True
    arc_ijk_absolute: bool = False  # IJK relative to start (most common)


def parse_gcode_line(line: str) -> dict:
    """Parse a single GCode line into words"""
    words = {}
    
    # Remove comments
    if ';' in line:
        line = line[:line.index(';')]
    if '(' in line:
        line = line[:line.index('(')]
    
    line = line.strip().upper()
    if not line:
        return words
    
    # Parse words (letter + number)
    i = 0
    while i < len(line):
        if line[i].isalpha():
            letter = line[i]
            i += 1
            # Collect the number (including sign and decimal)
            num_str = ""
            while i < len(line) and (line[i].isdigit() or line[i] in '.-+'):
                num_str += line[i]
                i += 1
            if num_str:
                # Handle multiple values for same letter (e.g., G0 G90)
                value = float(num_str)
                if letter in words:
                    if not isinstance(words[letter], list):
                        words[letter] = [words[letter]]
                    words[letter].append(value)
                else:
                    words[letter] = value
        else:
            i += 1
    
    return words


def calculate_arc_center(start: Position, end: Position, i: float, j: float, k: float,
                        plane: Plane, use_radius: bool = False, radius: float = 0.0) -> Tuple[Position, float, float]:
    """Calculate arc center, radius, and sweep angle"""
    
    if plane == Plane.XY:
        s1, s2, s3 = start.x, start.y, start.z
        e1, e2, e3 = end.x, end.y, end.z
        o1, o2 = i, j
    elif plane == Plane.XZ:
        s1, s2, s3 = start.x, start.z, start.y
        e1, e2, e3 = end.x, end.z, end.y
        o1, o2 = i, k
    else:  # YZ
        s1, s2, s3 = start.y, start.z, start.x
        e1, e2, e3 = end.y, end.z, end.x
        o1, o2 = j, k
    
    # Center is relative to start point (standard for most GCode)
    c1 = s1 + o1
    c2 = s2 + o2
    
    # Calculate radius
    radius = math.sqrt(o1**2 + o2**2)
    
    # Calculate start and end angles
    start_angle = math.atan2(s2 - c2, s1 - c1)
    end_angle = math.atan2(e2 - c2, e1 - c1)
    
    # Build center position
    center = Position()
    if plane == Plane.XY:
        center.x, center.y, center.z = c1, c2, s3
    elif plane == Plane.XZ:
        center.x, center.z, center.y = c1, c2, s3
    else:
        center.y, center.z, center.x = c1, c2, s3
    
    return center, radius, start_angle, end_angle


def interpolate_segment(segment: MotionSegment, t: float) -> Position:
    """Interpolate position along a segment at parameter t (0 to 1)"""
    
    if segment.motion_type in (MotionType.RAPID, MotionType.LINEAR):
        # Linear interpolation
        return Position(
            x=segment.start.x + t * (segment.end.x - segment.start.x),
            y=segment.start.y + t * (segment.end.y - segment.start.y),
            z=segment.start.z + t * (segment.end.z - segment.start.z),
        )
    else:
        # Arc interpolation
        center = segment.center
        plane = segment.plane
        
        if plane == Plane.XY:
            cx, cy = center.x, center.y
            start_angle = math.atan2(segment.start.y - cy, segment.start.x - cx)
            angle = start_angle + t * segment.arc_sweep
            
            return Position(
                x=cx + segment.arc_radius * math.cos(angle),
                y=cy + segment.arc_radius * math.sin(angle),
                z=segment.start.z + t * (segment.end.z - segment.start.z),
            )
        elif plane == Plane.XZ:
            cx, cz = center.x, center.z
            start_angle = math.atan2(segment.start.z - cz, segment.start.x - cx)
            angle = start_angle + t * segment.arc_sweep
            
            return Position(
                x=cx + segment.arc_radius * math.cos(angle),
                y=segment.start.y + t * (segment.end.y - segment.start.y),
                z=cz + segment.arc_radius * math.sin(angle),
            )
        else:  # YZ
            cy, cz = center.y, center.z
            start_angle = math.atan2(segment.start.z - cz, segment.start.y - cy)
            angle = start_angle + t * segment.arc_sweep
            
            return Position(
                x=segment.start.x + t * (segment.end.x - segment.start.x),
                y=cy + segment.arc_radius * math.cos(angle),
                z=cz + segment.arc_radius * math.sin(angle),
            )


class GCodeInterpreter:
    """Simple GCode interpreter that generates motion segments"""
    
    def __init__(self, rapid_rate: float = 6000.0, default_feed: float = 1000.0):
        self.state = MachineState()
        self.state.rapid_rate = rapid_rate
        self.state.feed_rate = default_feed
        self.segments: List[MotionSegment] = []
    
    def process_line(self, line: str, block_index: int) -> Optional[MotionSegment]:
        """Process a single GCode line and return motion segment if any"""
        words = parse_gcode_line(line)
        if not words:
            return None
        
        # Handle G-codes
        g_codes = words.get('G', [])
        if not isinstance(g_codes, list):
            g_codes = [g_codes]
        
        for g in g_codes:
            g_int = int(g * 10)  # G1 -> 10, G38.2 -> 382
            
            if g_int == 0:  # G0 - rapid
                self.state.motion_mode = MotionType.RAPID
            elif g_int == 10:  # G1 - linear
                self.state.motion_mode = MotionType.LINEAR
            elif g_int == 20:  # G2 - CW arc
                self.state.motion_mode = MotionType.CW_ARC
            elif g_int == 30:  # G3 - CCW arc
                self.state.motion_mode = MotionType.CCW_ARC
            elif g_int == 170:  # G17 - XY plane
                self.state.plane = Plane.XY
            elif g_int == 180:  # G18 - XZ plane
                self.state.plane = Plane.XZ
            elif g_int == 190:  # G19 - YZ plane
                self.state.plane = Plane.YZ
            elif g_int == 900:  # G90 - absolute
                self.state.absolute = True
            elif g_int == 910:  # G91 - incremental
                self.state.absolute = False
        
        # Handle F (feed rate)
        if 'F' in words:
            self.state.feed_rate = words['F']
        
        # Handle motion (X, Y, Z coordinates)
        has_motion = any(c in words for c in 'XYZABC')
        if not has_motion:
            return None
        
        # Calculate target position
        start = Position(
            x=self.state.position.x,
            y=self.state.position.y,
            z=self.state.position.z,
        )
        
        if self.state.absolute:
            end = Position(
                x=words.get('X', start.x),
                y=words.get('Y', start.y),
                z=words.get('Z', start.z),
            )
        else:
            end = Position(
                x=start.x + words.get('X', 0),
                y=start.y + words.get('Y', 0),
                z=start.z + words.get('Z', 0),
            )
        
        # Create segment
        feed_rate = self.state.rapid_rate if self.state.motion_mode == MotionType.RAPID else self.state.feed_rate
        
        segment = MotionSegment(
            start=start,
            end=end,
            center=None,
            feed_rate=feed_rate,
            motion_type=self.state.motion_mode,
            block_index=block_index,
            plane=self.state.plane,
        )
        
        # Handle arc specifics
        if self.state.motion_mode in (MotionType.CW_ARC, MotionType.CCW_ARC):
            i = words.get('I', 0.0)
            j = words.get('J', 0.0)
            k = words.get('K', 0.0)
            r = words.get('R', 0.0)
            
            center, radius, start_angle, end_angle = calculate_arc_center(
                start, end, i, j, k, self.state.plane
            )
            
            segment.center = center
            segment.arc_radius = radius
            
            # Calculate sweep angle
            sweep = end_angle - start_angle
            
            if self.state.motion_mode == MotionType.CW_ARC:
                # Clockwise: negative sweep
                if sweep > 0:
                    sweep -= 2 * math.pi
            else:
                # Counter-clockwise: positive sweep
                if sweep < 0:
                    sweep += 2 * math.pi
            
            segment.arc_sweep = sweep
            segment.segment_length = abs(radius * sweep)
        else:
            # Linear segment length
            dx = end.x - start.x
            dy = end.y - start.y
            dz = end.z - start.z
            segment.segment_length = math.sqrt(dx**2 + dy**2 + dz**2)
        
        # Calculate time
        if segment.segment_length > 0 and feed_rate > 0:
            segment.segment_time = segment.segment_length / (feed_rate / 60.0)  # Convert mm/min to mm/s
        
        # Update state
        self.state.position = end
        self.segments.append(segment)
        
        return segment
    
    def process_gcode(self, gcode: str) -> List[MotionSegment]:
        """Process complete GCode program"""
        self.segments = []
        
        for i, line in enumerate(gcode.strip().split('\n')):
            self.process_line(line, i)
        
        return self.segments


def generate_trajectory(segments: List[MotionSegment], time_resolution: float = 0.01) -> List[TrajectoryPoint]:
    """Generate trajectory points from motion segments"""
    points = []
    current_time = 0.0
    
    for segment in segments:
        if segment.segment_time <= 0:
            # Instantaneous move (shouldn't happen normally)
            pos = segment.end
            vel = Position()
            points.append(TrajectoryPoint(
                time=current_time,
                position=pos,
                velocity=vel,
                motion_type=segment.motion_type,
                block_index=segment.block_index,
            ))
            continue
        
        # Number of points for this segment
        num_points = max(2, int(segment.segment_time / time_resolution))
        
        # For arcs, ensure enough points for smooth curves
        if segment.motion_type in (MotionType.CW_ARC, MotionType.CCW_ARC):
            # At least 8 points per arc, more for larger arcs
            min_arc_points = max(8, int(abs(segment.arc_sweep) / (math.pi / 16)))
            num_points = max(num_points, min_arc_points)
        
        for i in range(num_points + 1):
            t = i / num_points
            point_time = current_time + t * segment.segment_time
            
            pos = interpolate_segment(segment, t)
            
            # Calculate velocity (constant for now)
            if segment.segment_time > 0:
                vx = (segment.end.x - segment.start.x) / segment.segment_time
                vy = (segment.end.y - segment.start.y) / segment.segment_time
                vz = (segment.end.z - segment.start.z) / segment.segment_time
            else:
                vx = vy = vz = 0.0
            
            vel = Position(x=vx, y=vy, z=vz)
            
            points.append(TrajectoryPoint(
                time=point_time,
                position=pos,
                velocity=vel,
                motion_type=segment.motion_type,
                block_index=segment.block_index,
            ))
        
        current_time += segment.segment_time
    
    return points


def output_csv(points: List[TrajectoryPoint]):
    """Output points as CSV"""
    print("time,x,y,z,vx,vy,vz,block_index,motion_type")
    for pt in points:
        print(f"{pt.time:.6f},{pt.position.x:.6f},{pt.position.y:.6f},{pt.position.z:.6f},"
              f"{pt.velocity.x:.6f},{pt.velocity.y:.6f},{pt.velocity.z:.6f},"
              f"{pt.block_index},{pt.motion_type.name}")


def output_json(points: List[TrajectoryPoint]):
    """Output points as JSON"""
    data = {
        "point_count": len(points),
        "duration": points[-1].time if points else 0.0,
        "points": [
            {
                "time": pt.time,
                "position": {"x": pt.position.x, "y": pt.position.y, "z": pt.position.z},
                "velocity": {"x": pt.velocity.x, "y": pt.velocity.y, "z": pt.velocity.z},
                "block_index": pt.block_index,
                "motion_type": pt.motion_type.name,
            }
            for pt in points
        ]
    }
    print(json.dumps(data, indent=2))


def output_simple(points: List[TrajectoryPoint]):
    """Output points as simple XYZ"""
    print(f"# {len(points)} points, duration: {points[-1].time if points else 0:.3f}s")
    for pt in points:
        print(f"{pt.position.x:.6f} {pt.position.y:.6f} {pt.position.z:.6f}")


def compare_csv(points: List[TrajectoryPoint], csv_file: str, tolerance: float = 0.001) -> bool:
    """Compare generated points with a CSV file"""
    import csv
    
    with open(csv_file, 'r') as f:
        reader = csv.DictReader(f)
        csv_points = list(reader)
    
    if len(points) != len(csv_points):
        print(f"MISMATCH: Different point counts: {len(points)} vs {len(csv_points)}")
        return False
    
    max_diff = 0.0
    mismatches = []
    
    for i, (ref, csv_pt) in enumerate(zip(points, csv_points)):
        dx = abs(ref.position.x - float(csv_pt['x']))
        dy = abs(ref.position.y - float(csv_pt['y']))
        dz = abs(ref.position.z - float(csv_pt['z']))
        dt = abs(ref.time - float(csv_pt['time']))
        
        max_diff = max(max_diff, dx, dy, dz, dt)
        
        if dx > tolerance or dy > tolerance or dz > tolerance or dt > tolerance:
            mismatches.append({
                'index': i,
                'ref': {'t': ref.time, 'x': ref.position.x, 'y': ref.position.y, 'z': ref.position.z},
                'csv': {'t': float(csv_pt['time']), 'x': float(csv_pt['x']), 'y': float(csv_pt['y']), 'z': float(csv_pt['z'])},
                'diff': {'t': dt, 'x': dx, 'y': dy, 'z': dz},
            })
    
    if mismatches:
        print(f"MISMATCH: {len(mismatches)} points differ by more than {tolerance}")
        for m in mismatches[:5]:  # Show first 5
            print(f"  Point {m['index']}: ref=({m['ref']['x']:.4f},{m['ref']['y']:.4f},{m['ref']['z']:.4f}) "
                  f"csv=({m['csv']['x']:.4f},{m['csv']['y']:.4f},{m['csv']['z']:.4f}) "
                  f"diff=({m['diff']['x']:.6f},{m['diff']['y']:.6f},{m['diff']['z']:.6f})")
        return False
    
    print(f"OK: All {len(points)} points match within tolerance {tolerance} (max diff: {max_diff:.6f})")
    return True


def main():
    parser = argparse.ArgumentParser(description="GCode reference trajectory generator")
    parser.add_argument('--gcode', '-g', help="GCode to process (use \\\\n for newlines)")
    parser.add_argument('--file', '-f', help="File to read GCode from")
    parser.add_argument('--format', choices=['csv', 'json', 'simple'], default='csv',
                       help="Output format")
    parser.add_argument('--resolution', type=float, default=0.01,
                       help="Time resolution in seconds")
    parser.add_argument('--feed-rate', type=float, default=1000.0,
                       help="Default feed rate (mm/min)")
    parser.add_argument('--rapid-rate', type=float, default=6000.0,
                       help="Rapid feed rate (mm/min)")
    parser.add_argument('--compare', '-c', help="CSV file to compare against")
    parser.add_argument('--tolerance', type=float, default=0.001,
                       help="Comparison tolerance")
    
    args = parser.parse_args()
    
    # Get GCode content
    if args.gcode:
        gcode = args.gcode.replace('\\n', '\n')
    elif args.file:
        with open(args.file, 'r') as f:
            gcode = f.read()
    else:
        print("Error: Either --gcode or --file must be specified", file=sys.stderr)
        sys.exit(1)
    
    # Process GCode
    interpreter = GCodeInterpreter(
        rapid_rate=args.rapid_rate,
        default_feed=args.feed_rate,
    )
    segments = interpreter.process_gcode(gcode)
    points = generate_trajectory(segments, args.resolution)
    
    # Output or compare
    if args.compare:
        success = compare_csv(points, args.compare, args.tolerance)
        sys.exit(0 if success else 1)
    else:
        if args.format == 'csv':
            output_csv(points)
        elif args.format == 'json':
            output_json(points)
        else:
            output_simple(points)


if __name__ == '__main__':
    main()
