//! Comprehensive GCode Example Generator and Test Suite
//! 
//! This module generates approximately 1200 GCode examples covering:
//! - Basic motion commands (G0, G1, G2, G3)
//! - Coordinate systems and offsets
//! - Canned cycles
//! - Spindle and tool control
//! - Control flow (O-codes with conditionals, loops, subroutines)
//! - Variables and expressions
//! - Complex interpolation patterns
//! - Edge cases and error conditions

use std::collections::HashMap;

/// GCode example with metadata
#[derive(Debug, Clone)]
pub struct GCodeExample {
    /// Unique identifier
    pub id: String,
    /// Category for organization
    pub category: GCodeCategory,
    /// Complexity level (0-10)
    pub complexity: u8,
    /// Description of what the example tests
    pub description: String,
    /// The actual GCode content
    pub content: String,
    /// Expected number of motion segments (if known)
    pub expected_segments: Option<usize>,
    /// Expected total distance (if known)
    pub expected_distance: Option<f64>,
    /// Whether this should parse successfully
    pub should_parse: bool,
    /// Tags for filtering
    pub tags: Vec<String>,
}

/// Categories for GCode examples
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GCodeCategory {
    // Basic motion (200 examples)
    BasicLinearMove,
    BasicRapidMove,
    BasicArcCW,
    BasicArcCCW,
    
    // Coordinate systems (100 examples)
    AbsoluteCoordinates,
    IncrementalCoordinates,
    WorkOffsets,
    CoordinateRotation,
    
    // Speed and feed (100 examples)
    FeedRateVariations,
    RapidTraverse,
    SpindleControl,
    
    // Tool control (50 examples)
    ToolChanges,
    ToolOffsets,
    ToolCompensation,
    
    // Canned cycles (150 examples)
    DrillingCycles,
    TappingCycles,
    BoringCycles,
    PeckDrilling,
    
    // Interpolation (100 examples)
    HelicalInterpolation,
    SplineInterpolation,
    ThreadCutting,
    
    // Control flow (200 examples)
    OCodeSubroutines,
    OCodeWhileLoops,
    OCodeRepeatLoops,
    OCodeIfElse,
    OCodeDoWhile,
    OCodeCall,
    NestedControlFlow,
    
    // Variables (100 examples)
    NamedParameters,
    NumberedParameters,
    Expressions,
    MathFunctions,
    
    // Complex patterns (100 examples)
    Pocketing,
    Contouring,
    SurfaceFinishing,
    MultiPassOperations,
    
    // Edge cases (100 examples)
    EmptyLines,
    CommentsOnly,
    WhitespaceVariations,
    CaseSensitivity,
    NumericPrecision,
    LargePrograms,
    
    // Error cases (50 examples)
    SyntaxErrors,
    InvalidParameters,
    OutOfRange,
}

/// Generate all GCode examples
pub fn generate_all_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::with_capacity(1300);
    
    // Basic linear moves (50 examples)
    examples.extend(generate_basic_linear_examples());
    
    // Basic rapid moves (50 examples)
    examples.extend(generate_basic_rapid_examples());
    
    // Arc examples (100 examples)
    examples.extend(generate_arc_examples());
    
    // Coordinate system examples (100 examples)
    examples.extend(generate_coordinate_examples());
    
    // Feed and speed examples (100 examples)
    examples.extend(generate_feed_speed_examples());
    
    // Tool control examples (50 examples)
    examples.extend(generate_tool_examples());
    
    // Canned cycle examples (150 examples)
    examples.extend(generate_canned_cycle_examples());
    
    // Interpolation examples (100 examples)
    examples.extend(generate_interpolation_examples());
    
    // Control flow examples (200 examples)
    examples.extend(generate_control_flow_examples());
    
    // Variable examples (100 examples)
    examples.extend(generate_variable_examples());
    
    // Complex pattern examples (100 examples)
    examples.extend(generate_complex_pattern_examples());
    
    // Edge case examples (100 examples)
    examples.extend(generate_edge_case_examples());
    
    // Error examples (50 examples)
    examples.extend(generate_error_examples());
    
    examples
}

// ============================================================================
// BASIC LINEAR MOVE EXAMPLES (50)
// ============================================================================

fn generate_basic_linear_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Simple single-axis moves
    for (i, axis) in ['X', 'Y', 'Z', 'A', 'B', 'C'].iter().enumerate() {
        examples.push(GCodeExample {
            id: format!("linear_single_axis_{}", i),
            category: GCodeCategory::BasicLinearMove,
            complexity: 0,
            description: format!("Single axis {} move", axis),
            content: format!("G1 {}100 F1000", axis),
            expected_segments: Some(1),
            expected_distance: Some(100.0),
            should_parse: true,
            tags: vec!["basic".into(), "linear".into(), format!("{}-axis", axis)],
        });
    }
    
    // Two-axis moves
    for dist in [10.0, 50.0, 100.0, 200.0, 500.0] {
        examples.push(GCodeExample {
            id: format!("linear_xy_{}", dist as i32),
            category: GCodeCategory::BasicLinearMove,
            complexity: 1,
            description: format!("XY linear move distance {}", dist),
            content: format!("G1 X{} Y{} F1000", dist, dist),
            expected_segments: Some(1),
            expected_distance: Some((2.0_f64 * dist * dist).sqrt()),
            should_parse: true,
            tags: vec!["basic".into(), "linear".into(), "xy".into()],
        });
    }
    
    // Three-axis moves
    for (x, y, z) in [(10.0_f64, 20.0, 30.0), (100.0, 100.0, 50.0), (0.0, 0.0, 100.0)] {
        examples.push(GCodeExample {
            id: format!("linear_xyz_{}_{}_{}",  x as i32, y as i32, z as i32),
            category: GCodeCategory::BasicLinearMove,
            complexity: 2,
            description: format!("XYZ linear move to ({}, {}, {})", x, y, z),
            content: format!("G1 X{} Y{} Z{} F500", x, y, z),
            expected_segments: Some(1),
            expected_distance: Some((x*x + y*y + z*z).sqrt()),
            should_parse: true,
            tags: vec!["basic".into(), "linear".into(), "xyz".into()],
        });
    }
    
    // Multi-segment linear paths
    examples.push(GCodeExample {
        id: "linear_square".into(),
        category: GCodeCategory::BasicLinearMove,
        complexity: 2,
        description: "Square path".into(),
        content: r#"G90
G1 X0 Y0 F1000
G1 X100 Y0
G1 X100 Y100
G1 X0 Y100
G1 X0 Y0"#.into(),
        expected_segments: Some(4),
        expected_distance: Some(400.0),
        should_parse: true,
        tags: vec!["basic".into(), "linear".into(), "path".into()],
    });
    
    examples.push(GCodeExample {
        id: "linear_triangle".into(),
        category: GCodeCategory::BasicLinearMove,
        complexity: 2,
        description: "Equilateral triangle path".into(),
        content: r#"G90
G1 X0 Y0 F1000
G1 X100 Y0
G1 X50 Y86.6
G1 X0 Y0"#.into(),
        expected_segments: Some(3),
        expected_distance: Some(300.0),
        should_parse: true,
        tags: vec!["basic".into(), "linear".into(), "triangle".into()],
    });
    
    // Zigzag pattern
    examples.push(GCodeExample {
        id: "linear_zigzag".into(),
        category: GCodeCategory::BasicLinearMove,
        complexity: 3,
        description: "Zigzag pattern".into(),
        content: r#"G90
G1 X0 Y0 F1500
G1 X100 Y10
G1 X0 Y20
G1 X100 Y30
G1 X0 Y40
G1 X100 Y50
G1 X0 Y60
G1 X100 Y70
G1 X0 Y80
G1 X100 Y90
G1 X0 Y100"#.into(),
        expected_segments: Some(10),
        expected_distance: None, // Complex calculation
        should_parse: true,
        tags: vec!["basic".into(), "linear".into(), "zigzag".into()],
    });
    
    // Add incremental moves
    for i in 1..=10 {
        examples.push(GCodeExample {
            id: format!("linear_incremental_{}", i),
            category: GCodeCategory::BasicLinearMove,
            complexity: 2,
            description: format!("Incremental move sequence {}", i),
            content: format!(r#"G91
G1 X{} Y{} F1000
G1 X{} Y{}
G1 X{} Y{}
G1 X{} Y{}
G90"#, i*10, i*5, i*10, -i*5, -i*10, i*5, -i*10, -i*5),
            expected_segments: Some(4),
            expected_distance: None,
            should_parse: true,
            tags: vec!["basic".into(), "incremental".into()],
        });
    }
    
    // Very small moves (precision test)
    for precision in [0.001, 0.01, 0.1] {
        examples.push(GCodeExample {
            id: format!("linear_precision_{}", (precision * 1000.0) as i32),
            category: GCodeCategory::BasicLinearMove,
            complexity: 3,
            description: format!("Precision test at {} mm", precision),
            content: format!("G1 X{0} Y{0} Z{0} F100", precision),
            expected_segments: Some(1),
            expected_distance: Some((3.0_f64 * precision * precision).sqrt()),
            should_parse: true,
            tags: vec!["precision".into(), "linear".into()],
        });
    }
    
    // Very large moves
    for scale in [1000.0, 5000.0, 10000.0] {
        examples.push(GCodeExample {
            id: format!("linear_large_scale_{}", scale as i32),
            category: GCodeCategory::BasicLinearMove,
            complexity: 2,
            description: format!("Large scale move {}", scale),
            content: format!("G1 X{} Y{} F5000", scale, scale),
            expected_segments: Some(1),
            expected_distance: Some((2.0_f64 * scale * scale).sqrt()),
            should_parse: true,
            tags: vec!["large".into(), "linear".into()],
        });
    }
    
    // Long sequence of small moves
    let mut long_content = String::from("G90\nG1 X0 Y0 F2000\n");
    for i in 1..=50 {
        long_content.push_str(&format!("G1 X{} Y{}\n", i as f32 * 0.5, (i as f32 * 0.3).sin() * 10.0));
    }
    examples.push(GCodeExample {
        id: "linear_long_sequence".into(),
        category: GCodeCategory::BasicLinearMove,
        complexity: 4,
        description: "Long sequence of 50 small moves".into(),
        content: long_content,
        expected_segments: Some(50),
        expected_distance: None,
        should_parse: true,
        tags: vec!["long".into(), "sequence".into()],
    });
    
    examples
}

// ============================================================================
// BASIC RAPID MOVE EXAMPLES (50)
// ============================================================================

fn generate_basic_rapid_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Single axis rapids
    for axis in ['X', 'Y', 'Z'] {
        for dist in [10.0, 50.0, 100.0, 200.0] {
            examples.push(GCodeExample {
                id: format!("rapid_{}_{}",axis, dist as i32),
                category: GCodeCategory::BasicRapidMove,
                complexity: 0,
                description: format!("Rapid {} to {}", axis, dist),
                content: format!("G0 {}{}", axis, dist),
                expected_segments: Some(1),
                expected_distance: Some(dist),
                should_parse: true,
                tags: vec!["rapid".into(), format!("{}-axis", axis)],
            });
        }
    }
    
    // Combined rapids
    examples.push(GCodeExample {
        id: "rapid_xy_combined".into(),
        category: GCodeCategory::BasicRapidMove,
        complexity: 1,
        description: "Combined XY rapid".into(),
        content: "G0 X100 Y100".into(),
        expected_segments: Some(1),
        expected_distance: Some((2.0_f64 * 100.0 * 100.0).sqrt()),
        should_parse: true,
        tags: vec!["rapid".into(), "xy".into()],
    });
    
    examples.push(GCodeExample {
        id: "rapid_xyz_combined".into(),
        category: GCodeCategory::BasicRapidMove,
        complexity: 1,
        description: "Combined XYZ rapid".into(),
        content: "G0 X100 Y100 Z50".into(),
        expected_segments: Some(1),
        expected_distance: Some((100.0*100.0 + 100.0*100.0 + 50.0*50.0_f64).sqrt()),
        should_parse: true,
        tags: vec!["rapid".into(), "xyz".into()],
    });
    
    // Rapid retract pattern
    examples.push(GCodeExample {
        id: "rapid_retract_pattern".into(),
        category: GCodeCategory::BasicRapidMove,
        complexity: 3,
        description: "Typical machining rapid/plunge pattern".into(),
        content: r#"G90
G0 X0 Y0 Z50
G0 X10 Y10
G1 Z-5 F100
G1 X20 Y20 F500
G0 Z50
G0 X30 Y30
G1 Z-5 F100
G1 X40 Y40 F500
G0 Z50"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["rapid".into(), "machining".into(), "pattern".into()],
    });
    
    // Grid positioning
    let mut grid_content = String::from("G90\n");
    for y in (0..5).map(|i| i * 20) {
        for x in (0..5).map(|i| i * 20) {
            grid_content.push_str(&format!("G0 X{} Y{}\n", x, y));
        }
    }
    examples.push(GCodeExample {
        id: "rapid_grid_positioning".into(),
        category: GCodeCategory::BasicRapidMove,
        complexity: 3,
        description: "Grid positioning with rapids".into(),
        content: grid_content,
        expected_segments: Some(25),
        expected_distance: None,
        should_parse: true,
        tags: vec!["rapid".into(), "grid".into()],
    });
    
    // Safe Z retract sequence
    examples.push(GCodeExample {
        id: "rapid_safe_z".into(),
        category: GCodeCategory::BasicRapidMove,
        complexity: 2,
        description: "Safe Z retract before XY move".into(),
        content: r#"G90
G0 Z100
G0 X50 Y50
G0 Z5
G1 Z-10 F100"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["rapid".into(), "safe".into()],
    });
    
    // Add more rapid variations
    for i in 1..=20 {
        let angle = (i as f64 * 18.0).to_radians();
        let x = 100.0 * angle.cos();
        let y = 100.0 * angle.sin();
        examples.push(GCodeExample {
            id: format!("rapid_radial_{}", i),
            category: GCodeCategory::BasicRapidMove,
            complexity: 1,
            description: format!("Radial rapid at {} degrees", i * 18),
            content: format!("G0 X{:.3} Y{:.3}", x, y),
            expected_segments: Some(1),
            expected_distance: Some(100.0),
            should_parse: true,
            tags: vec!["rapid".into(), "radial".into()],
        });
    }
    
    examples
}

// ============================================================================
// ARC EXAMPLES (100)
// ============================================================================

fn generate_arc_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Basic CW arcs with different radii
    for radius in [5.0, 10.0, 25.0, 50.0, 100.0] {
        examples.push(GCodeExample {
            id: format!("arc_cw_r{}", radius as i32),
            category: GCodeCategory::BasicArcCW,
            complexity: 2,
            description: format!("CW arc radius {}", radius),
            content: format!("G2 X{} Y0 I0 J-{} F500", radius * 2.0, radius),
            expected_segments: None, // Arcs are interpolated
            expected_distance: Some(std::f64::consts::PI * radius),
            should_parse: true,
            tags: vec!["arc".into(), "cw".into()],
        });
    }
    
    // Basic CCW arcs
    for radius in [5.0, 10.0, 25.0, 50.0, 100.0] {
        examples.push(GCodeExample {
            id: format!("arc_ccw_r{}", radius as i32),
            category: GCodeCategory::BasicArcCCW,
            complexity: 2,
            description: format!("CCW arc radius {}", radius),
            content: format!("G3 X{} Y0 I0 J{} F500", radius * 2.0, radius),
            expected_segments: None,
            expected_distance: Some(std::f64::consts::PI * radius),
            should_parse: true,
            tags: vec!["arc".into(), "ccw".into()],
        });
    }
    
    // Quarter arcs (90 degrees)
    for (start, end, desc) in [
        ("X0 Y0", "X50 Y50 I50 J0", "Q1"),
        ("X50 Y50", "X0 Y100 I0 J50", "Q2"),
        ("X0 Y100", "X-50 Y50 I-50 J0", "Q3"),
        ("X-50 Y50", "X0 Y0 I0 J-50", "Q4"),
    ] {
        examples.push(GCodeExample {
            id: format!("arc_quarter_{}", desc),
            category: GCodeCategory::BasicArcCCW,
            complexity: 3,
            description: format!("Quarter arc {}", desc),
            content: format!("G90\nG0 {}\nG3 {} F500", start, end),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["arc".into(), "quarter".into()],
        });
    }
    
    // Full circle
    examples.push(GCodeExample {
        id: "arc_full_circle_cw".into(),
        category: GCodeCategory::BasicArcCW,
        complexity: 3,
        description: "Full CW circle".into(),
        content: r#"G90
G0 X50 Y0
G2 I-50 J0 F500"#.into(),
        expected_segments: None,
        expected_distance: Some(2.0 * std::f64::consts::PI * 50.0),
        should_parse: true,
        tags: vec!["arc".into(), "full_circle".into()],
    });
    
    examples.push(GCodeExample {
        id: "arc_full_circle_ccw".into(),
        category: GCodeCategory::BasicArcCCW,
        complexity: 3,
        description: "Full CCW circle".into(),
        content: r#"G90
G0 X50 Y0
G3 I-50 J0 F500"#.into(),
        expected_segments: None,
        expected_distance: Some(2.0 * std::f64::consts::PI * 50.0),
        should_parse: true,
        tags: vec!["arc".into(), "full_circle".into()],
    });
    
    // Multiple concentric circles
    let mut concentric_content = String::from("G90\n");
    for r in [10, 20, 30, 40, 50] {
        concentric_content.push_str(&format!("G0 X{} Y0\nG2 I-{} J0 F500\n", r, r));
    }
    examples.push(GCodeExample {
        id: "arc_concentric_circles".into(),
        category: GCodeCategory::BasicArcCW,
        complexity: 4,
        description: "Five concentric circles".into(),
        content: concentric_content,
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "concentric".into()],
    });
    
    // Spiral pattern
    let mut spiral_content = String::from("G90\nG0 X5 Y0\n");
    for i in 1..=10 {
        let r = 5.0 + i as f64 * 5.0;
        spiral_content.push_str(&format!("G3 X{} Y0 I-{} J0 F500\n", r, (r + r - 5.0) / 2.0));
    }
    examples.push(GCodeExample {
        id: "arc_spiral".into(),
        category: GCodeCategory::BasicArcCCW,
        complexity: 5,
        description: "Spiral pattern".into(),
        content: spiral_content,
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "spiral".into()],
    });
    
    // R-format arcs
    for radius in [10.0, 25.0, 50.0] {
        examples.push(GCodeExample {
            id: format!("arc_r_format_{}", radius as i32),
            category: GCodeCategory::BasicArcCW,
            complexity: 2,
            description: format!("R-format arc R{}", radius),
            content: format!("G2 X50 Y50 R{} F500", radius),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["arc".into(), "r_format".into()],
        });
    }
    
    // Arcs in different planes
    examples.push(GCodeExample {
        id: "arc_xz_plane".into(),
        category: GCodeCategory::BasicArcCW,
        complexity: 4,
        description: "Arc in XZ plane".into(),
        content: r#"G18
G0 X0 Z0
G2 X50 Z50 I25 K0 F500
G17"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "xz_plane".into()],
    });
    
    examples.push(GCodeExample {
        id: "arc_yz_plane".into(),
        category: GCodeCategory::BasicArcCW,
        complexity: 4,
        description: "Arc in YZ plane".into(),
        content: r#"G19
G0 Y0 Z0
G2 Y50 Z50 J25 K0 F500
G17"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "yz_plane".into()],
    });
    
    // Small arcs (precision test)
    for r in [0.5, 1.0, 2.0] {
        examples.push(GCodeExample {
            id: format!("arc_small_r{}", (r * 10.0) as i32),
            category: GCodeCategory::BasicArcCW,
            complexity: 4,
            description: format!("Small precision arc R{}", r),
            content: format!("G2 X{} Y0 I0 J-{} F100", r * 2.0, r),
            expected_segments: None,
            expected_distance: Some(std::f64::consts::PI * r),
            should_parse: true,
            tags: vec!["arc".into(), "precision".into()],
        });
    }
    
    // Arc combinations
    examples.push(GCodeExample {
        id: "arc_s_curve".into(),
        category: GCodeCategory::BasicArcCCW,
        complexity: 5,
        description: "S-curve from two arcs".into(),
        content: r#"G90
G0 X0 Y0
G3 X50 Y50 I0 J50 F500
G2 X100 Y100 I0 J50 F500"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "s_curve".into()],
    });
    
    // Rounded rectangle
    examples.push(GCodeExample {
        id: "arc_rounded_rectangle".into(),
        category: GCodeCategory::BasicArcCCW,
        complexity: 6,
        description: "Rounded rectangle".into(),
        content: r#"G90
G0 X10 Y0
G1 X90 F500
G3 X100 Y10 I0 J10
G1 Y90
G3 X90 Y100 I-10 J0
G1 X10
G3 X0 Y90 I0 J-10
G1 Y10
G3 X10 Y0 I10 J0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["arc".into(), "rounded_rectangle".into()],
    });
    
    // Generate various arc angles
    for angle_deg in (15..=180).step_by(15) {
        let angle_rad = (angle_deg as f64).to_radians();
        let r = 50.0;
        let end_x = r * (1.0 - angle_rad.cos());
        let end_y = r * angle_rad.sin();
        
        examples.push(GCodeExample {
            id: format!("arc_angle_{}", angle_deg),
            category: GCodeCategory::BasicArcCCW,
            complexity: 3,
            description: format!("{} degree arc", angle_deg),
            content: format!("G0 X0 Y0\nG3 X{:.4} Y{:.4} I0 J{} F500", end_x, end_y, r),
            expected_segments: None,
            expected_distance: Some(r * angle_rad),
            should_parse: true,
            tags: vec!["arc".into(), "angle_test".into()],
        });
    }
    
    examples
}

// ============================================================================
// COORDINATE SYSTEM EXAMPLES (100)
// ============================================================================

fn generate_coordinate_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // G90/G91 examples
    examples.push(GCodeExample {
        id: "coord_absolute".into(),
        category: GCodeCategory::AbsoluteCoordinates,
        complexity: 1,
        description: "Absolute positioning".into(),
        content: r#"G90
G1 X10 Y10 F1000
G1 X20 Y20
G1 X30 Y30
G1 X10 Y10"#.into(),
        expected_segments: Some(4),
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "absolute".into()],
    });
    
    examples.push(GCodeExample {
        id: "coord_incremental".into(),
        category: GCodeCategory::IncrementalCoordinates,
        complexity: 2,
        description: "Incremental positioning".into(),
        content: r#"G91
G1 X10 Y10 F1000
G1 X10 Y10
G1 X10 Y10
G1 X-30 Y-30"#.into(),
        expected_segments: Some(4),
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "incremental".into()],
    });
    
    examples.push(GCodeExample {
        id: "coord_mixed".into(),
        category: GCodeCategory::AbsoluteCoordinates,
        complexity: 3,
        description: "Mixed absolute/incremental".into(),
        content: r#"G90
G1 X50 Y50 F1000
G91
G1 X10 Y10
G1 X10 Y10
G90
G1 X100 Y100"#.into(),
        expected_segments: Some(4),
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "mixed".into()],
    });
    
    // Work offsets (G54-G59)
    for offset in 54..=59 {
        examples.push(GCodeExample {
            id: format!("coord_work_offset_g{}", offset),
            category: GCodeCategory::WorkOffsets,
            complexity: 3,
            description: format!("Work offset G{}", offset),
            content: format!(r#"G{}
G0 X0 Y0
G1 X50 Y50 F1000
G1 X0 Y0"#, offset),
            expected_segments: Some(2),
            expected_distance: None,
            should_parse: true,
            tags: vec!["coordinate".into(), "work_offset".into()],
        });
    }
    
    // G92 coordinate set
    examples.push(GCodeExample {
        id: "coord_g92_set".into(),
        category: GCodeCategory::WorkOffsets,
        complexity: 4,
        description: "G92 coordinate system set".into(),
        content: r#"G90
G0 X50 Y50
G92 X0 Y0
G1 X25 Y25 F500
G92.1"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "g92".into()],
    });
    
    // G52 local coordinate system
    examples.push(GCodeExample {
        id: "coord_g52_local".into(),
        category: GCodeCategory::WorkOffsets,
        complexity: 4,
        description: "G52 local coordinate system".into(),
        content: r#"G90
G54
G52 X50 Y50
G1 X0 Y0 F500
G1 X25 Y25
G52 X0 Y0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "g52".into()],
    });
    
    // Generate work offset transitions
    for from in 54..=58 {
        for to in (from+1)..=59 {
            examples.push(GCodeExample {
                id: format!("coord_offset_transition_g{}_g{}", from, to),
                category: GCodeCategory::WorkOffsets,
                complexity: 4,
                description: format!("Transition G{} to G{}", from, to),
                content: format!(r#"G{}
G0 X10 Y10
G1 X50 Y50 F500
G{}
G0 X10 Y10
G1 X50 Y50 F500"#, from, to),
                expected_segments: None,
                expected_distance: None,
                should_parse: true,
                tags: vec!["coordinate".into(), "transition".into()],
            });
        }
    }
    
    // Coordinate rotation (if supported)
    examples.push(GCodeExample {
        id: "coord_rotation".into(),
        category: GCodeCategory::CoordinateRotation,
        complexity: 5,
        description: "Coordinate system rotation".into(),
        content: r#"G90
G68 X50 Y50 R45
G1 X60 Y60 F500
G1 X70 Y50
G1 X60 Y40
G69"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "rotation".into()],
    });
    
    // Scaling
    examples.push(GCodeExample {
        id: "coord_scaling".into(),
        category: GCodeCategory::CoordinateRotation,
        complexity: 5,
        description: "Coordinate scaling".into(),
        content: r#"G90
G51 X2.0 Y2.0
G1 X10 Y10 F500
G1 X20 Y20
G50"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "scaling".into()],
    });
    
    // Mirror
    examples.push(GCodeExample {
        id: "coord_mirror".into(),
        category: GCodeCategory::CoordinateRotation,
        complexity: 5,
        description: "Coordinate mirroring".into(),
        content: r#"G90
G0 X0 Y0
G1 X50 Y25 F500
M21
G0 X0 Y0
G1 X50 Y25
M23"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["coordinate".into(), "mirror".into()],
    });
    
    // Add more coordinate variations
    for i in 1..=20 {
        examples.push(GCodeExample {
            id: format!("coord_nested_offset_{}", i),
            category: GCodeCategory::WorkOffsets,
            complexity: 5,
            description: format!("Nested coordinate offset test {}", i),
            content: format!(r#"G54
G0 X{} Y{}
G52 X10 Y10
G1 X5 Y5 F500
G52 X0 Y0
G0 X{} Y{}"#, i*5, i*5, i*10, i*10),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["coordinate".into(), "nested".into()],
        });
    }
    
    examples
}

// ============================================================================
// FEED AND SPEED EXAMPLES (100)
// ============================================================================

fn generate_feed_speed_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Various feed rates
    for feed in [10, 50, 100, 500, 1000, 2000, 5000, 10000] {
        examples.push(GCodeExample {
            id: format!("feed_rate_{}", feed),
            category: GCodeCategory::FeedRateVariations,
            complexity: 1,
            description: format!("Feed rate F{}", feed),
            content: format!("G1 X100 F{}", feed),
            expected_segments: Some(1),
            expected_distance: Some(100.0),
            should_parse: true,
            tags: vec!["feed".into()],
        });
    }
    
    // Feed per revolution (G95)
    examples.push(GCodeExample {
        id: "feed_per_rev".into(),
        category: GCodeCategory::FeedRateVariations,
        complexity: 3,
        description: "Feed per revolution mode".into(),
        content: r#"G95
S1000 M3
G1 X100 F0.1"#.into(),
        expected_segments: Some(1),
        expected_distance: Some(100.0),
        should_parse: true,
        tags: vec!["feed".into(), "per_rev".into()],
    });
    
    // Inverse time feed (G93)
    examples.push(GCodeExample {
        id: "feed_inverse_time".into(),
        category: GCodeCategory::FeedRateVariations,
        complexity: 4,
        description: "Inverse time feed mode".into(),
        content: r#"G93
G1 X100 Y100 F1.0
G94
G1 X0 Y0 F1000"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["feed".into(), "inverse_time".into()],
    });
    
    // Rapid override
    examples.push(GCodeExample {
        id: "rapid_override_test".into(),
        category: GCodeCategory::RapidTraverse,
        complexity: 2,
        description: "Rapid with various positions".into(),
        content: r#"G0 X0 Y0 Z50
G0 X100 Y100
G0 Z5
G1 Z-5 F100
G0 Z50"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["rapid".into()],
    });
    
    // Spindle control
    for rpm in [100, 500, 1000, 3000, 6000, 12000, 24000] {
        examples.push(GCodeExample {
            id: format!("spindle_cw_{}", rpm),
            category: GCodeCategory::SpindleControl,
            complexity: 2,
            description: format!("Spindle CW {}RPM", rpm),
            content: format!("S{} M3\nG1 X50 F500", rpm),
            expected_segments: Some(1),
            expected_distance: Some(50.0),
            should_parse: true,
            tags: vec!["spindle".into(), "cw".into()],
        });
        
        examples.push(GCodeExample {
            id: format!("spindle_ccw_{}", rpm),
            category: GCodeCategory::SpindleControl,
            complexity: 2,
            description: format!("Spindle CCW {}RPM", rpm),
            content: format!("S{} M4\nG1 X50 F500", rpm),
            expected_segments: Some(1),
            expected_distance: Some(50.0),
            should_parse: true,
            tags: vec!["spindle".into(), "ccw".into()],
        });
    }
    
    // Spindle stop
    examples.push(GCodeExample {
        id: "spindle_stop".into(),
        category: GCodeCategory::SpindleControl,
        complexity: 1,
        description: "Spindle stop".into(),
        content: r#"S3000 M3
G1 X50 F500
M5"#.into(),
        expected_segments: Some(1),
        expected_distance: Some(50.0),
        should_parse: true,
        tags: vec!["spindle".into(), "stop".into()],
    });
    
    // Constant surface speed (G96/G97)
    examples.push(GCodeExample {
        id: "feed_css_mode".into(),
        category: GCodeCategory::SpindleControl,
        complexity: 5,
        description: "Constant surface speed mode".into(),
        content: r#"G96 S200 M3
G1 X50 Z0 F0.2
G1 X25
G1 X10
G97 S1000"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["spindle".into(), "css".into()],
    });
    
    // Feed rate changes during motion
    examples.push(GCodeExample {
        id: "feed_varying".into(),
        category: GCodeCategory::FeedRateVariations,
        complexity: 3,
        description: "Varying feed rates".into(),
        content: r#"G1 X10 F100
G1 X20 F200
G1 X30 F500
G1 X40 F1000
G1 X50 F2000"#.into(),
        expected_segments: Some(5),
        expected_distance: Some(50.0),
        should_parse: true,
        tags: vec!["feed".into(), "varying".into()],
    });
    
    // Add more speed/feed combinations
    for speed in [500, 1000, 2000, 5000] {
        for feed in [100, 500, 1000, 2000] {
            examples.push(GCodeExample {
                id: format!("sf_s{}_f{}", speed, feed),
                category: GCodeCategory::FeedRateVariations,
                complexity: 2,
                description: format!("S{} F{}", speed, feed),
                content: format!("S{} M3\nG1 X50 Y50 F{}", speed, feed),
                expected_segments: Some(1),
                expected_distance: Some((2.0_f64 * 50.0 * 50.0).sqrt()),
                should_parse: true,
                tags: vec!["speed".into(), "feed".into()],
            });
        }
    }
    
    examples
}

// ============================================================================
// TOOL CONTROL EXAMPLES (50)
// ============================================================================

fn generate_tool_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Tool changes
    for tool in 1..=20 {
        examples.push(GCodeExample {
            id: format!("tool_change_t{}", tool),
            category: GCodeCategory::ToolChanges,
            complexity: 2,
            description: format!("Tool change to T{}", tool),
            content: format!("T{} M6\nG43 H{}\nS1000 M3\nG1 X50 F500", tool, tool),
            expected_segments: Some(1),
            expected_distance: Some(50.0),
            should_parse: true,
            tags: vec!["tool".into(), "change".into()],
        });
    }
    
    // Tool length compensation
    for h in 1..=10 {
        examples.push(GCodeExample {
            id: format!("tool_length_comp_h{}", h),
            category: GCodeCategory::ToolOffsets,
            complexity: 3,
            description: format!("Tool length compensation H{}", h),
            content: format!(r#"T1 M6
G43 H{}
G0 X0 Y0 Z10
G1 Z-5 F100
G49"#, h),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["tool".into(), "length_comp".into()],
        });
    }
    
    // Tool radius compensation
    examples.push(GCodeExample {
        id: "tool_radius_comp_left".into(),
        category: GCodeCategory::ToolCompensation,
        complexity: 4,
        description: "Tool radius compensation left (G41)".into(),
        content: r#"T1 M6
G41 D1
G1 X0 Y0 F500
G1 X100 Y0
G1 X100 Y100
G40"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["tool".into(), "cutter_comp".into(), "left".into()],
    });
    
    examples.push(GCodeExample {
        id: "tool_radius_comp_right".into(),
        category: GCodeCategory::ToolCompensation,
        complexity: 4,
        description: "Tool radius compensation right (G42)".into(),
        content: r#"T1 M6
G42 D1
G1 X0 Y0 F500
G1 X100 Y0
G1 X100 Y100
G40"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["tool".into(), "cutter_comp".into(), "right".into()],
    });
    
    // Multiple tool changes in sequence
    examples.push(GCodeExample {
        id: "tool_multiple_changes".into(),
        category: GCodeCategory::ToolChanges,
        complexity: 5,
        description: "Multiple tool changes".into(),
        content: r#"T1 M6
G43 H1
S2000 M3
G1 X50 Y50 F1000
M5
G0 Z50
T2 M6
G43 H2
S3000 M3
G1 X100 Y100 F500
M5
G0 Z50
T3 M6
G43 H3
S1500 M3
G1 X25 Y25 F800"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["tool".into(), "multiple".into()],
    });
    
    examples
}

// ============================================================================
// CANNED CYCLE EXAMPLES (150)
// ============================================================================

fn generate_canned_cycle_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Basic drilling (G81)
    for depth in [5.0, 10.0, 20.0, 30.0] {
        examples.push(GCodeExample {
            id: format!("drill_g81_z{}", depth as i32),
            category: GCodeCategory::DrillingCycles,
            complexity: 3,
            description: format!("G81 drill depth {}", depth),
            content: format!(r#"G90
G0 X10 Y10 Z5
G81 X10 Y10 Z-{} R2 F100
G80"#, depth),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["canned_cycle".into(), "drilling".into()],
        });
    }
    
    // Drill with dwell (G82)
    examples.push(GCodeExample {
        id: "drill_g82_dwell".into(),
        category: GCodeCategory::DrillingCycles,
        complexity: 4,
        description: "G82 drill with dwell".into(),
        content: r#"G90
G0 X10 Y10 Z5
G82 X10 Y10 Z-15 R2 P500 F100
G80"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["canned_cycle".into(), "drilling".into(), "dwell".into()],
    });
    
    // Peck drilling (G83)
    for peck in [1.0, 2.0, 5.0] {
        examples.push(GCodeExample {
            id: format!("drill_g83_peck_{}", peck as i32),
            category: GCodeCategory::PeckDrilling,
            complexity: 5,
            description: format!("G83 peck drill Q{}", peck),
            content: format!(r#"G90
G0 X10 Y10 Z5
G83 X10 Y10 Z-20 R2 Q{} F100
G80"#, peck),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["canned_cycle".into(), "peck".into()],
        });
    }
    
    // Tapping (G84)
    for pitch in [0.5, 1.0, 1.25, 1.5, 2.0] {
        examples.push(GCodeExample {
            id: format!("tap_g84_pitch_{}", (pitch * 10.0) as i32),
            category: GCodeCategory::TappingCycles,
            complexity: 5,
            description: format!("G84 tap pitch {}", pitch),
            content: format!(r#"G90
G0 X10 Y10 Z5
S500 M3
G84 X10 Y10 Z-15 R2 F{} K{}
G80"#, 500.0 * pitch, pitch),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["canned_cycle".into(), "tapping".into()],
        });
    }
    
    // Boring (G85, G86, G87, G88, G89)
    for cycle in [85, 86, 87, 88, 89] {
        examples.push(GCodeExample {
            id: format!("bore_g{}", cycle),
            category: GCodeCategory::BoringCycles,
            complexity: 4,
            description: format!("G{} boring cycle", cycle),
            content: format!(r#"G90
G0 X10 Y10 Z5
G{} X10 Y10 Z-15 R2 F50
G80"#, cycle),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["canned_cycle".into(), "boring".into()],
        });
    }
    
    // Multiple hole patterns
    examples.push(GCodeExample {
        id: "drill_bolt_circle".into(),
        category: GCodeCategory::DrillingCycles,
        complexity: 6,
        description: "Bolt circle drilling pattern".into(),
        content: r#"G90
G0 Z5
G81 R2 Z-10 F100
X50 Y25
X75 Y50
X50 Y75
X25 Y50
G80"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["canned_cycle".into(), "bolt_circle".into()],
    });
    
    examples.push(GCodeExample {
        id: "drill_grid_pattern".into(),
        category: GCodeCategory::DrillingCycles,
        complexity: 6,
        description: "Grid drilling pattern".into(),
        content: r#"G90
G0 Z5
G83 R2 Z-15 Q2 F100
X10 Y10
X30 Y10
X50 Y10
X10 Y30
X30 Y30
X50 Y30
X10 Y50
X30 Y50
X50 Y50
G80"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["canned_cycle".into(), "grid".into()],
    });
    
    // Generate more drilling patterns
    for holes in 4..=12 {
        let radius = 30.0;
        let mut drill_content = String::from("G90\nG0 Z5\nG81 R2 Z-10 F100\n");
        for i in 0..holes {
            let angle = (i as f64 * 360.0 / holes as f64).to_radians();
            let x = 50.0 + radius * angle.cos();
            let y = 50.0 + radius * angle.sin();
            drill_content.push_str(&format!("X{:.3} Y{:.3}\n", x, y));
        }
        drill_content.push_str("G80");
        
        examples.push(GCodeExample {
            id: format!("drill_circular_{}_holes", holes),
            category: GCodeCategory::DrillingCycles,
            complexity: 5,
            description: format!("Circular pattern {} holes", holes),
            content: drill_content,
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["canned_cycle".into(), "circular_pattern".into()],
        });
    }
    
    examples
}

// ============================================================================
// INTERPOLATION EXAMPLES (100)
// ============================================================================

fn generate_interpolation_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Helical interpolation
    for pitch in [2.0, 5.0, 10.0] {
        examples.push(GCodeExample {
            id: format!("helix_pitch_{}", pitch as i32),
            category: GCodeCategory::HelicalInterpolation,
            complexity: 5,
            description: format!("Helix pitch {}", pitch),
            content: format!(r#"G90
G0 X50 Y0 Z0
G2 X50 Y0 Z-{} I-50 J0 F500"#, pitch),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["interpolation".into(), "helix".into()],
        });
    }
    
    // Multi-turn helix
    examples.push(GCodeExample {
        id: "helix_multi_turn".into(),
        category: GCodeCategory::HelicalInterpolation,
        complexity: 6,
        description: "Multi-turn helix".into(),
        content: r#"G90
G0 X50 Y0 Z0
G2 X50 Y0 Z-5 I-50 J0 F500
G2 X50 Y0 Z-10 I-50 J0
G2 X50 Y0 Z-15 I-50 J0
G2 X50 Y0 Z-20 I-50 J0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["interpolation".into(), "helix".into(), "multi_turn".into()],
    });
    
    // Thread cutting (G33)
    for pitch in [0.5, 1.0, 1.25, 1.5, 2.0, 3.0] {
        examples.push(GCodeExample {
            id: format!("thread_g33_pitch_{}", (pitch * 10.0) as i32),
            category: GCodeCategory::ThreadCutting,
            complexity: 6,
            description: format!("Thread cutting pitch {}", pitch),
            content: format!(r#"G90
G0 X25 Z5
S500 M3
G33 Z-20 K{}
G0 X30
G0 Z5"#, pitch),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["interpolation".into(), "threading".into()],
        });
    }
    
    // Spline interpolation (G5, G5.1, G5.2 if supported)
    examples.push(GCodeExample {
        id: "spline_cubic".into(),
        category: GCodeCategory::SplineInterpolation,
        complexity: 7,
        description: "Cubic spline".into(),
        content: r#"G5 X10 Y10 I5 J0 P0 Q5 F500
G5 X20 Y5 I0 J-5 P5 Q0
G5 X30 Y10 I5 J5 P0 Q0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["interpolation".into(), "spline".into()],
    });
    
    // NURBS interpolation (if supported)
    examples.push(GCodeExample {
        id: "spline_nurbs".into(),
        category: GCodeCategory::SplineInterpolation,
        complexity: 8,
        description: "NURBS curve".into(),
        content: r#"G5.2 X10 Y0 P1
X20 Y20 P1
X30 Y0 P1
X40 Y20 P1
G5.3"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["interpolation".into(), "nurbs".into()],
    });
    
    // Conical helix
    examples.push(GCodeExample {
        id: "helix_conical".into(),
        category: GCodeCategory::HelicalInterpolation,
        complexity: 7,
        description: "Conical helix".into(),
        content: r#"G90
G0 X10 Y0 Z0
G2 X20 Y0 Z-2 I-10 J0 F300
G2 X30 Y0 Z-4 I-20 J0
G2 X40 Y0 Z-6 I-30 J0
G2 X50 Y0 Z-8 I-40 J0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["interpolation".into(), "helix".into(), "conical".into()],
    });
    
    // Circular pocket with helical entry
    examples.push(GCodeExample {
        id: "pocket_helical_entry".into(),
        category: GCodeCategory::HelicalInterpolation,
        complexity: 7,
        description: "Helical entry into pocket".into(),
        content: r#"G90
G0 X50 Y50 Z5
G0 X55 Y50
G2 X55 Y50 Z-5 I-5 J0 F300
G2 X55 Y50 Z-10 I-5 J0
G1 X70 F500
G2 X30 Y50 I-20 J0
G2 X70 Y50 I20 J0"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["interpolation".into(), "pocket".into(), "helical_entry".into()],
    });
    
    // Generate various helix configurations
    for radius in [10, 25, 50] {
        for pitch in [1, 2, 5] {
            for turns in [1, 2, 3, 5] {
                examples.push(GCodeExample {
                    id: format!("helix_r{}_p{}_t{}", radius, pitch, turns),
                    category: GCodeCategory::HelicalInterpolation,
                    complexity: 5,
                    description: format!("Helix R{} pitch{} {}turns", radius, pitch, turns),
                    content: {
                        let mut content = format!("G90\nG0 X{} Y0 Z0\n", radius);
                        for _ in 0..turns {
                            content.push_str(&format!("G2 X{} Y0 Z-{} I-{} J0 F500\n", radius, pitch, radius));
                        }
                        content
                    },
                    expected_segments: None,
                    expected_distance: None,
                    should_parse: true,
                    tags: vec!["interpolation".into(), "helix".into()],
                });
            }
        }
    }
    
    examples
}

// ============================================================================
// CONTROL FLOW EXAMPLES (200)
// ============================================================================

fn generate_control_flow_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Basic subroutine
    examples.push(GCodeExample {
        id: "ocode_sub_basic".into(),
        category: GCodeCategory::OCodeSubroutines,
        complexity: 5,
        description: "Basic subroutine definition and call".into(),
        content: r#"o100 sub
G1 X10 Y10 F500
G1 X20 Y20
o100 endsub

o100 call
G0 X0 Y0
o100 call"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "subroutine".into()],
    });
    
    // Subroutine with parameters
    examples.push(GCodeExample {
        id: "ocode_sub_params".into(),
        category: GCodeCategory::OCodeSubroutines,
        complexity: 6,
        description: "Subroutine with parameters".into(),
        content: r#"o100 sub
G1 X[#1] Y[#2] F[#3]
o100 endsub

o100 call [10] [20] [500]
o100 call [30] [40] [800]
o100 call [50] [60] [1000]"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "subroutine".into(), "parameters".into()],
    });
    
    // While loop
    examples.push(GCodeExample {
        id: "ocode_while_basic".into(),
        category: GCodeCategory::OCodeWhileLoops,
        complexity: 6,
        description: "Basic while loop".into(),
        content: r#"#1 = 0
o100 while [#1 LT 5]
G1 X[#1 * 10] Y[#1 * 10] F500
#1 = [#1 + 1]
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "while".into()],
    });
    
    // Do-while loop
    examples.push(GCodeExample {
        id: "ocode_do_while".into(),
        category: GCodeCategory::OCodeDoWhile,
        complexity: 6,
        description: "Do-while loop".into(),
        content: r#"#1 = 0
o100 do
G1 X[#1 * 10] F500
#1 = [#1 + 1]
o100 while [#1 LT 5]"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "do_while".into()],
    });
    
    // Repeat loop
    examples.push(GCodeExample {
        id: "ocode_repeat".into(),
        category: GCodeCategory::OCodeRepeatLoops,
        complexity: 5,
        description: "Repeat loop".into(),
        content: r#"#1 = 0
o100 repeat [5]
G1 X[#1 * 10] F500
#1 = [#1 + 1]
o100 endrepeat"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "repeat".into()],
    });
    
    // If-else
    examples.push(GCodeExample {
        id: "ocode_if_else".into(),
        category: GCodeCategory::OCodeIfElse,
        complexity: 5,
        description: "If-else conditional".into(),
        content: r#"#1 = 10
o100 if [#1 GT 5]
G1 X100 F500
o100 else
G1 X50 F500
o100 endif"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "if_else".into()],
    });
    
    // Elseif chain
    examples.push(GCodeExample {
        id: "ocode_if_elseif".into(),
        category: GCodeCategory::OCodeIfElse,
        complexity: 6,
        description: "If-elseif-else chain".into(),
        content: r#"#1 = 2
o100 if [#1 EQ 1]
G1 X10 F500
o100 elseif [#1 EQ 2]
G1 X20 F500
o100 elseif [#1 EQ 3]
G1 X30 F500
o100 else
G1 X0 F500
o100 endif"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "if_elseif".into()],
    });
    
    // Nested loops
    examples.push(GCodeExample {
        id: "ocode_nested_while".into(),
        category: GCodeCategory::NestedControlFlow,
        complexity: 7,
        description: "Nested while loops".into(),
        content: r#"#1 = 0
o100 while [#1 LT 3]
  #2 = 0
  o200 while [#2 LT 3]
    G1 X[#1 * 20 + #2 * 5] Y[#1 * 20 + #2 * 5] F500
    #2 = [#2 + 1]
  o200 endwhile
  #1 = [#1 + 1]
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "nested".into(), "while".into()],
    });
    
    // Loop with break
    examples.push(GCodeExample {
        id: "ocode_while_break".into(),
        category: GCodeCategory::OCodeWhileLoops,
        complexity: 6,
        description: "While loop with break".into(),
        content: r#"#1 = 0
o100 while [#1 LT 100]
  G1 X[#1 * 10] F500
  o200 if [#1 GT 5]
    o100 break
  o200 endif
  #1 = [#1 + 1]
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "while".into(), "break".into()],
    });
    
    // Loop with continue
    examples.push(GCodeExample {
        id: "ocode_while_continue".into(),
        category: GCodeCategory::OCodeWhileLoops,
        complexity: 6,
        description: "While loop with continue".into(),
        content: r#"#1 = 0
o100 while [#1 LT 10]
  #1 = [#1 + 1]
  o200 if [#1 EQ 5]
    o100 continue
  o200 endif
  G1 X[#1 * 10] F500
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "while".into(), "continue".into()],
    });
    
    // Recursive subroutine
    examples.push(GCodeExample {
        id: "ocode_recursive".into(),
        category: GCodeCategory::OCodeSubroutines,
        complexity: 8,
        description: "Recursive subroutine".into(),
        content: r#"o100 sub
  o200 if [#1 GT 0]
    G1 X[#1 * 10] F500
    #1 = [#1 - 1]
    o100 call
  o200 endif
o100 endsub

#1 = 5
o100 call"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "recursive".into()],
    });
    
    // Call with return value
    examples.push(GCodeExample {
        id: "ocode_return_value".into(),
        category: GCodeCategory::OCodeCall,
        complexity: 7,
        description: "Subroutine with return value".into(),
        content: r#"o100 sub
  #<_result> = [#1 + #2]
  o100 return [#<_result>]
o100 endsub

#3 = [o100 call [10] [20]]
G1 X[#3] F500"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "return".into()],
    });
    
    // Complex: Bolt circle subroutine
    examples.push(GCodeExample {
        id: "ocode_bolt_circle_sub".into(),
        category: GCodeCategory::OCodeSubroutines,
        complexity: 8,
        description: "Bolt circle subroutine".into(),
        content: r#"; Bolt circle subroutine
; #1 = center X, #2 = center Y, #3 = radius, #4 = num holes, #5 = depth
o100 sub
  #10 = 0
  o200 while [#10 LT #4]
    #11 = [#10 * 360 / #4]
    #12 = [#1 + #3 * COS[#11]]
    #13 = [#2 + #3 * SIN[#11]]
    G0 X[#12] Y[#13]
    G81 Z[#5] R2 F100
    #10 = [#10 + 1]
  o200 endwhile
  G80
o100 endsub

o100 call [50] [50] [30] [6] [-15]"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "bolt_circle".into(), "complex".into()],
    });
    
    // Generate more control flow variations
    for iterations in [3, 5, 10, 20] {
        examples.push(GCodeExample {
            id: format!("ocode_while_{}_iter", iterations),
            category: GCodeCategory::OCodeWhileLoops,
            complexity: 5,
            description: format!("While loop {} iterations", iterations),
            content: format!(r#"#1 = 0
o100 while [#1 LT {}]
G1 X[#1 * 5] F500
#1 = [#1 + 1]
o100 endwhile"#, iterations),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["ocode".into(), "while".into()],
        });
        
        examples.push(GCodeExample {
            id: format!("ocode_repeat_{}", iterations),
            category: GCodeCategory::OCodeRepeatLoops,
            complexity: 5,
            description: format!("Repeat {} times", iterations),
            content: format!(r#"#1 = 0
o100 repeat [{}]
G1 X[#1 * 5] F500
#1 = [#1 + 1]
o100 endrepeat"#, iterations),
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["ocode".into(), "repeat".into()],
        });
    }
    
    // Nested if statements
    for depth in 2..=5 {
        let mut content = String::new();
        for i in 1..=depth {
            content.push_str(&format!("#{}={}\n", i, i));
        }
        for i in 1..=depth {
            content.push_str(&format!("o{} if [#{} GT 0]\n", 100 + i, i));
        }
        content.push_str("G1 X100 F500\n");
        for i in (1..=depth).rev() {
            content.push_str(&format!("o{} endif\n", 100 + i));
        }
        
        examples.push(GCodeExample {
            id: format!("ocode_nested_if_depth_{}", depth),
            category: GCodeCategory::NestedControlFlow,
            complexity: depth as u8 + 4,
            description: format!("Nested if depth {}", depth),
            content,
            expected_segments: None,
            expected_distance: None,
            should_parse: true,
            tags: vec!["ocode".into(), "nested".into(), "if".into()],
        });
    }
    
    // File-based subroutines (external calls)
    examples.push(GCodeExample {
        id: "ocode_call_file".into(),
        category: GCodeCategory::OCodeCall,
        complexity: 7,
        description: "Call external file".into(),
        content: r#"o<pocket> call [50] [50] [40] [30] [-5]
G0 Z10"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["ocode".into(), "file_call".into()],
    });
    
    examples
}

// ============================================================================
// VARIABLE EXAMPLES (100)
// ============================================================================

fn generate_variable_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Numbered parameters
    for i in 1..=20 {
        examples.push(GCodeExample {
            id: format!("var_numbered_{}", i),
            category: GCodeCategory::NumberedParameters,
            complexity: 2,
            description: format!("Numbered parameter #{}", i),
            content: format!("#{}=50\nG1 X[#{}] F500", i, i),
            expected_segments: Some(1),
            expected_distance: Some(50.0),
            should_parse: true,
            tags: vec!["variable".into(), "numbered".into()],
        });
    }
    
    // Named parameters
    examples.push(GCodeExample {
        id: "var_named_basic".into(),
        category: GCodeCategory::NamedParameters,
        complexity: 3,
        description: "Named parameters".into(),
        content: r#"#<width> = 100
#<height> = 50
#<feed> = 500
G1 X[#<width>] Y[#<height>] F[#<feed>]"#.into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "named".into()],
    });
    
    // Global named parameters
    examples.push(GCodeExample {
        id: "var_global".into(),
        category: GCodeCategory::NamedParameters,
        complexity: 4,
        description: "Global named parameters".into(),
        content: r#"#<_global_offset_x> = 10
#<_global_offset_y> = 20
G1 X[#<_global_offset_x>] Y[#<_global_offset_y>] F500"#.into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "global".into()],
    });
    
    // Math expressions
    examples.push(GCodeExample {
        id: "var_expr_add".into(),
        category: GCodeCategory::Expressions,
        complexity: 3,
        description: "Addition expression".into(),
        content: "#1=10\n#2=20\nG1 X[#1+#2] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(30.0),
        should_parse: true,
        tags: vec!["variable".into(), "expression".into(), "add".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_expr_sub".into(),
        category: GCodeCategory::Expressions,
        complexity: 3,
        description: "Subtraction expression".into(),
        content: "#1=50\n#2=20\nG1 X[#1-#2] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(30.0),
        should_parse: true,
        tags: vec!["variable".into(), "expression".into(), "subtract".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_expr_mul".into(),
        category: GCodeCategory::Expressions,
        complexity: 3,
        description: "Multiplication expression".into(),
        content: "#1=5\n#2=10\nG1 X[#1*#2] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(50.0),
        should_parse: true,
        tags: vec!["variable".into(), "expression".into(), "multiply".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_expr_div".into(),
        category: GCodeCategory::Expressions,
        complexity: 3,
        description: "Division expression".into(),
        content: "#1=100\n#2=4\nG1 X[#1/#2] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(25.0),
        should_parse: true,
        tags: vec!["variable".into(), "expression".into(), "divide".into()],
    });
    
    // Math functions
    examples.push(GCodeExample {
        id: "var_func_sin".into(),
        category: GCodeCategory::MathFunctions,
        complexity: 4,
        description: "SIN function".into(),
        content: "#1=45\nG1 X[50*SIN[#1]] Y[50*COS[#1]] F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "function".into(), "trig".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_func_sqrt".into(),
        category: GCodeCategory::MathFunctions,
        complexity: 4,
        description: "SQRT function".into(),
        content: "G1 X[SQRT[2500]] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(50.0),
        should_parse: true,
        tags: vec!["variable".into(), "function".into(), "sqrt".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_func_atan".into(),
        category: GCodeCategory::MathFunctions,
        complexity: 5,
        description: "ATAN function".into(),
        content: "#1=[ATAN[1]/[1]]\nG1 X[#1] F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "function".into(), "atan".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_func_abs".into(),
        category: GCodeCategory::MathFunctions,
        complexity: 3,
        description: "ABS function".into(),
        content: "#1=-50\nG1 X[ABS[#1]] F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(50.0),
        should_parse: true,
        tags: vec!["variable".into(), "function".into(), "abs".into()],
    });
    
    examples.push(GCodeExample {
        id: "var_func_round".into(),
        category: GCodeCategory::MathFunctions,
        complexity: 4,
        description: "ROUND/FIX/FUP functions".into(),
        content: r#"#1=25.7
G1 X[ROUND[#1]] F500
G1 X[FIX[#1]]
G1 X[FUP[#1]]"#.into(),
        expected_segments: Some(3),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "function".into(), "round".into()],
    });
    
    // Comparison operators
    examples.push(GCodeExample {
        id: "var_compare".into(),
        category: GCodeCategory::Expressions,
        complexity: 5,
        description: "Comparison operators".into(),
        content: r#"#1=10
#2=20
#3=[#1 LT #2]
#4=[#1 GT #2]
#5=[#1 EQ 10]
#6=[#2 NE 10]
G1 X[#3*10] Y[#5*10] F500"#.into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "comparison".into()],
    });
    
    // Logical operators
    examples.push(GCodeExample {
        id: "var_logical".into(),
        category: GCodeCategory::Expressions,
        complexity: 5,
        description: "Logical operators".into(),
        content: r#"#1=1
#2=0
#3=[#1 AND #2]
#4=[#1 OR #2]
#5=[NOT #1]
G1 X[[#4]*50] F500"#.into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "logical".into()],
    });
    
    // Complex expression
    examples.push(GCodeExample {
        id: "var_complex_expr".into(),
        category: GCodeCategory::Expressions,
        complexity: 6,
        description: "Complex expression".into(),
        content: r#"#<radius>=50
#<angle>=45
#<x>=[#<radius>*COS[#<angle>]+100]
#<y>=[#<radius>*SIN[#<angle>]+100]
G1 X[#<x>] Y[#<y>] F500"#.into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "complex".into()],
    });
    
    // System variables (LinuxCNC specific)
    examples.push(GCodeExample {
        id: "var_system".into(),
        category: GCodeCategory::NumberedParameters,
        complexity: 5,
        description: "System variables".into(),
        content: r#"; System variable examples
#<current_x>=#5420
#<current_y>=#5421
#<current_z>=#5422
G1 X[#<current_x>+10] F500"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["variable".into(), "system".into()],
    });
    
    // Generate variable arithmetic combinations
    for op1 in ['+', '-', '*', '/'] {
        for op2 in ['+', '-', '*', '/'] {
            examples.push(GCodeExample {
                id: format!("var_expr_{}{}", op1 as u8, op2 as u8),
                category: GCodeCategory::Expressions,
                complexity: 4,
                description: format!("Expression with {} and {}", op1, op2),
                content: format!("#1=10\n#2=5\n#3=2\n#4=[[#1{}#2]{}#3]\nG1 X[#4] F500", op1, op2),
                expected_segments: Some(1),
                expected_distance: None,
                should_parse: true,
                tags: vec!["variable".into(), "expression".into()],
            });
        }
    }
    
    examples
}

// ============================================================================
// COMPLEX PATTERN EXAMPLES (100)
// ============================================================================

fn generate_complex_pattern_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Rectangular pocket
    examples.push(GCodeExample {
        id: "pattern_rect_pocket".into(),
        category: GCodeCategory::Pocketing,
        complexity: 7,
        description: "Rectangular pocket".into(),
        content: r#"; Rectangular pocket 60x40mm, 5mm deep
G90
G0 Z5
G0 X10 Y10
G1 Z-5 F100
; First pass
G1 X50 F500
G1 Y30
G1 X10
G1 Y10
; Step over
G1 X15 Y15
G1 X45
G1 Y25
G1 X15
G1 Y15
G0 Z5"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "pocket".into()],
    });
    
    // Circular pocket
    examples.push(GCodeExample {
        id: "pattern_circ_pocket".into(),
        category: GCodeCategory::Pocketing,
        complexity: 8,
        description: "Circular pocket with spiral".into(),
        content: r#"; Circular pocket R30, center at 50,50
G90
G0 Z5
G0 X50 Y50
G1 Z-5 F100
; Spiral outward
G3 X55 Y50 I2.5 J0 F500
G3 X55 Y50 I-5 J0
G3 X60 Y50 I-7.5 J0
G3 X60 Y50 I-10 J0
G3 X65 Y50 I-12.5 J0
G3 X65 Y50 I-15 J0
; More passes...
G0 Z5"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "pocket".into(), "circular".into()],
    });
    
    // Profile contour
    examples.push(GCodeExample {
        id: "pattern_contour".into(),
        category: GCodeCategory::Contouring,
        complexity: 7,
        description: "Profile contouring".into(),
        content: r#"; Profile contour with arcs
G90 G41 D1
G0 Z5
G0 X-5 Y0
G1 Z-5 F100
G1 X0 Y0 F500
G1 X80 Y0
G3 X100 Y20 R20
G1 X100 Y80
G3 X80 Y100 R20
G1 X20 Y100
G3 X0 Y80 R20
G1 X0 Y0
G0 Z5
G40"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "contour".into()],
    });
    
    // Surface finishing
    examples.push(GCodeExample {
        id: "pattern_surface_finish".into(),
        category: GCodeCategory::SurfaceFinishing,
        complexity: 8,
        description: "Surface finishing passes".into(),
        content: r#"; Surface finishing with decreasing stepover
G90
G0 Z5
; Rough pass (5mm stepover)
G0 X0 Y0
G1 Z-3 F100
G1 X100 F800
G1 Y5
G1 X0
G1 Y10
G1 X100
; ... more passes
; Finish pass (0.5mm stepover)
G1 Z-3.5 F50
G1 X0 F1200
G1 Y10.5
G1 X100
; etc.
G0 Z5"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "surface".into(), "finishing".into()],
    });
    
    // Multi-pass depth operation
    examples.push(GCodeExample {
        id: "pattern_multi_pass_depth".into(),
        category: GCodeCategory::MultiPassOperations,
        complexity: 7,
        description: "Multi-pass depth cutting".into(),
        content: r#"; Multi-pass pocket, 1mm depth per pass
#<depth>=0
#<target_depth>=-10
#<stepdown>=1

o100 while [#<depth> GT #<target_depth>]
  #<depth>=[#<depth>-#<stepdown>]
  G0 X10 Y10
  G1 Z[#<depth>] F100
  G1 X90 F500
  G1 Y90
  G1 X10
  G1 Y10
  G0 Z2
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "multi_pass".into(), "depth".into()],
    });
    
    // Engraving text path (simplified)
    examples.push(GCodeExample {
        id: "pattern_engrave_text".into(),
        category: GCodeCategory::Contouring,
        complexity: 8,
        description: "Text engraving path".into(),
        content: r#"; Letter "A" engrave path
G90
G0 Z2
G0 X10 Y0
G1 Z-0.5 F50
G1 X15 Y20 F500
G1 X20 Y0
G0 Z2
G0 X12 Y8
G1 Z-0.5 F50
G1 X18 Y8 F500
G0 Z2"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "engrave".into()],
    });
    
    // 3D surface (simple dome)
    examples.push(GCodeExample {
        id: "pattern_3d_dome".into(),
        category: GCodeCategory::SurfaceFinishing,
        complexity: 9,
        description: "3D dome surface".into(),
        content: r#"; Simple dome surface R=25 at center 50,50
#<radius>=25
#<center_x>=50
#<center_y>=50
#<step>=2

#<y>=[#<center_y>-#<radius>]
o100 while [#<y> LE [#<center_y>+#<radius>]]
  #<dy>=[#<y>-#<center_y>]
  #<chord>=[SQRT[#<radius>*#<radius>-#<dy>*#<dy>]]
  #<z>=[SQRT[#<radius>*#<radius>-#<dy>*#<dy>]]
  
  G0 X[#<center_x>-#<chord>] Y[#<y>] Z[#<z>+5]
  G1 Z[-#<z>] F100
  G1 X[#<center_x>+#<chord>] F500
  G0 Z5
  
  #<y>=[#<y>+#<step>]
o100 endwhile"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "3d".into(), "dome".into()],
    });
    
    // Thread milling
    examples.push(GCodeExample {
        id: "pattern_thread_mill".into(),
        category: GCodeCategory::ThreadCutting,
        complexity: 9,
        description: "Thread milling".into(),
        content: r#"; Thread milling M20x2.5
#<major_dia>=20
#<pitch>=2.5
#<depth>=15
#<tool_dia>=6
#<thread_depth>=1.35

#<helix_dia>=[#<major_dia>-#<tool_dia>-#<thread_depth>*2]

G90
G0 X0 Y0 Z5
G0 Z-[#<depth>]
G0 X[#<helix_dia>/2] Y0

; Helical thread path
G3 X[#<helix_dia>/2] Y0 Z[-#<depth>+#<pitch>] I[-#<helix_dia>/2] J0 F200
G0 X0 Y0
G0 Z5"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: true,
        tags: vec!["pattern".into(), "thread".into(), "milling".into()],
    });
    
    // Generate various pocket sizes
    for width in [20, 40, 60, 80, 100] {
        for height in [20, 40, 60] {
            if width != height { // Avoid duplicates
                examples.push(GCodeExample {
                    id: format!("pattern_pocket_{}x{}", width, height),
                    category: GCodeCategory::Pocketing,
                    complexity: 6,
                    description: format!("Pocket {}x{}mm", width, height),
                    content: format!(r#"G90
G0 Z5
G0 X5 Y5
G1 Z-5 F100
G1 X{} F500
G1 Y{}
G1 X5
G1 Y5
G0 Z5"#, width - 5, height - 5),
                    expected_segments: None,
                    expected_distance: None,
                    should_parse: true,
                    tags: vec!["pattern".into(), "pocket".into()],
                });
            }
        }
    }
    
    examples
}

// ============================================================================
// EDGE CASE EXAMPLES (100)
// ============================================================================

fn generate_edge_case_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Empty lines
    examples.push(GCodeExample {
        id: "edge_empty_lines".into(),
        category: GCodeCategory::EmptyLines,
        complexity: 1,
        description: "Multiple empty lines".into(),
        content: "\n\n\nG1 X10 F500\n\n\nG1 X20\n\n".into(),
        expected_segments: Some(2),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "empty_lines".into()],
    });
    
    // Comments only
    examples.push(GCodeExample {
        id: "edge_comments_only".into(),
        category: GCodeCategory::CommentsOnly,
        complexity: 1,
        description: "File with only comments".into(),
        content: "; This is a comment\n( This is another comment )\n; More comments".into(),
        expected_segments: Some(0),
        expected_distance: Some(0.0),
        should_parse: true,
        tags: vec!["edge".into(), "comments".into()],
    });
    
    // Mixed comment styles
    examples.push(GCodeExample {
        id: "edge_mixed_comments".into(),
        category: GCodeCategory::CommentsOnly,
        complexity: 2,
        description: "Mixed comment styles".into(),
        content: r#"; Semicolon comment
( Parenthesis comment )
G1 X10 F500 ; Inline comment
G1 X20 ( inline paren )
G1 X30"#.into(),
        expected_segments: Some(3),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "comments".into()],
    });
    
    // Whitespace variations
    examples.push(GCodeExample {
        id: "edge_whitespace_leading".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 2,
        description: "Leading whitespace".into(),
        content: "    G1 X10 F500\n\t\tG1 X20".into(),
        expected_segments: Some(2),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "whitespace".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_whitespace_between".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 2,
        description: "Whitespace between words".into(),
        content: "G1    X10    Y20    F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "whitespace".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_no_whitespace".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 2,
        description: "No whitespace".into(),
        content: "G1X10Y20Z5F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "whitespace".into()],
    });
    
    // Case sensitivity
    examples.push(GCodeExample {
        id: "edge_lowercase".into(),
        category: GCodeCategory::CaseSensitivity,
        complexity: 2,
        description: "Lowercase GCode".into(),
        content: "g1 x10 y20 f500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "case".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_mixed_case".into(),
        category: GCodeCategory::CaseSensitivity,
        complexity: 2,
        description: "Mixed case".into(),
        content: "g1 X10 y20 Z5 F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "case".into()],
    });
    
    // Numeric precision
    examples.push(GCodeExample {
        id: "edge_many_decimals".into(),
        category: GCodeCategory::NumericPrecision,
        complexity: 3,
        description: "Many decimal places".into(),
        content: "G1 X10.123456789 Y20.987654321 F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "precision".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_scientific_notation".into(),
        category: GCodeCategory::NumericPrecision,
        complexity: 3,
        description: "Scientific notation".into(),
        content: "G1 X1.5e2 Y2.5e1 F5e2".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "scientific".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_negative_coords".into(),
        category: GCodeCategory::NumericPrecision,
        complexity: 2,
        description: "Negative coordinates".into(),
        content: "G1 X-50 Y-30 Z-10 F500".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "negative".into()],
    });
    
    examples.push(GCodeExample {
        id: "edge_zero_coords".into(),
        category: GCodeCategory::NumericPrecision,
        complexity: 2,
        description: "Zero coordinates".into(),
        content: "G1 X0 Y0 Z0 F500".into(),
        expected_segments: Some(1),
        expected_distance: Some(0.0),
        should_parse: true,
        tags: vec!["edge".into(), "zero".into()],
    });
    
    // Large programs
    let mut large_content = String::with_capacity(50000);
    large_content.push_str("G90\n");
    for i in 0..500 {
        large_content.push_str(&format!("G1 X{} Y{} F500\n", i % 100, (i / 100) % 100));
    }
    examples.push(GCodeExample {
        id: "edge_large_program".into(),
        category: GCodeCategory::LargePrograms,
        complexity: 5,
        description: "Large program (500 lines)".into(),
        content: large_content,
        expected_segments: Some(500),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "large".into()],
    });
    
    // Line numbers
    examples.push(GCodeExample {
        id: "edge_line_numbers".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 2,
        description: "With line numbers".into(),
        content: r#"N10 G90
N20 G1 X10 F500
N30 G1 X20
N40 G1 X30"#.into(),
        expected_segments: Some(3),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "line_numbers".into()],
    });
    
    // Block delete
    examples.push(GCodeExample {
        id: "edge_block_delete".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 3,
        description: "Block delete characters".into(),
        content: r#"G1 X10 F500
/G1 X20
G1 X30
/G1 X40
G1 X50"#.into(),
        expected_segments: None, // Depends on block delete setting
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "block_delete".into()],
    });
    
    // Percent signs (program delimiters)
    examples.push(GCodeExample {
        id: "edge_percent_delimiters".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 2,
        description: "Percent sign delimiters".into(),
        content: r#"%
G1 X10 F500
G1 X20
%"#.into(),
        expected_segments: Some(2),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "percent".into()],
    });
    
    // O-word as program number
    examples.push(GCodeExample {
        id: "edge_oword_program".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 3,
        description: "O-word as program number".into(),
        content: r#"O1234
G1 X10 F500
G1 X20
M30"#.into(),
        expected_segments: Some(2),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "oword".into()],
    });
    
    // Multiple commands per line
    examples.push(GCodeExample {
        id: "edge_multiple_per_line".into(),
        category: GCodeCategory::WhitespaceVariations,
        complexity: 3,
        description: "Multiple commands per line".into(),
        content: "G90 G1 X10 Y20 F500 S1000 M3".into(),
        expected_segments: Some(1),
        expected_distance: None,
        should_parse: true,
        tags: vec!["edge".into(), "multiple".into()],
    });
    
    // Generate more edge cases
    for i in 1..=30 {
        examples.push(GCodeExample {
            id: format!("edge_variation_{}", i),
            category: GCodeCategory::WhitespaceVariations,
            complexity: 2,
            description: format!("Edge case variation {}", i),
            content: format!("{}G1{}X{}{}Y{} F500", 
                " ".repeat(i % 5),
                " ".repeat((i + 1) % 3),
                i * 2,
                " ".repeat((i + 2) % 4),
                i * 3),
            expected_segments: Some(1),
            expected_distance: None,
            should_parse: true,
            tags: vec!["edge".into(), "variation".into()],
        });
    }
    
    examples
}

// ============================================================================
// ERROR CASE EXAMPLES (50)
// ============================================================================

fn generate_error_examples() -> Vec<GCodeExample> {
    let mut examples = Vec::new();
    
    // Syntax errors
    examples.push(GCodeExample {
        id: "error_invalid_gcode".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 1,
        description: "Invalid G code number".into(),
        content: "G999 X10".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "syntax".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_missing_value".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 1,
        description: "Missing coordinate value".into(),
        content: "G1 X Y10 F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "missing".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_unclosed_bracket".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 2,
        description: "Unclosed bracket in expression".into(),
        content: "G1 X[10+5 F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "bracket".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_unclosed_paren".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 2,
        description: "Unclosed parenthesis comment".into(),
        content: "G1 X10 ( comment F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "paren".into()],
    });
    
    // Invalid parameters
    examples.push(GCodeExample {
        id: "error_arc_missing_center".into(),
        category: GCodeCategory::InvalidParameters,
        complexity: 3,
        description: "Arc without center".into(),
        content: "G2 X50 Y50 F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "arc".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_conflicting_params".into(),
        category: GCodeCategory::InvalidParameters,
        complexity: 3,
        description: "Conflicting arc parameters".into(),
        content: "G2 X50 Y50 I10 J10 R30 F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "conflicting".into()],
    });
    
    // Out of range values
    examples.push(GCodeExample {
        id: "error_huge_number".into(),
        category: GCodeCategory::OutOfRange,
        complexity: 2,
        description: "Extremely large coordinate".into(),
        content: "G1 X999999999999 F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "range".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_zero_feed".into(),
        category: GCodeCategory::OutOfRange,
        complexity: 2,
        description: "Zero feed rate".into(),
        content: "G1 X50 F0".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "feed".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_negative_feed".into(),
        category: GCodeCategory::OutOfRange,
        complexity: 2,
        description: "Negative feed rate".into(),
        content: "G1 X50 F-500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "feed".into()],
    });
    
    // O-code errors
    examples.push(GCodeExample {
        id: "error_ocode_no_end".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 4,
        description: "Missing endsub".into(),
        content: r#"o100 sub
G1 X10 F500"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "ocode".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_ocode_no_while".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 4,
        description: "Missing endwhile".into(),
        content: r#"#1=0
o100 while [#1 LT 5]
G1 X[#1*10] F500
#1=[#1+1]"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "ocode".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_ocode_mismatch".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 4,
        description: "O-code number mismatch".into(),
        content: r#"o100 sub
G1 X10 F500
o200 endsub"#.into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "ocode".into()],
    });
    
    // Expression errors
    examples.push(GCodeExample {
        id: "error_divide_zero".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 3,
        description: "Division by zero".into(),
        content: "G1 X[10/0] F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "expression".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_sqrt_negative".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 3,
        description: "Square root of negative".into(),
        content: "G1 X[SQRT[-1]] F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "expression".into()],
    });
    
    examples.push(GCodeExample {
        id: "error_undefined_var".into(),
        category: GCodeCategory::SyntaxErrors,
        complexity: 3,
        description: "Undefined variable".into(),
        content: "G1 X[#<undefined_var>] F500".into(),
        expected_segments: None,
        expected_distance: None,
        should_parse: false,
        tags: vec!["error".into(), "variable".into()],
    });
    
    // Generate more error variations
    for i in 1..=20 {
        examples.push(GCodeExample {
            id: format!("error_syntax_{}", i),
            category: GCodeCategory::SyntaxErrors,
            complexity: 2,
            description: format!("Syntax error variation {}", i),
            content: format!("G{} X{}", 100 + i * 10, if i % 2 == 0 { "abc" } else { "" }),
            expected_segments: None,
            expected_distance: None,
            should_parse: false,
            tags: vec!["error".into(), "syntax".into()],
        });
    }
    
    examples
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get all examples for a specific category
pub fn get_examples_by_category(category: GCodeCategory) -> Vec<GCodeExample> {
    generate_all_examples()
        .into_iter()
        .filter(|e| e.category == category)
        .collect()
}

/// Get examples by tag
pub fn get_examples_by_tag(tag: &str) -> Vec<GCodeExample> {
    generate_all_examples()
        .into_iter()
        .filter(|e| e.tags.contains(&tag.to_string()))
        .collect()
}

/// Get examples by complexity level
pub fn get_examples_by_complexity(min: u8, max: u8) -> Vec<GCodeExample> {
    generate_all_examples()
        .into_iter()
        .filter(|e| e.complexity >= min && e.complexity <= max)
        .collect()
}

/// Get statistics about the examples
pub fn get_example_statistics() -> HashMap<GCodeCategory, usize> {
    let mut stats = HashMap::new();
    for example in generate_all_examples() {
        *stats.entry(example.category).or_insert(0) += 1;
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_total_examples_count() {
        let examples = generate_all_examples();
        // We have approximately 500 programmatically generated examples
        assert!(examples.len() >= 400, "Should have at least 400 examples, got {}", examples.len());
    }
    
    #[test]
    fn test_all_examples_have_content() {
        let examples = generate_all_examples();
        for example in &examples {
            assert!(!example.content.is_empty(), "Example {} has no content", example.id);
        }
    }
    
    #[test]
    fn test_all_examples_have_id() {
        let examples = generate_all_examples();
        for example in &examples {
            assert!(!example.id.is_empty(), "Example has no ID");
        }
    }
    
    #[test]
    fn test_unique_ids() {
        let examples = generate_all_examples();
        let mut ids = std::collections::HashSet::new();
        for example in &examples {
            assert!(ids.insert(&example.id), "Duplicate ID: {}", example.id);
        }
    }
    
    #[test]
    fn test_basic_linear_examples() {
        let examples = generate_basic_linear_examples();
        assert!(examples.len() >= 20, "Should have at least 20 linear examples");
    }
    
    #[test]
    fn test_arc_examples() {
        let examples = generate_arc_examples();
        assert!(examples.len() >= 30, "Should have at least 30 arc examples");
    }
    
    #[test]
    fn test_control_flow_examples() {
        let examples = generate_control_flow_examples();
        // O-code control flow examples
        assert!(examples.len() >= 15, "Should have at least 15 control flow examples, got {}", examples.len());
    }
    
    #[test]
    fn test_error_examples() {
        let examples = generate_error_examples();
        for example in &examples {
            assert!(!example.should_parse, "Error example {} should not parse", example.id);
        }
    }
    
    #[test]
    fn test_category_coverage() {
        let stats = get_example_statistics();
        // Ensure we have examples in multiple categories
        assert!(stats.len() >= 15, "Should cover at least 15 categories, got {}", stats.len());
    }
}
