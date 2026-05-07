use std::time::Instant;
use std::path::Path;
use std::fs;
use std::error::Error;

use crate::gcode::{Parser, Interpreter, MotionSegment};

/// Results for a single test entry
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub duration_ms: f64,
    pub points: usize,
}

/// Summary of the benchmark run
#[derive(Debug, Clone, Default)]
pub struct BenchmarkResult {
    pub file_name: String,
    pub file_size: usize,
    pub line_count: usize,
    pub file_read_ms: f64,
    pub parse_ms: f64,
    pub interp_ms: f64,
    pub copy_segments_ms: f64,
    pub path_length_mm: f64,
    pub machining_time_s: f64,
    pub time_tests: Vec<TestResult>,
    pub deviation_tests: Vec<TestResult>,
    pub total_ms: f64,
    pub lines_per_second: f64,
    pub mb_per_sec: f64,
}

// Helper types and conversion
#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionType { Rapid, Linear, ArcCW, ArcCCW, Spline }
impl From<u8> for MotionType { fn from(v: u8) -> Self { match v { 0=>MotionType::Rapid,1=>MotionType::Linear,2=>MotionType::ArcCW,3=>MotionType::ArcCCW,4=>MotionType::Spline,_=>MotionType::Linear } } }

#[derive(Debug, Clone)]
struct Segment {
    start: [f64;9],
    end: [f64;9],
    center: [f64;9],
    arc_radius: f64,
    arc_sweep: f64,
    segment_length: f64,
    segment_time: f64,
    motion_type: MotionType,
}

impl From<&FfiMotionSegment> for Segment {
    fn from(seg: &FfiMotionSegment) -> Self {
        Self {
            start: [seg.start.x, seg.start.y, seg.start.z, seg.start.a, seg.start.b, seg.start.c, seg.start.u, seg.start.v, seg.start.w],
            end: [seg.end.x, seg.end.y, seg.end.z, seg.end.a, seg.end.b, seg.end.c, seg.end.u, seg.end.v, seg.end.w],
            center: [seg.center.x, seg.center.y, seg.center.z, seg.center.a, seg.center.b, seg.center.c, seg.center.u, seg.center.v, seg.center.w],
            arc_radius: seg.arc_radius,
            arc_sweep: seg.arc_sweep,
            segment_length: seg.segment_length,
            segment_time: seg.segment_time,
            motion_type: MotionType::from(seg.motion_type),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TrajectoryStats { point_count: usize, total_time: f64, total_length: f64 }

fn benchmark_trajectory_time_step(segments: &[Segment], time_step_s: f64) -> (TrajectoryStats, std::time::Duration) {
    let start = Instant::now();
    let mut stats = TrajectoryStats::default();
    let mut last = (0.0f64, 0.0f64, 0.0f64);
    let mut has_last = false;

    for seg in segments.iter() {
        let seg_time = seg.segment_time;
        if seg_time <= 0.0 { continue }
        let steps = ((seg_time / time_step_s).ceil() as usize).max(1);
        let inv_steps = 1.0 / steps as f64;
        let is_linear = matches!(seg.motion_type, MotionType::Rapid|MotionType::Linear|MotionType::Spline);
        let (s0,s1,s2) = (seg.start[0], seg.start[1], seg.start[2]);
        let (d0,d1,d2) = (seg.end[0]-s0, seg.end[1]-s1, seg.end[2]-s2);

        if is_linear {
            for j in 0..=steps {
                let t = j as f64 * inv_steps;
                let x = s0 + t*d0; let y = s1 + t*d1; let z = s2 + t*d2;
                if has_last { let dx = x-last.0; let dy = y-last.1; let dz = z-last.2; stats.total_length += (dx*dx+dy*dy+dz*dz).sqrt(); }
                last = (x,y,z); has_last=true; stats.point_count += 1;
            }
        } else {
            let (cx, cy) = (seg.center[0], seg.center[1]);
            let start_angle = (s1 - cy).atan2(s0 - cx);
            let radius = seg.arc_radius; let sweep = seg.arc_sweep;
            for j in 0..=steps {
                let t = j as f64 * inv_steps; let angle = start_angle + t*sweep; let (sin_a, cos_a) = angle.sin_cos();
                let x = cx + radius * cos_a; let y = cy + radius * sin_a; let z = s2 + t*d2;
                if has_last { let dx = x-last.0; let dy = y-last.1; let dz = z-last.2; stats.total_length += (dx*dx+dy*dy+dz*dz).sqrt(); }
                last = (x,y,z); has_last=true; stats.point_count += 1;
            }
        }
        stats.total_time += seg_time;
    }
    (stats, start.elapsed())
}

fn benchmark_trajectory_deviation(segments: &[Segment], max_deviation_mm: f64) -> (TrajectoryStats, std::time::Duration) {
    let start = Instant::now();
    let mut stats = TrajectoryStats::default();
    let mut last = (0.0f64,0.0f64,0.0f64);
    let mut has_last = false;

    for seg in segments.iter() {
        let seg_length = seg.segment_length; if seg_length <= 0.0 { continue }
        let is_linear = matches!(seg.motion_type, MotionType::Rapid|MotionType::Linear|MotionType::Spline);
        let steps = if is_linear { ((seg_length / max_deviation_mm).ceil() as usize).max(1) } else {
            let radius = seg.arc_radius; if radius>0.0 && max_deviation_mm>0.0 { let ratio = max_deviation_mm / radius; if ratio < 1.0 { let angle_per_step = 2.0 * (1.0 - ratio).acos(); if angle_per_step > 0.0 { (seg.arc_sweep.abs() / angle_per_step).ceil() as usize } else { 4 } } else { 4 } } else { 4 }
        }.min(100000).max(if is_linear {1} else {4});
        let inv_steps = 1.0 / steps as f64;
        let (s0,s1,s2) = (seg.start[0],seg.start[1],seg.start[2]); let (d0,d1,d2) = (seg.end[0]-s0, seg.end[1]-s1, seg.end[2]-s2);
        if is_linear {
            for j in 0..=steps { let t = j as f64 * inv_steps; let x = s0 + t*d0; let y = s1 + t*d1; let z = s2 + t*d2; if has_last { let dx=x-last.0; let dy=y-last.1; let dz=z-last.2; stats.total_length += (dx*dx+dy*dy+dz*dz).sqrt(); } last=(x,y,z); has_last=true; stats.point_count +=1 }
        } else {
            let (cx,cy) = (seg.center[0], seg.center[1]); let start_angle = (s1-cy).atan2(s0-cx); let radius = seg.arc_radius; let sweep = seg.arc_sweep; for j in 0..=steps { let t = j as f64 * inv_steps; let angle=start_angle + t*sweep; let (sin_a,cos_a) = angle.sin_cos(); let x = cx + radius * cos_a; let y = cy + radius * sin_a; let z = s2 + t*d2; if has_last { let dx=x-last.0; let dy=y-last.1; let dz=z-last.2; stats.total_length += (dx*dx+dy*dy+dz*dz).sqrt(); } last=(x,y,z); has_last=true; stats.point_count +=1 }
        }
        stats.total_time += seg.segment_time;
    }
    (stats, start.elapsed())
}

/// Run the full benchmark from a gcode file path
pub fn run_benchmark_from_file<P: AsRef<Path>>(path: P) -> Result<BenchmarkResult, Box<dyn Error>> {
    let file = path.as_ref();
    let filename = file.display().to_string();
    let total_start = Instant::now();

    // Read file
    let file_start = Instant::now();
    let content = fs::read_to_string(file)?;
    let file_read = file_start.elapsed();

    let file_size = content.len();
    let line_count = content.lines().count();

    // Parse
    let parse_start = Instant::now();
    let mut parser = Parser::new()?;
    parser.parse_string(&content)?;
    let parse_time = parse_start.elapsed();
    let block_count = parser.block_count();
    drop(content);

    // Interpret
    let interp_start = Instant::now();
    let mut interpreter = Interpreter::new()?;
    interpreter.configure(6000.0, 1000.0, 10000.0);
    interpreter.load_blocks(&parser)?;
    let interp_time = interp_start.elapsed();
    let segment_count = interpreter.segment_count();

    // Copy segments
    let seg_start = Instant::now();
    let ffi_segments = interpreter.get_all_segments();
    let segments: Vec<Segment> = ffi_segments.iter().map(Segment::from).collect();
    let seg_time = seg_start.elapsed();

    let mut path_length = 0.0f64;
    let mut machining_time = 0.0f64;
    for seg in &segments { path_length += seg.segment_length; machining_time += seg.segment_time; }

    // Time step tests
    let time_tests_spec = [ ("5ms", 0.005), ("1ms", 0.001), ("0.5ms", 0.0005) ];
    let mut time_results = Vec::new();
    for (name,step) in time_tests_spec.iter() {
        let (stats, dur) = benchmark_trajectory_time_step(&segments, *step);
        time_results.push(TestResult { name: name.to_string(), duration_ms: dur.as_secs_f64()*1000.0, points: stats.point_count });
    }

    // Deviation tests
    let dev_tests_spec = [ ("10μm", 0.010), ("5μm", 0.005), ("1μm", 0.001) ];
    let mut dev_results = Vec::new();
    for (name,dev) in dev_tests_spec.iter() {
        let (stats, dur) = benchmark_trajectory_deviation(&segments, *dev);
        dev_results.push(TestResult { name: name.to_string(), duration_ms: dur.as_secs_f64()*1000.0, points: stats.point_count });
    }

    let total_time = total_start.elapsed();
    let throughput = line_count as f64 / total_time.as_secs_f64();
    let mb_per_sec = (file_size as f64 / (1024.0*1024.0)) / total_time.as_secs_f64();

    Ok(BenchmarkResult {
        file_name: filename,
        file_size,
        line_count,
        file_read_ms: file_read.as_secs_f64() * 1000.0,
        parse_ms: parse_time.as_secs_f64() * 1000.0,
        interp_ms: interp_time.as_secs_f64() * 1000.0,
        copy_segments_ms: seg_time.as_secs_f64() * 1000.0,
        path_length_mm: path_length,
        machining_time_s: machining_time,
        time_tests: time_results,
        deviation_tests: dev_results,
        total_ms: total_time.as_secs_f64() * 1000.0,
        lines_per_second: throughput,
        mb_per_sec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn run_benchmark_small_file() {
        // Create a small temporary GCode file
        let tmp = std::env::temp_dir().join("gcbm_test_small.gcode");
        let mut f = fs::File::create(&tmp).expect("create temp");
        write!(f, "G21\nG90\nG0 X0 Y0\nG1 X10 Y0 F1000\nG1 X10 Y10\n").unwrap();
        drop(f);

        let res = run_benchmark_from_file(&tmp).expect("benchmark run");
        assert!(res.line_count >= 5);
        assert!(res.file_size > 0);
        assert!(!res.time_tests.is_empty());
        assert!(!res.deviation_tests.is_empty());

        // cleanup
        let _ = fs::remove_file(&tmp);
    }
}

