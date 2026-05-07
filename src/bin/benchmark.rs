//! G-Code Performance Benchmark Tool
//!
//! Benchmarks G-code parsing and trajectory generation in Rust.
//! Memory-efficient streaming computation with O(1) memory for trajectory stats.

use std::env;
use std::time::Instant;

// Use the cxx-based FFI module from the parent library crate
use gcodeflow::gcode::{Parser, Interpreter, TrajectoryGenerator};

#[cfg(tether_ffi)]
use gcodeflow::gcode::FfiMotionSegment;

/// Memory limit: 6GB
const MEMORY_LIMIT_BYTES: u64 = 6 * 1024 * 1024 * 1024;

/// Format bytes for human readability
fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Format memory size
fn format_memory(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn print_separator() {
    println!("{}", "-".repeat(70));
}

/// Trajectory statistics (computed without storing all points)
#[derive(Debug, Clone, Default)]
struct TrajectoryStats {
    point_count: usize,
    total_time: f64,
    total_length: f64,
}

/// Motion types
#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionType {
    Rapid,
    Linear,
    ArcCW,
    ArcCCW,
    Spline,
}

impl From<u8> for MotionType {
    fn from(v: u8) -> Self {
        match v {
            0 => MotionType::Rapid,
            1 => MotionType::Linear,
            2 => MotionType::ArcCW,
            3 => MotionType::ArcCCW,
            4 => MotionType::Spline,
            _ => MotionType::Linear,
        }
    }
}

/// Safe wrapper for a motion segment (converted from FFI)
struct Segment {
    start: [f64; 9],
    end: [f64; 9],
    center: [f64; 9],
    arc_radius: f64,
    arc_sweep: f64,
    segment_length: f64,
    segment_time: f64,
    motion_type: MotionType,
}

#[cfg(tether_ffi)]
impl From<&FfiMotionSegment> for Segment {
    fn from(seg: &FfiMotionSegment) -> Self {
        Self {
            start: [seg.start.x, seg.start.y, seg.start.z, 
                    seg.start.a, seg.start.b, seg.start.c,
                    seg.start.u, seg.start.v, seg.start.w],
            end: [seg.end.x, seg.end.y, seg.end.z,
                  seg.end.a, seg.end.b, seg.end.c,
                  seg.end.u, seg.end.v, seg.end.w],
            center: [seg.center.x, seg.center.y, seg.center.z,
                     seg.center.a, seg.center.b, seg.center.c,
                     seg.center.u, seg.center.v, seg.center.w],
            arc_radius: seg.arc_radius,
            arc_sweep: seg.arc_sweep,
            segment_length: seg.segment_length,
            segment_time: seg.segment_time,
            motion_type: MotionType::from(seg.motion_type),
        }
    }
}

/// Stream-compute trajectory statistics with fixed time steps
/// O(1) memory - only tracks running statistics
fn benchmark_trajectory_time_step(
    segments: &[Segment],
    time_step_s: f64,
) -> (TrajectoryStats, std::time::Duration) {
    let start_time = Instant::now();
    
    let mut stats = TrajectoryStats::default();
    let mut last_x = 0.0f64;
    let mut last_y = 0.0f64;
    let mut last_z = 0.0f64;
    let mut has_last = false;
    
    for seg in segments.iter() {
        let seg_time = seg.segment_time;
        if seg_time <= 0.0 {
            continue;
        }
        
        let steps = ((seg_time / time_step_s).ceil() as usize).max(1);
        let inv_steps = 1.0 / steps as f64;
        let is_linear = matches!(seg.motion_type, MotionType::Rapid | MotionType::Linear | MotionType::Spline);
        
        // Pre-fetch segment data for linear interpolation
        let (s0, s1, s2) = (seg.start[0], seg.start[1], seg.start[2]);
        let (d0, d1, d2) = (seg.end[0] - s0, seg.end[1] - s1, seg.end[2] - s2);
        
        if is_linear {
            for j in 0..=steps {
                let t = j as f64 * inv_steps;
                
                let x = s0 + t * d0;
                let y = s1 + t * d1;
                let z = s2 + t * d2;
                
                if has_last {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    let dz = z - last_z;
                    stats.total_length += (dx*dx + dy*dy + dz*dz).sqrt();
                }
                last_x = x;
                last_y = y;
                last_z = z;
                has_last = true;
                
                stats.point_count += 1;
            }
        } else {
            // Arc interpolation
            let (cx, cy) = (seg.center[0], seg.center[1]);
            let start_angle = (s1 - cy).atan2(s0 - cx);
            let radius = seg.arc_radius;
            let sweep = seg.arc_sweep;
            
            for j in 0..=steps {
                let t = j as f64 * inv_steps;
                let angle = start_angle + t * sweep;
                let (sin_a, cos_a) = angle.sin_cos();
                
                let x = cx + radius * cos_a;
                let y = cy + radius * sin_a;
                let z = s2 + t * d2;
                
                if has_last {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    let dz = z - last_z;
                    stats.total_length += (dx*dx + dy*dy + dz*dz).sqrt();
                }
                last_x = x;
                last_y = y;
                last_z = z;
                has_last = true;
                
                stats.point_count += 1;
            }
        }
        
        stats.total_time += seg_time;
    }
    
    (stats, start_time.elapsed())
}

/// Stream-compute trajectory statistics with fixed deviation tolerance
/// O(1) memory - only tracks running statistics
fn benchmark_trajectory_deviation(
    segments: &[Segment],
    max_deviation_mm: f64,
) -> (TrajectoryStats, std::time::Duration) {
    let start_time = Instant::now();
    
    let mut stats = TrajectoryStats::default();
    let mut last_x = 0.0f64;
    let mut last_y = 0.0f64;
    let mut last_z = 0.0f64;
    let mut has_last = false;
    
    for seg in segments.iter() {
        let seg_length = seg.segment_length;
        if seg_length <= 0.0 {
            continue;
        }
        
        let is_linear = matches!(seg.motion_type, MotionType::Rapid | MotionType::Linear | MotionType::Spline);
        
        let steps = if is_linear {
            ((seg_length / max_deviation_mm).ceil() as usize).max(1)
        } else {
            let radius = seg.arc_radius;
            if radius > 0.0 && max_deviation_mm > 0.0 {
                let ratio = max_deviation_mm / radius;
                if ratio < 1.0 {
                    let angle_per_step = 2.0 * (1.0 - ratio).acos();
                    if angle_per_step > 0.0 {
                        (seg.arc_sweep.abs() / angle_per_step).ceil() as usize
                    } else {
                        4
                    }
                } else {
                    4
                }
            } else {
                4
            }
        }.min(100000).max(if is_linear { 1 } else { 4 });
        
        let inv_steps = 1.0 / steps as f64;
        
        let (s0, s1, s2) = (seg.start[0], seg.start[1], seg.start[2]);
        let (d0, d1, d2) = (seg.end[0] - s0, seg.end[1] - s1, seg.end[2] - s2);
        
        if is_linear {
            for j in 0..=steps {
                let t = j as f64 * inv_steps;
                
                let x = s0 + t * d0;
                let y = s1 + t * d1;
                let z = s2 + t * d2;
                
                if has_last {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    let dz = z - last_z;
                    stats.total_length += (dx*dx + dy*dy + dz*dz).sqrt();
                }
                last_x = x;
                last_y = y;
                last_z = z;
                has_last = true;
                
                stats.point_count += 1;
            }
        } else {
            let (cx, cy) = (seg.center[0], seg.center[1]);
            let start_angle = (s1 - cy).atan2(s0 - cx);
            let radius = seg.arc_radius;
            let sweep = seg.arc_sweep;
            
            for j in 0..=steps {
                let t = j as f64 * inv_steps;
                let angle = start_angle + t * sweep;
                let (sin_a, cos_a) = angle.sin_cos();
                
                let x = cx + radius * cos_a;
                let y = cy + radius * sin_a;
                let z = s2 + t * d2;
                
                if has_last {
                    let dx = x - last_x;
                    let dy = y - last_y;
                    let dz = z - last_z;
                    stats.total_length += (dx*dx + dy*dy + dz*dz).sqrt();
                }
                last_x = x;
                last_y = y;
                last_z = z;
                has_last = true;
                
                stats.point_count += 1;
            }
        }
        
        stats.total_time += seg.segment_time;
    }
    
    (stats, start_time.elapsed())
}

fn main() {
    // Reuse the library/utility function to run a benchmark and print results
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <gcode_file>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    match gcodeflow::benchmark::run_benchmark_from_file(filename) {
        Ok(res) => {
            println!("\n══════════════════════════════════════════════════════════════");
            println!("              G-CODE PERFORMANCE BENCHMARK (RUST/CXX)");
            println!("══════════════════════════════════════════════════════════════\n");

            println!("File: {}", res.file_name);
            println!("Size: {} ({} lines)", format_bytes(res.file_size), res.line_count);
            println!("1. File I/O (read into memory)             {:>10.2} ms", res.file_read_ms);
            println!("2. Parse G-code to blocks                  {:>10.2} ms", res.parse_ms);
            println!("3. Interpretation and segment copy         {:>10.2} ms", res.interp_ms + res.copy_segments_ms);
            println!("   Path length:    {:.2} mm", res.path_length_mm);
            println!("   Machining time: {:.2} s", res.machining_time_s);

            println!("\nTime tests:");
            for t in res.time_tests.iter() {
                println!("  {:<8} {:>10.2} ms  ({} points)", t.name, t.duration_ms, t.points);
            }

            println!("\nDeviation tests:");
            for t in res.deviation_tests.iter() {
                println!("  {:<8} {:>10.2} ms  ({} points)", t.name, t.duration_ms, t.points);
            }

            println!("\nTotal benchmark time                       {:>10.2} ms", res.total_ms);
            println!("Throughput: {:.0} lines/sec  ({:.2} MB/sec)", res.lines_per_second, res.mb_per_sec);
            println!("\n══════════════════════════════════════════════════════════════");
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
            std::process::exit(1);
        }
    }
}


