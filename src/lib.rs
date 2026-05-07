//! GCodeFlow library
//!
//! A Rust library for G-code parsing, trajectory generation, and CNC machine simulation.
//! 
//! # Modules
//! 
//! * `gcode` - G-code parsing and interpretation (FFI to C++ Tether library)
//! 
//! # Example
//! 
//! ```no_run
//! use gcodeflow::gcode::{Parser, Interpreter, TrajectoryGenerator};
//! 
//! // Parse G-code
//! let mut parser = Parser::new().unwrap();
//! parser.parse_string("G0 X10 Y20\nG1 X30 Y40 F1000").unwrap();
//! 
//! // Generate trajectory
//! let mut interp = Interpreter::new().unwrap();
//! interp.configure(100.0, 500.0, 2000.0);
//! interp.load_blocks(&parser).unwrap();
//! 
//! let mut gen = TrajectoryGenerator::new().unwrap();
//! gen.configure(0.001, 0.01);
//! gen.generate(&interp).unwrap();
//! 
//! // Get trajectory points
//! let points = gen.get_all_points();
//! println!("Generated {} trajectory points", points.len());
//! ```

pub mod gcode;
pub mod benchmark;

// Re-export commonly used types at crate root for convenience
pub use gcode::{Parser, Interpreter, TrajectoryGenerator, GCodeError};
pub use gcode::{Position, MotionSegment, TrajectoryPoint, MotionType};
pub use gcode::{MachineConfig, KinematicsType};
pub use gcode::{Token, TokenType, tokenize_line};
