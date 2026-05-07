//! GCodeFlow - GCode Visualization and Simulation Tool
//!
//! A comprehensive GCode visualization application built with Bevy and egui,
//! featuring 3D accelerated rendering, syntax highlighting, and simulation capabilities.

mod app;
mod camera;
mod config;
mod editor;
mod kinematics;
mod plot_view;
mod native_plot_window;
mod benchmark_ui;
mod rendering;
mod simulation;
mod trajectory;
mod ui;

// Use gcode module from the library crate to get the linked FFI bindings
use gcodeflow::gcode;

use anyhow::Result;
use bevy::prelude::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// GCodeFlow - GCode Visualization and Simulation Tool
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input GCode file to load
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Output screenshot path (renders and exits)
    #[arg(short, long)]
    pub screenshot: Option<PathBuf>,

    /// Camera angle preset: "top", "front", "side", "iso", "custom"
    #[arg(long, default_value = "iso")]
    pub camera_angle: String,

    /// Custom camera position (x,y,z) when camera_angle is "custom"
    #[arg(long)]
    pub camera_pos: Option<String>,

    /// Custom camera target (x,y,z) when camera_angle is "custom"
    #[arg(long)]
    pub camera_target: Option<String>,

    /// Window width
    #[arg(long, default_value = "1920")]
    pub width: u32,

    /// Window height
    #[arg(long, default_value = "1080")]
    pub height: u32,

    /// Headless mode (no window, for CI/testing)
    #[arg(long)]
    pub headless: bool,

    /// Disable autosave
    #[arg(long)]
    pub no_autosave: bool,

    /// Configuration file path
    #[arg(long, default_value = "gcodeflow.yaml")]
    pub config: PathBuf,

    /// Verbosity level (0-3)
    #[arg(short, long, default_value = "1")]
    pub verbose: u8,
    
    /// Subcommand for CLI tools
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// CLI subcommands for testing and utilities
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Display syntax-highlighted GCode
    Highlight {
        /// GCode to highlight (can be multiline with \n)
        #[arg(short, long)]
        gcode: Option<String>,
        
        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Output format: "ansi" (terminal colors), "html", "json"
        #[arg(long, default_value = "ansi")]
        format: String,
    },
    
    /// Generate trajectory points from GCode
    Points {
        /// GCode to process (can be multiline with \n)
        #[arg(short, long)]
        gcode: Option<String>,
        
        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Time resolution for trajectory sampling (seconds)
        #[arg(long, default_value = "0.01")]
        resolution: f64,
        
        /// Output format: "csv", "json", "simple"
        #[arg(long, default_value = "csv")]
        format: String,
        
        /// Feed rate for moves (mm/min)
        #[arg(long, default_value = "1000")]
        feed_rate: f64,
    },
    
    /// Parse GCode and show block information
    Parse {
        /// GCode to parse
        #[arg(short, long)]
        gcode: Option<String>,
        
        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    
    /// Debug visualization data - show what traces would be rendered
    Debug {
        /// GCode to debug
        #[arg(short, long)]
        gcode: Option<String>,
        
        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Time resolution for trajectory sampling (seconds)
        #[arg(long, default_value = "0.01")]
        resolution: f64,
        
        /// Feed rate for moves (mm/min)
        #[arg(long, default_value = "1000")]
        feed_rate: f64,
        
        /// Show verbose segment info
        #[arg(long)]
        verbose: bool,
    },

    /// Emulate movement and print sampled tool poses with timestamps (no waiting)
    Emulate {
        /// GCode to emulate
        #[arg(short, long)]
        gcode: Option<String>,

        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Time step resolution for sampling (seconds)
        #[arg(long, default_value = "0.01")]
        resolution: f64,
    },
    
    /// Verify trajectory generation for integration testing
    Verify {
        /// GCode to verify
        #[arg(short, long)]
        gcode: Option<String>,
        
        /// File to read GCode from
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Expected number of trajectory points (fail if different)
        #[arg(long)]
        expect_points: Option<usize>,
        
        /// Expected final X position (fail if different, tolerance 0.001)
        #[arg(long)]
        expect_x: Option<f64>,
        
        /// Expected final Y position (fail if different, tolerance 0.001)
        #[arg(long)]
        expect_y: Option<f64>,
        
        /// Expected final Z position (fail if different, tolerance 0.001)
        #[arg(long)]
        expect_z: Option<f64>,
        
        /// Expected total duration in seconds (fail if different, tolerance 0.01)
        #[arg(long)]
        expect_duration: Option<f64>,
        
        /// Time resolution for trajectory sampling (seconds)
        #[arg(long, default_value = "0.01")]
        resolution: f64,
        
        /// Feed rate for moves (mm/min)
        #[arg(long, default_value = "1000")]
        feed_rate: f64,
        
        /// Output format: "summary", "json", "verbose"
        #[arg(long, default_value = "summary")]
        format: String,
    },
}

impl Default for Args {
    fn default() -> Self {
        Self {
            input: None,
            screenshot: None,
            camera_angle: "iso".to_string(),
            camera_pos: None,
            camera_target: None,
            width: 1920,
            height: 1080,
            headless: false,
            no_autosave: false,
            config: PathBuf::from("gcodeflow.yaml"),
            verbose: 1,
            command: None,
        }
    }
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let log_level = match args.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    // Handle CLI subcommands
    if let Some(ref cmd) = args.command {
        return handle_cli_command(cmd);
    }

    log::info!("Starting GCodeFlow v{}", env!("CARGO_PKG_VERSION"));

    // Run the application
    app::run(args)
}

/// Handle CLI subcommands without launching the GUI
fn handle_cli_command(cmd: &Command) -> Result<()> {
    match cmd {
        Command::Highlight { gcode, file, format } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_highlighted(&content, format)?;
        }
        Command::Points { gcode, file, resolution, format, feed_rate } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_points(&content, *resolution, format, *feed_rate)?;
        }
        Command::Parse { gcode, file } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_parsed(&content)?;
        }
        Command::Debug { gcode, file, resolution, feed_rate, verbose } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_debug(&content, *resolution, *feed_rate, *verbose)?;
        }
        Command::Emulate { gcode, file, resolution } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_emulate(&content, *resolution)?;
        }
        Command::Verify { gcode, file, expect_points, expect_x, expect_y, expect_z, expect_duration, resolution, feed_rate, format } => {
            let content = get_gcode_content(gcode.as_deref(), file.as_ref())?;
            output_verify(&content, *resolution, *feed_rate, expect_points.as_ref(), expect_x.as_ref(), expect_y.as_ref(), expect_z.as_ref(), expect_duration.as_ref(), format)?;
        }
    }
    Ok(())
}

/// Output emulation of movement: timestamped tool poses at fixed resolution
fn output_emulate(content: &str, resolution: f64) -> Result<()> {
    use gcode::{TrajectoryGenerator, MachineConfig, KinematicsType};

    let config = MachineConfig {
        min_x: -500.0,
        max_x: 500.0,
        min_y: -500.0,
        max_y: 500.0,
        min_z: 0.0,
        max_z: 400.0,
        min_a: -360.0,
        max_a: 360.0,
        min_b: -360.0,
        max_b: 360.0,
        min_c: -360.0,
        max_c: 360.0,
        max_velocity_linear: 2000.0,
        max_velocity_angular: 3600.0,
        max_acceleration: 3000.0,
        max_jerk: 10000.0,
        default_feed_rate: 1000.0,
        rapid_feed_rate: 2000.0,
        use_metric: true,
        kinematics_type: KinematicsType::Cartesian,
    };

    let mut generator = TrajectoryGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create generator: {:?}", e))?;

    let points = generator.generate_from_gcode(
        content,
        config.max_velocity_linear,
        config.max_acceleration,
        config.max_jerk,
        resolution,
    )
        .map_err(|e| anyhow::anyhow!("Failed to generate trajectory: {:?}", e))?;

    if points.is_empty() {
        println!("No trajectory points generated");
        return Ok(());
    }

    let duration = points.last().map(|p| p.time).unwrap_or(0.0);
    let mut t = 0.0_f64;

    println!("# Emulation steps (resolution = {}s):", resolution);
    println!("# time x y z feed_rate_mm_per_min");

    while t <= duration + 1e-12 {
        // Find bracketing points
        let idx = points.partition_point(|p| p.time <= t);
        let (pos, feed_rate) = if idx == 0 {
            let p = &points[0];
            ((p.position.x, p.position.y, p.position.z), (p.velocity.x.hypot(p.velocity.y).hypot(p.velocity.z) * 60.0))
        } else if idx >= points.len() {
            let p = points.last().unwrap();
            ((p.position.x, p.position.y, p.position.z), (p.velocity.x.hypot(p.velocity.y).hypot(p.velocity.z) * 60.0))
        } else {
            let p0 = &points[idx - 1];
            let p1 = &points[idx];
            let dt = p1.time - p0.time;
            let frac = if dt.abs() < 1e-12 { 0.0 } else { (t - p0.time) / dt };
            let x = p0.position.x + (p1.position.x - p0.position.x) * frac;
            let y = p0.position.y + (p1.position.y - p0.position.y) * frac;
            let z = p0.position.z + (p1.position.z - p0.position.z) * frac;
            let vx = p0.velocity.x + (p1.velocity.x - p0.velocity.x) * frac;
            let vy = p0.velocity.y + (p1.velocity.y - p0.velocity.y) * frac;
            let vz = p0.velocity.z + (p1.velocity.z - p0.velocity.z) * frac;
            ((x, y, z), (vx.hypot(vy).hypot(vz) * 60.0))
        };

        println!("{:.6} {:.6} {:.6} {:.6} {:.3}", t, pos.0, pos.1, pos.2, feed_rate);
        t += resolution;
    }

    Ok(())
}

/// Get GCode content from either direct string or file
fn get_gcode_content(gcode: Option<&str>, file: Option<&PathBuf>) -> Result<String> {
    if let Some(g) = gcode {
        // Handle escape sequences in the string
        Ok(g.replace("\\n", "\n"))
    } else if let Some(f) = file {
        Ok(std::fs::read_to_string(f)?)
    } else {
        anyhow::bail!("Either --gcode or --file must be specified")
    }
}

/// Output syntax-highlighted GCode
fn output_highlighted(content: &str, format: &str) -> Result<()> {
    use gcode::{tokenize_line, TokenType};
    
    for line in content.lines() {
        let tokens = tokenize_line(line);
        
        match format {
            "ansi" => {
                let mut pos = 0;
                for token in &tokens {
                    // Output text before token
                    if token.start > pos {
                        print!("{}", &line[pos..token.start]);
                    }
                    
                    // ANSI color codes
                    let color_code = match token.token_type {
                        TokenType::GCode => "\x1b[38;5;214m",     // Orange
                        TokenType::MCode => "\x1b[38;5;207m",     // Pink
                        TokenType::Axis => "\x1b[38;5;117m",      // Light blue
                        TokenType::Parameter => "\x1b[38;5;147m", // Light purple
                        TokenType::Number => "\x1b[38;5;156m",    // Light green
                        TokenType::Comment => "\x1b[38;5;102m",   // Gray
                        TokenType::OCode => "\x1b[38;5;220m",     // Yellow
                        TokenType::Operator => "\x1b[38;5;255m",  // White
                        TokenType::Variable => "\x1b[38;5;159m",  // Cyan
                        TokenType::Error => "\x1b[38;5;196m",     // Red
                        TokenType::Unknown => "\x1b[0m",          // Reset
                    };
                    
                    let token_text = &line[token.start..token.start + token.length];
                    print!("{}{}\x1b[0m", color_code, token_text);
                    
                    pos = token.start + token.length;
                }
                // Output remaining text
                if pos < line.len() {
                    print!("{}", &line[pos..]);
                }
                println!();
            }
            "html" => {
                print!("<pre><code>");
                let mut pos = 0;
                for token in &tokens {
                    if token.start > pos {
                        print!("{}", html_escape(&line[pos..token.start]));
                    }
                    
                    let class = match token.token_type {
                        TokenType::GCode => "gcode",
                        TokenType::MCode => "mcode",
                        TokenType::Axis => "axis",
                        TokenType::Parameter => "param",
                        TokenType::Number => "number",
                        TokenType::Comment => "comment",
                        TokenType::OCode => "ocode",
                        TokenType::Operator => "operator",
                        TokenType::Variable => "variable",
                        TokenType::Error => "error",
                        TokenType::Unknown => "unknown",
                    };
                    
                    let token_text = &line[token.start..token.start + token.length];
                    print!("<span class=\"{}\">{}</span>", class, html_escape(token_text));
                    
                    pos = token.start + token.length;
                }
                if pos < line.len() {
                    print!("{}", html_escape(&line[pos..]));
                }
                println!("</code></pre>");
            }
            "json" => {
                let tokens_json: Vec<_> = tokens.iter().map(|t| {
                    let token_text = &line[t.start..t.start + t.length];
                    serde_json::json!({
                        "start": t.start,
                        "length": t.length,
                        "type": format!("{:?}", t.token_type),
                        "text": token_text,
                    })
                }).collect();
                
                println!("{}", serde_json::json!({
                    "line": line,
                    "tokens": tokens_json,
                }));
            }
            _ => {
                anyhow::bail!("Unknown format: {}. Use 'ansi', 'html', or 'json'", format);
            }
        }
    }
    
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

/// Output trajectory points
fn output_points(content: &str, resolution: f64, format: &str, feed_rate: f64) -> Result<()> {
    use gcode::{TrajectoryGenerator, MachineConfig, KinematicsType};
    
    let config = MachineConfig {
        min_x: -500.0,
        max_x: 500.0,
        min_y: -500.0,
        max_y: 500.0,
        min_z: 0.0,
        max_z: 400.0,
        min_a: -360.0,
        max_a: 360.0,
        min_b: -360.0,
        max_b: 360.0,
        min_c: -360.0,
        max_c: 360.0,
        max_velocity_linear: feed_rate * 2.0,
        max_velocity_angular: 3600.0,
        max_acceleration: 3000.0,
        max_jerk: 10000.0,
        default_feed_rate: feed_rate,
        rapid_feed_rate: feed_rate * 2.0,
        use_metric: true,
        kinematics_type: KinematicsType::Cartesian,
    };
    
    let mut generator = TrajectoryGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create generator: {:?}", e))?;
    
    let points = generator.generate_from_gcode(
        content,
        config.max_velocity_linear,
        config.max_acceleration,
        config.max_jerk,
        resolution,
    )
        .map_err(|e| anyhow::anyhow!("Failed to generate trajectory: {:?}", e))?;
    
    match format {
        "csv" => {
            println!("time,x,y,z,vx,vy,vz,block_index,motion_type");
            for pt in &points {
                println!("{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:?}",
                    pt.time,
                    pt.position.x, pt.position.y, pt.position.z,
                    pt.velocity.x, pt.velocity.y, pt.velocity.z,
                    pt.block_index,
                    pt.motion_type,
                );
            }
        }
        "json" => {
            let pts: Vec<_> = points.iter().map(|pt| {
                serde_json::json!({
                    "time": pt.time,
                    "position": {"x": pt.position.x, "y": pt.position.y, "z": pt.position.z},
                    "velocity": {"x": pt.velocity.x, "y": pt.velocity.y, "z": pt.velocity.z},
                    "block_index": pt.block_index,
                    "motion_type": format!("{:?}", pt.motion_type),
                })
            }).collect();
            
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "point_count": points.len(),
                "duration": points.last().map(|p| p.time).unwrap_or(0.0),
                "points": pts,
            }))?);
        }
        "simple" => {
            println!("# {} points, duration: {:.3}s", 
                points.len(),
                points.last().map(|p| p.time).unwrap_or(0.0));
            for pt in &points {
                println!("{:.6} {:.6} {:.6}", pt.position.x, pt.position.y, pt.position.z);
            }
        }
        _ => {
            anyhow::bail!("Unknown format: {}. Use 'csv', 'json', or 'simple'", format);
        }
    }
    
    Ok(())
}

/// Output parsed GCode blocks
fn output_parsed(content: &str) -> Result<()> {
    use gcode::Parser;
    
    let mut parser = Parser::new()
        .map_err(|e| anyhow::anyhow!("Failed to create parser: {:?}", e))?;
    
    parser.parse_string(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse: {:?}", e))?;
    
    let block_count = parser.block_count();
    println!("Parsed {} blocks:\n", block_count);
    
    for i in 0..block_count {
        let original_text = parser.get_block_original_text(i);
        println!("Block {}: {}", i, original_text.trim());
        println!();
    }
    
    Ok(())
}

/// Output debug visualization info
fn output_debug(content: &str, resolution: f64, feed_rate: f64, verbose: bool) -> Result<()> {
    use gcode::{TrajectoryGenerator, MachineConfig, KinematicsType};
    
    println!("=== GCodeFlow Debug Output ===\n");
    println!("Input GCode ({} lines):", content.lines().count());
    for (i, line) in content.lines().enumerate() {
        println!("  {:3}: {}", i + 1, line);
    }
    println!();
    
    let config = MachineConfig {
        min_x: -500.0,
        max_x: 500.0,
        min_y: -500.0,
        max_y: 500.0,
        min_z: 0.0,
        max_z: 400.0,
        min_a: -360.0,
        max_a: 360.0,
        min_b: -360.0,
        max_b: 360.0,
        min_c: -360.0,
        max_c: 360.0,
        max_velocity_linear: feed_rate * 2.0,
        max_velocity_angular: 3600.0,
        max_acceleration: 3000.0,
        max_jerk: 10000.0,
        default_feed_rate: feed_rate,
        rapid_feed_rate: feed_rate * 2.0,
        use_metric: true,
        kinematics_type: KinematicsType::Cartesian,
    };
    
    println!("Machine Config:");
    println!("  Feed rate: {} mm/min", feed_rate);
    println!("  Rapid rate: {} mm/min", feed_rate * 2.0);
    println!("  Resolution: {}s", resolution);
    println!();
    
    let mut generator = TrajectoryGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create generator: {:?}", e))?;
    
    let points = generator.generate_from_gcode(
        content,
        config.max_velocity_linear,
        config.max_acceleration,
        config.max_jerk,
        resolution,
    )
        .map_err(|e| anyhow::anyhow!("Failed to generate trajectory: {:?}", e))?;
    
    println!("Trajectory Summary:");
    println!("  Total points: {}", points.len());
    if let Some(last) = points.last() {
        println!("  Duration: {:.3}s", last.time);
    }
    
    // Compute bounds
    if !points.is_empty() {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        let mut min_z = f64::MAX;
        let mut max_z = f64::MIN;
        
        for pt in &points {
            min_x = min_x.min(pt.position.x);
            max_x = max_x.max(pt.position.x);
            min_y = min_y.min(pt.position.y);
            max_y = max_y.max(pt.position.y);
            min_z = min_z.min(pt.position.z);
            max_z = max_z.max(pt.position.z);
        }
        
        println!("  Bounds:");
        println!("    X: {:.3} to {:.3} (range: {:.3})", min_x, max_x, max_x - min_x);
        println!("    Y: {:.3} to {:.3} (range: {:.3})", min_y, max_y, max_y - min_y);
        println!("    Z: {:.3} to {:.3} (range: {:.3})", min_z, max_z, max_z - min_z);
        
        // Compute total distance
        let mut total_dist = 0.0;
        for i in 1..points.len() {
            let dx = points[i].position.x - points[i-1].position.x;
            let dy = points[i].position.y - points[i-1].position.y;
            let dz = points[i].position.z - points[i-1].position.z;
            total_dist += (dx*dx + dy*dy + dz*dz).sqrt();
        }
        println!("  Total distance: {:.3} mm", total_dist);
    }
    println!();
    
    // Group points by block/segment
    println!("Motion Segments (what gets rendered as traces):");
    let mut segments: Vec<(usize, usize, gcode::MotionType, f64, f64, f64, f64, f64, f64)> = Vec::new();
    let mut current_block = 0usize;
    let mut current_type: gcode::MotionType = points.first().map(|p| gcode::MotionType::from(p.motion_type)).unwrap_or(gcode::MotionType::Rapid);
    let mut segment_start = 0;
    
    for (i, pt) in points.iter().enumerate() {
        let pt_motion_type = gcode::MotionType::from(pt.motion_type);
        if pt.block_index as usize != current_block || pt_motion_type != current_type {
            if i > segment_start {
                let start_pt = &points[segment_start];
                let end_pt = &points[i - 1];
                segments.push((
                    current_block,
                    i - segment_start,
                    current_type,
                    start_pt.position.x, start_pt.position.y, start_pt.position.z,
                    end_pt.position.x, end_pt.position.y, end_pt.position.z,
                ));
            }
            current_block = pt.block_index as usize;
            current_type = pt_motion_type;
            segment_start = i;
        }
    }
    // Add last segment
    if !points.is_empty() && points.len() > segment_start {
        let start_pt = &points[segment_start];
        let end_pt = points.last().unwrap();
        segments.push((
            current_block,
            points.len() - segment_start,
            current_type,
            start_pt.position.x, start_pt.position.y, start_pt.position.z,
            end_pt.position.x, end_pt.position.y, end_pt.position.z,
        ));
    }
    
    println!("  {} segments:", segments.len());
    for (i, (block, count, motion_type, sx, sy, sz, ex, ey, ez)) in segments.iter().enumerate() {
        let type_str = match motion_type {
            gcode::MotionType::Rapid => "RAPID",
            gcode::MotionType::Linear => "LINEAR",
            gcode::MotionType::ArcCW => "ARC_CW",
            gcode::MotionType::ArcCCW => "ARC_CCW",
            gcode::MotionType::Spline => "SPLINE",
        };
        println!("    Segment {}: block={}, type={}, {} pts", i, block, type_str, count);
        println!("      Start: ({:.3}, {:.3}, {:.3})", sx, sy, sz);
        println!("      End:   ({:.3}, {:.3}, {:.3})", ex, ey, ez);
        
        // Check for degenerate segments (start == end)
        let dx = ex - sx;
        let dy = ey - sy;
        let dz = ez - sz;
        let dist = (dx*dx + dy*dy + dz*dz).sqrt();
        if dist < 0.001 {
            println!("      WARNING: Degenerate segment (zero length)!");
        }
    }
    println!();
    
    // Bevy coordinate system note
    println!("Bevy Rendering Note:");
    println!("  Bevy uses Y-up coordinate system");
    println!("  GCode uses Z-up (typical CNC convention)");
    println!("  Rendering swaps Y<->Z: GCode(X,Y,Z) -> Bevy(X,Z,Y)");
    println!();
    
    if verbose {
        println!("All Points:");
        for (i, pt) in points.iter().enumerate() {
            let type_char = match gcode::MotionType::from(pt.motion_type) {
                gcode::MotionType::Rapid => 'R',
                gcode::MotionType::Linear => 'L',
                gcode::MotionType::ArcCW => 'C',
                gcode::MotionType::ArcCCW => 'A',
                gcode::MotionType::Spline => 'S',
            };
            println!("  {:4}: t={:.4}s {} block={} pos=({:.3}, {:.3}, {:.3}) -> Bevy({:.3}, {:.3}, {:.3})",
                i, pt.time, type_char, pt.block_index,
                pt.position.x, pt.position.y, pt.position.z,
                pt.position.x, pt.position.z, pt.position.y);  // Bevy coords
        }
    }
    
    Ok(())
}

/// Verify trajectory generation for integration testing
fn output_verify(
    content: &str,
    resolution: f64,
    feed_rate: f64,
    expect_points: Option<&usize>,
    expect_x: Option<&f64>,
    expect_y: Option<&f64>,
    expect_z: Option<&f64>,
    expect_duration: Option<&f64>,
    format: &str,
) -> Result<()> {
    use gcode::{TrajectoryGenerator, MachineConfig, KinematicsType};
    
    let config = MachineConfig {
        min_x: -500.0,
        max_x: 500.0,
        min_y: -500.0,
        max_y: 500.0,
        min_z: -100.0,
        max_z: 400.0,
        min_a: -360.0,
        max_a: 360.0,
        min_b: -360.0,
        max_b: 360.0,
        min_c: -360.0,
        max_c: 360.0,
        max_velocity_linear: feed_rate * 2.0,
        max_velocity_angular: 3600.0,
        max_acceleration: 3000.0,
        max_jerk: 10000.0,
        default_feed_rate: feed_rate,
        rapid_feed_rate: feed_rate * 2.0,
        use_metric: true,
        kinematics_type: KinematicsType::Cartesian,
    };
    
    let mut generator = TrajectoryGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create generator: {:?}", e))?;
    
    let points = generator.generate_from_gcode(
        content,
        config.max_velocity_linear,
        config.max_acceleration,
        config.max_jerk,
        resolution,
    )
        .map_err(|e| anyhow::anyhow!("Failed to generate trajectory: {:?}", e))?;
    
    let num_points = points.len();
    let final_pos = points.last().map(|p| (p.position.x, p.position.y, p.position.z));
    let duration = points.last().map(|p| p.time).unwrap_or(0.0);
    
    // Compute bounds and distance
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;
    let mut total_dist = 0.0;
    
    for (i, pt) in points.iter().enumerate() {
        min_x = min_x.min(pt.position.x);
        max_x = max_x.max(pt.position.x);
        min_y = min_y.min(pt.position.y);
        max_y = max_y.max(pt.position.y);
        min_z = min_z.min(pt.position.z);
        max_z = max_z.max(pt.position.z);
        
        if i > 0 {
            let dx = pt.position.x - points[i-1].position.x;
            let dy = pt.position.y - points[i-1].position.y;
            let dz = pt.position.z - points[i-1].position.z;
            total_dist += (dx*dx + dy*dy + dz*dz).sqrt();
        }
    }
    
    // Validate expectations
    let mut failures: Vec<String> = Vec::new();
    const TOL_POS: f64 = 0.001;
    const TOL_TIME: f64 = 0.01;
    
    if let Some(exp) = expect_points {
        if num_points != *exp {
            failures.push(format!("Point count: expected {}, got {}", exp, num_points));
        }
    }
    
    if let (Some(fx), Some(fy), Some(fz)) = (final_pos.as_ref().map(|p| p.0), final_pos.as_ref().map(|p| p.1), final_pos.as_ref().map(|p| p.2)) {
        if let Some(exp) = expect_x {
            if (fx - exp).abs() > TOL_POS {
                failures.push(format!("Final X: expected {:.3}, got {:.3}", exp, fx));
            }
        }
        if let Some(exp) = expect_y {
            if (fy - exp).abs() > TOL_POS {
                failures.push(format!("Final Y: expected {:.3}, got {:.3}", exp, fy));
            }
        }
        if let Some(exp) = expect_z {
            if (fz - exp).abs() > TOL_POS {
                failures.push(format!("Final Z: expected {:.3}, got {:.3}", exp, fz));
            }
        }
    }
    
    if let Some(exp) = expect_duration {
        if (duration - exp).abs() > TOL_TIME {
            failures.push(format!("Duration: expected {:.3}s, got {:.3}s", exp, duration));
        }
    }
    
    let passed = failures.is_empty();
    
    // Output based on format
    match format {
        "json" => {
            let (fx, fy, fz) = final_pos.unwrap_or((0.0, 0.0, 0.0));
            println!("{{");
            println!("  \"passed\": {},", passed);
            println!("  \"points\": {},", num_points);
            println!("  \"duration\": {:.6},", duration);
            println!("  \"final_position\": {{ \"x\": {:.6}, \"y\": {:.6}, \"z\": {:.6} }},", fx, fy, fz);
            println!("  \"bounds\": {{");
            println!("    \"min\": {{ \"x\": {:.6}, \"y\": {:.6}, \"z\": {:.6} }},", 
                if min_x == f64::MAX { 0.0 } else { min_x },
                if min_y == f64::MAX { 0.0 } else { min_y },
                if min_z == f64::MAX { 0.0 } else { min_z });
            println!("    \"max\": {{ \"x\": {:.6}, \"y\": {:.6}, \"z\": {:.6} }}", 
                if max_x == f64::MIN { 0.0 } else { max_x },
                if max_y == f64::MIN { 0.0 } else { max_y },
                if max_z == f64::MIN { 0.0 } else { max_z });
            println!("  }},");
            println!("  \"total_distance\": {:.6},", total_dist);
            println!("  \"failures\": [");
            for (i, f) in failures.iter().enumerate() {
                let comma = if i < failures.len() - 1 { "," } else { "" };
                println!("    \"{}\"{}", f, comma);
            }
            println!("  ]");
            println!("}}");
        }
        "verbose" => {
            println!("=== Trajectory Verification Report ===");
            println!();
            println!("Input:");
            println!("  Resolution: {}s", resolution);
            println!("  Feed rate: {} mm/min", feed_rate);
            println!();
            println!("Results:");
            println!("  Points: {}", num_points);
            println!("  Duration: {:.3}s", duration);
            if let Some((fx, fy, fz)) = final_pos {
                println!("  Final position: ({:.3}, {:.3}, {:.3})", fx, fy, fz);
            }
            println!("  Total distance: {:.3} mm", total_dist);
            if min_x != f64::MAX {
                println!("  Bounds X: {:.3} to {:.3}", min_x, max_x);
                println!("  Bounds Y: {:.3} to {:.3}", min_y, max_y);
                println!("  Bounds Z: {:.3} to {:.3}", min_z, max_z);
            }
            println!();
            
            if !failures.is_empty() {
                println!("FAILURES:");
                for f in &failures {
                    println!("  ✗ {}", f);
                }
                println!();
            }
            
            println!("Status: {}", if passed { "PASSED ✓" } else { "FAILED ✗" });
        }
        _ => {
            // "summary" format
            let (fx, fy, fz) = final_pos.unwrap_or((0.0, 0.0, 0.0));
            println!("points={} duration={:.3}s final=({:.3},{:.3},{:.3}) dist={:.3}mm status={}",
                num_points, duration, fx, fy, fz, total_dist,
                if passed { "PASS" } else { "FAIL" });
            
            if !failures.is_empty() {
                for f in &failures {
                    println!("  FAIL: {}", f);
                }
            }
        }
    }
    
    if passed {
        Ok(())
    } else {
        anyhow::bail!("Verification failed: {} check(s) failed", failures.len())
    }
}
