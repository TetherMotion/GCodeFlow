//! FFI bindings to the C++ GCode library via cxx.

use thiserror::Error;
use glam::Vec3;

/// Error types for GCode operations
#[derive(Error, Debug, Clone)]
pub enum GCodeError {
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, GCodeError>;

#[cxx::bridge(namespace = "gcode_ffi")]
mod ffi {
    #[derive(Debug, Clone, Default)]
    struct FfiPosition {
        x: f64,
        y: f64,
        z: f64,
        a: f64,
        b: f64,
        c: f64,
        u: f64,
        v: f64,
        w: f64,
    }

    #[derive(Debug, Clone)]
    struct FfiMotionSegment {
        start: FfiPosition,
        end: FfiPosition,
        center: FfiPosition,
        feed_rate: f64,
        arc_radius: f64,
        arc_sweep: f64,
        segment_length: f64,
        segment_time: f64,
        motion_type: u8,
        block_index: i32,
        plane: u8,
        is_rapid: bool,
    }

    #[derive(Debug, Clone)]
    struct FfiTrajectoryPoint {
        time: f64,
        position: FfiPosition,
        velocity: FfiPosition,
        acceleration: FfiPosition,
        block_index: i32,
        segment_index: i32,
        motion_type: u8,
        is_interpolated: bool,
    }

    #[derive(Debug, Clone, Default)]
    struct FfiHighlightSpan {
        start: usize,
        length: usize,
        kind: u8,
    }

    unsafe extern "C++" {
        include!("gcode_ffi.hpp");

        type FfiParser;
        type FfiInterpreter;
        type FfiTrajectoryGenerator;

        fn new_parser() -> UniquePtr<FfiParser>;
        fn new_interpreter() -> UniquePtr<FfiInterpreter>;
        fn new_trajectory_generator() -> UniquePtr<FfiTrajectoryGenerator>;

        fn last_error() -> String;

        // FfiParser
        fn parse_string(self: &FfiParser, gcode: &str) -> bool;
        fn parse_file(self: &FfiParser, path: &str) -> bool;
        fn block_count(self: &FfiParser) -> usize;
        fn get_block_original_text(self: &FfiParser, index: usize) -> String;

        // FfiInterpreter
        fn configure(self: Pin<&mut FfiInterpreter>, max_vel: f64, max_accel: f64, max_jerk: f64);
        fn load_blocks(self: Pin<&mut FfiInterpreter>, parser: &FfiParser) -> bool;
        fn segment_count(self: &FfiInterpreter) -> usize;
        fn get_segment(self: &FfiInterpreter, index: usize) -> FfiMotionSegment;
        fn get_all_segments(self: &FfiInterpreter) -> Vec<FfiMotionSegment>;

        // FfiTrajectoryGenerator
        fn configure(self: Pin<&mut FfiTrajectoryGenerator>, time_step: f64, max_deviation: f64);
        fn generate(self: Pin<&mut FfiTrajectoryGenerator>, interp: &FfiInterpreter) -> bool;
        fn point_count(self: &FfiTrajectoryGenerator) -> usize;
        fn duration(self: &FfiTrajectoryGenerator) -> f64;
        fn get_point(self: &FfiTrajectoryGenerator, index: usize) -> FfiTrajectoryPoint;
        fn get_all_points(self: &FfiTrajectoryGenerator) -> Vec<FfiTrajectoryPoint>;
        
        // High-resolution sampling for plotting
        fn sample_at_interval(self: &FfiTrajectoryGenerator, interval_seconds: f64) -> Vec<FfiTrajectoryPoint>;
        fn sample_adaptive(self: &FfiTrajectoryGenerator, max_deviation_mm: f64) -> Vec<FfiTrajectoryPoint>;
        fn query_at_time(self: &FfiTrajectoryGenerator, time_seconds: f64) -> FfiTrajectoryPoint;
        fn get_block_range_for_lines(self: &FfiTrajectoryGenerator, start_line: usize, end_line: usize, 
                                       out_start_block: &mut usize, out_end_block: &mut usize) -> bool;

        // Syntax highlighting (C++ lexer-based)
        fn highlight_line(line: &str) -> Vec<FfiHighlightSpan>;
    }
}

// --------------------------------------------------------------------------
// Re-export bridge types and add helper conversions
// --------------------------------------------------------------------------
pub use ffi::{FfiMotionSegment, FfiPosition, FfiTrajectoryPoint};

impl FfiPosition {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

/// Motion segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionType {
    Rapid,
    Linear,
    ArcCW,
    ArcCCW,
    Spline,
}

impl From<u8> for MotionType {
    fn from(value: u8) -> Self {
        match value {
            0 => MotionType::Rapid,
            1 => MotionType::Linear,
            2 => MotionType::ArcCW,
            3 => MotionType::ArcCCW,
            4 => MotionType::Spline,
            _ => MotionType::Linear,
        }
    }
}

// --------------------------------------------------------------------------
// Safe Rust wrappers
// --------------------------------------------------------------------------

pub struct Parser {
    inner: cxx::UniquePtr<ffi::FfiParser>,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let inner = ffi::new_parser();
        if inner.is_null() {
            return Err(GCodeError::OperationFailed("Could not create parser".into()));
        }
        Ok(Self { inner })
    }

    pub fn parse_string(&mut self, gcode: &str) -> Result<()> {
        if !self.inner.parse_string(gcode) {
            return Err(GCodeError::ParseFailed(ffi::last_error()));
        }
        Ok(())
    }

    pub fn parse_file(&mut self, path: &str) -> Result<()> {
        if !self.inner.parse_file(path) {
            return Err(GCodeError::ParseFailed(ffi::last_error()));
        }
        Ok(())
    }

    pub fn block_count(&self) -> usize {
        self.inner.block_count()
    }

    pub fn get_block_original_text(&self, index: usize) -> String {
        self.inner.get_block_original_text(index)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new().expect("Failed to create parser")
    }
}

pub struct Interpreter {
    inner: cxx::UniquePtr<ffi::FfiInterpreter>,
}

impl Interpreter {
    pub fn new() -> Result<Self> {
        let inner = ffi::new_interpreter();
        if inner.is_null() {
            return Err(GCodeError::OperationFailed("Could not create interpreter".into()));
        }
        Ok(Self { inner })
    }

    pub fn configure(&mut self, max_vel: f64, max_accel: f64, max_jerk: f64) {
        self.inner.pin_mut().configure(max_vel, max_accel, max_jerk);
    }

    pub fn load_blocks(&mut self, parser: &Parser) -> Result<()> {
        if !self.inner.pin_mut().load_blocks(&parser.inner) {
            return Err(GCodeError::OperationFailed(ffi::last_error()));
        }
        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.inner.segment_count()
    }

    pub fn get_segment(&self, index: usize) -> FfiMotionSegment {
        self.inner.get_segment(index)
    }

    pub fn get_all_segments(&self) -> Vec<FfiMotionSegment> {
        self.inner.get_all_segments()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create interpreter")
    }
}

pub struct TrajectoryGenerator {
    inner: cxx::UniquePtr<ffi::FfiTrajectoryGenerator>,
}

impl TrajectoryGenerator {
    pub fn new() -> Result<Self> {
        let inner = ffi::new_trajectory_generator();
        if inner.is_null() {
            return Err(GCodeError::OperationFailed("Could not create trajectory generator".into()));
        }
        Ok(Self { inner })
    }

    pub fn configure(&mut self, time_step: f64, max_deviation: f64) {
        self.inner.pin_mut().configure(time_step, max_deviation);
    }

    pub fn generate(&mut self, interp: &Interpreter) -> Result<()> {
        if !self.inner.pin_mut().generate(&interp.inner) {
            return Err(GCodeError::GenerationFailed(ffi::last_error()));
        }
        Ok(())
    }

    pub fn point_count(&self) -> usize {
        self.inner.point_count()
    }

    pub fn duration(&self) -> f64 {
        self.inner.duration()
    }

    pub fn get_point(&self, index: usize) -> FfiTrajectoryPoint {
        self.inner.get_point(index)
    }

    pub fn get_all_points(&self) -> Vec<FfiTrajectoryPoint> {
        self.inner.get_all_points()
    }
    
    /// Sample trajectory at regular time intervals for plotting
    pub fn sample_at_interval(&self, interval_seconds: f64) -> Vec<FfiTrajectoryPoint> {
        self.inner.sample_at_interval(interval_seconds)
    }
    
    /// Sample trajectory adaptively based on spatial deviation
    pub fn sample_adaptive(&self, max_deviation_mm: f64) -> Vec<FfiTrajectoryPoint> {
        self.inner.sample_adaptive(max_deviation_mm)
    }
    
    /// Query trajectory state at a specific time
    pub fn query_at_time(&self, time_seconds: f64) -> FfiTrajectoryPoint {
        self.inner.query_at_time(time_seconds)
    }
    
    /// Get block index range for given gcode line range
    pub fn get_block_range_for_lines(&self, start_line: usize, end_line: usize) -> Option<(usize, usize)> {
        let mut start_block: usize = 0;
        let mut end_block: usize = 0;
        
        if self.inner.get_block_range_for_lines(start_line, end_line, &mut start_block, &mut end_block) {
            Some((start_block, end_block))
        } else {
            None
        }
    }

    /// Convenience: parse gcode string and generate trajectory in one call.
    pub fn generate_from_gcode(
        &mut self,
        gcode: &str,
        max_vel: f64,
        max_accel: f64,
        max_jerk: f64,
        time_resolution: f64,
    ) -> Result<Vec<FfiTrajectoryPoint>> {
        let mut parser = Parser::new()?;
        parser.parse_string(gcode)?;

        let mut interp = Interpreter::new()?;
        interp.configure(max_vel, max_accel, max_jerk);
        interp.load_blocks(&parser)?;

        self.configure(time_resolution, 0.01);
        self.generate(&interp)?;

        Ok(self.get_all_points())
    }
}

impl Default for TrajectoryGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create trajectory generator")
    }
}

// --------------------------------------------------------------------------
// Compatibility re-exports for the rest of the crate
// --------------------------------------------------------------------------
pub use FfiPosition as Position;
pub use FfiMotionSegment as MotionSegment;
pub use FfiTrajectoryPoint as TrajectoryPoint;

/// Machine configuration (used by callers to set kinematic limits)
#[derive(Debug, Clone)]
pub struct MachineConfig {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub min_a: f64,
    pub max_a: f64,
    pub min_b: f64,
    pub max_b: f64,
    pub min_c: f64,
    pub max_c: f64,
    pub max_velocity_linear: f64,
    pub max_velocity_angular: f64,
    pub max_acceleration: f64,
    pub max_jerk: f64,
    pub default_feed_rate: f64,
    pub rapid_feed_rate: f64,
    pub use_metric: bool,
    pub kinematics_type: KinematicsType,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
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
            max_velocity_linear: 6000.0,
            max_velocity_angular: 3600.0,
            max_acceleration: 1000.0,
            max_jerk: 10000.0,
            default_feed_rate: 1000.0,
            rapid_feed_rate: 6000.0,
            use_metric: true,
            kinematics_type: KinematicsType::Cartesian,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum KinematicsType {
    #[default]
    Cartesian = 0,
    CoreXY = 1,
    Delta = 2,
    Scara = 3,
    FiveAxis = 4,
}

// --------------------------------------------------------------------------
// Syntax highlighting
// --------------------------------------------------------------------------

/// Token type for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    GCode,
    MCode,
    Axis,
    Parameter,
    Number,
    Comment,
    OCode,
    Operator,
    Variable,
    Error,
    Unknown,
}

/// A token with position and type
#[derive(Debug, Clone)]
pub struct Token {
    pub start: usize,
    pub length: usize,
    pub token_type: TokenType,
}

/// Tokenize a single line of GCode for syntax highlighting.
///
/// This uses the C++ lexer highlighter (via cxx FFI) so the UI shares the
/// exact same tokenization/highlighting rules as the C++ parser.
pub fn tokenize_line(line: &str) -> Vec<Token> {
    let spans = ffi::highlight_line(line);
    let mut tokens = Vec::with_capacity(spans.len());

    for span in spans {
        let token_type = match span.kind {
            1 => continue, // Whitespace
            2 => TokenType::Comment,
            3 => TokenType::GCode,
            4 => TokenType::MCode,
            5 => TokenType::OCode,
            6 => TokenType::Axis,
            8 => TokenType::Number,
            9 => TokenType::Parameter, // #... parameters / variables
            10 => TokenType::Operator, // [expr]
            11 => TokenType::Operator,
            12 => TokenType::Error,
            _ => TokenType::Unknown,
        };

        if span.length == 0 {
            continue;
        }

        tokens.push(Token {
            start: span.start,
            length: span.length,
            token_type,
        });
    }

    tokens
}

/// Pure-Rust fallback tokenizer for syntax highlighting.
///
/// Kept for debugging and as a reference implementation.
pub fn tokenize_line_pure_rust(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Comment starting with ; or (
        if c == ';' || c == '(' {
            let start = i;
            if c == '(' {
                // Find matching )
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // include the )
                }
            } else {
                // ; comment goes to end of line
                i = chars.len();
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::Comment,
            });
            continue;
        }

        // G-code (G0, G1, G2, G3, G17, etc.)
        if c == 'G' || c == 'g' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::GCode,
            });
            continue;
        }

        // M-code (M0, M3, M5, etc.)
        if c == 'M' || c == 'm' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::MCode,
            });
            continue;
        }

        // O-code (O100, O<name>, etc.)
        if c == 'O' || c == 'o' {
            let start = i;
            i += 1;
            // O-codes can be numeric or named
            if i < chars.len() && chars[i] == '<' {
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            } else {
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            // Check for sub/call/endsub/etc.
            let rest_start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::OCode,
            });
            continue;
        }

        // Axis letters (X, Y, Z, A, B, C, U, V, W, I, J, K, R, F, S, P, Q, L, N)
        if matches!(c, 'X' | 'x' | 'Y' | 'y' | 'Z' | 'z' | 
                        'A' | 'a' | 'B' | 'b' | 'C' | 'c' |
                        'U' | 'u' | 'V' | 'v' | 'W' | 'w' |
                        'I' | 'i' | 'J' | 'j' | 'K' | 'k' |
                        'R' | 'r' | 'F' | 'f' | 'S' | 's' |
                        'P' | 'p' | 'Q' | 'q' | 'L' | 'l' | 'N' | 'n' | 'T' | 't' | 'D' | 'd' | 'H' | 'h')
        {
            let start = i;
            tokens.push(Token {
                start,
                length: 1,
                token_type: TokenType::Axis,
            });
            i += 1;
            continue;
        }

        // # variable reference
        if c == '#' {
            let start = i;
            i += 1;
            // Named variable #<name> or numbered #123
            if i < chars.len() && chars[i] == '<' {
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            } else {
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::Variable,
            });
            continue;
        }

        // Number (including negative and decimal)
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
           || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token {
                start,
                length: i - start,
                token_type: TokenType::Number,
            });
            continue;
        }

        // Operators
        if matches!(c, '+' | '-' | '*' | '/' | '=' | '[' | ']' | '<' | '>') {
            tokens.push(Token {
                start: i,
                length: 1,
                token_type: TokenType::Operator,
            });
            i += 1;
            continue;
        }

        // Unknown/other
        tokens.push(Token {
            start: i,
            length: 1,
            token_type: TokenType::Unknown,
        });
        i += 1;
    }

    tokens
}
