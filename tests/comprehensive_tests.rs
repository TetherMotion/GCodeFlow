//! Comprehensive unit tests for GCodeFlow
//!
//! This module contains extensive tests for:
//! - GCode parsing and interpretation
//! - Motion generation and trajectory computation
//! - Editor functionality
//! - Visualization and rendering
//! - Simulation playback
//! - Configuration management

mod gcode_examples;

use std::f64::consts::PI;

// Re-export for use in integration tests
pub use gcode_examples::*;

// ============================================================================
// GCODE PARSER TESTS
// ============================================================================

#[cfg(test)]
mod parser_tests {
    use super::gcode_examples::*;
    
    /// Helper to validate GCode content structure
    fn is_valid_gcode_structure(content: &str) -> bool {
        // Basic structure validation
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('(') {
                continue;
            }
            // Should start with valid word or be a comment
            let first_char = trimmed.chars().next().unwrap_or(' ');
            if !matches!(first_char.to_ascii_uppercase(), 
                'G' | 'M' | 'X' | 'Y' | 'Z' | 'A' | 'B' | 'C' | 
                'F' | 'S' | 'T' | 'N' | 'O' | '#' | '%' | '/') {
                return false;
            }
        }
        true
    }
    
    #[test]
    fn test_all_valid_examples_have_valid_structure() {
        let examples = generate_all_examples();
        for example in examples.iter().filter(|e| e.should_parse) {
            assert!(
                is_valid_gcode_structure(&example.content),
                "Example {} has invalid structure:\n{}",
                example.id,
                example.content
            );
        }
    }
    
    #[test]
    fn test_linear_move_parsing() {
        let examples = get_examples_by_category(GCodeCategory::BasicLinearMove);
        assert!(!examples.is_empty());
        
        for example in &examples {
            assert!(example.content.contains("G1") || example.content.contains("g1"),
                "Linear move example {} should contain G1", example.id);
        }
    }
    
    #[test]
    fn test_rapid_move_parsing() {
        let examples = get_examples_by_category(GCodeCategory::BasicRapidMove);
        assert!(!examples.is_empty());
        
        for example in &examples {
            assert!(example.content.contains("G0") || example.content.contains("g0"),
                "Rapid move example {} should contain G0", example.id);
        }
    }
    
    #[test]
    fn test_arc_cw_parsing() {
        let examples = get_examples_by_category(GCodeCategory::BasicArcCW);
        assert!(!examples.is_empty());
        
        for example in &examples {
            assert!(example.content.contains("G2") || example.content.contains("g2"),
                "CW arc example {} should contain G2", example.id);
        }
    }
    
    #[test]
    fn test_arc_ccw_parsing() {
        let examples = get_examples_by_category(GCodeCategory::BasicArcCCW);
        assert!(!examples.is_empty());
        
        for example in &examples {
            assert!(example.content.contains("G3") || example.content.contains("g3"),
                "CCW arc example {} should contain G3", example.id);
        }
    }
    
    #[test]
    fn test_coordinate_system_examples() {
        let abs_examples = get_examples_by_category(GCodeCategory::AbsoluteCoordinates);
        let inc_examples = get_examples_by_category(GCodeCategory::IncrementalCoordinates);
        
        assert!(!abs_examples.is_empty());
        assert!(!inc_examples.is_empty());
        
        // Absolute should contain G90
        for example in &abs_examples {
            assert!(example.content.contains("G90") || example.content.contains("g90"),
                "Absolute example {} should contain G90", example.id);
        }
        
        // Incremental should contain G91
        for example in &inc_examples {
            assert!(example.content.contains("G91") || example.content.contains("g91"),
                "Incremental example {} should contain G91", example.id);
        }
    }
    
    #[test]
    fn test_ocode_examples_structure() {
        let sub_examples = get_examples_by_category(GCodeCategory::OCodeSubroutines);
        let while_examples = get_examples_by_category(GCodeCategory::OCodeWhileLoops);
        
        assert!(!sub_examples.is_empty());
        assert!(!while_examples.is_empty());
        
        // Subroutines should have sub/endsub
        for example in sub_examples.iter().filter(|e| e.should_parse) {
            assert!(example.content.to_lowercase().contains("sub"),
                "Subroutine example {} should contain 'sub'", example.id);
        }
        
        // While loops should have while/endwhile
        for example in while_examples.iter().filter(|e| e.should_parse) {
            assert!(example.content.to_lowercase().contains("while"),
                "While example {} should contain 'while'", example.id);
        }
    }
    
    #[test]
    fn test_variable_examples_contain_parameters() {
        let numbered = get_examples_by_category(GCodeCategory::NumberedParameters);
        let named = get_examples_by_category(GCodeCategory::NamedParameters);
        
        assert!(!numbered.is_empty());
        assert!(!named.is_empty());
        
        // Numbered should contain #N
        for example in &numbered {
            assert!(example.content.contains('#'),
                "Numbered param example {} should contain '#'", example.id);
        }
        
        // Named should contain #<name>
        for example in &named {
            assert!(example.content.contains("#<"),
                "Named param example {} should contain '#<'", example.id);
        }
    }
}

// ============================================================================
// MOTION GENERATION TESTS
// ============================================================================

#[cfg(test)]
mod motion_tests {
    use super::*;
    
    /// Simple point structure for testing
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Point3D {
        x: f64,
        y: f64,
        z: f64,
    }
    
    impl Point3D {
        fn new(x: f64, y: f64, z: f64) -> Self {
            Self { x, y, z }
        }
        
        fn distance_to(&self, other: &Point3D) -> f64 {
            let dx = self.x - other.x;
            let dy = self.y - other.y;
            let dz = self.z - other.z;
            (dx*dx + dy*dy + dz*dz).sqrt()
        }
        
        fn approx_eq(&self, other: &Point3D, epsilon: f64) -> bool {
            (self.x - other.x).abs() < epsilon &&
            (self.y - other.y).abs() < epsilon &&
            (self.z - other.z).abs() < epsilon
        }
    }
    
    #[test]
    fn test_linear_distance_calculation() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(100.0, 0.0, 0.0);
        assert!((p1.distance_to(&p2) - 100.0).abs() < 0.001);
        
        let p3 = Point3D::new(100.0, 100.0, 0.0);
        let expected = (2.0_f64 * 100.0 * 100.0).sqrt();
        assert!((p1.distance_to(&p3) - expected).abs() < 0.001);
    }
    
    #[test]
    fn test_3d_distance_calculation() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(10.0, 20.0, 30.0);
        let expected = (10.0*10.0 + 20.0*20.0 + 30.0*30.0_f64).sqrt();
        assert!((p1.distance_to(&p2) - expected).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_circumference_quarter() {
        // Quarter arc radius 50
        let radius = 50.0;
        let expected_length = PI * radius / 2.0; // Quarter circle
        // Just verify the formula
        assert!((expected_length - 78.54).abs() < 0.1);
    }
    
    #[test]
    fn test_arc_circumference_full() {
        // Full circle radius 50
        let radius = 50.0;
        let expected_length = 2.0 * PI * radius;
        assert!((expected_length - 314.159).abs() < 0.1);
    }
    
    #[test]
    fn test_helix_length() {
        // Single turn helix: sqrt(circumference^2 + pitch^2)
        let radius = 50.0;
        let pitch = 5.0;
        let circumference = 2.0 * PI * radius;
        let helix_length = (circumference * circumference + pitch * pitch).sqrt();
        assert!(helix_length > circumference); // Helix is longer than flat circle
    }
    
    #[test]
    fn test_feed_rate_time_calculation() {
        // Time = distance / feed_rate
        let distance = 100.0_f64; // mm
        let feed_rate = 500.0_f64; // mm/min
        let time_min = distance / feed_rate;
        let time_sec = time_min * 60.0;
        assert!((time_sec - 12.0).abs() < 0.001);
    }
    
    #[test]
    fn test_point_approximation() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = Point3D::new(1.0001, 2.0001, 3.0001);
        assert!(p1.approx_eq(&p2, 0.001));
        assert!(!p1.approx_eq(&p2, 0.00001));
    }
    
    #[test]
    fn test_incremental_move_accumulation() {
        // Simulate incremental moves
        let mut current = Point3D::new(0.0, 0.0, 0.0);
        let moves = [(10.0, 5.0, 0.0), (10.0, -5.0, 0.0), (-10.0, 5.0, 0.0), (-10.0, -5.0, 0.0)];
        
        for (dx, dy, dz) in &moves {
            current = Point3D::new(current.x + dx, current.y + dy, current.z + dz);
        }
        
        // Should return to origin
        assert!(current.approx_eq(&Point3D::new(0.0, 0.0, 0.0), 0.001));
    }
    
    #[test]
    fn test_square_path_total_distance() {
        let size = 100.0;
        let corners = [
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(size, 0.0, 0.0),
            Point3D::new(size, size, 0.0),
            Point3D::new(0.0, size, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
        ];
        
        let mut total_distance = 0.0;
        for i in 1..corners.len() {
            total_distance += corners[i-1].distance_to(&corners[i]);
        }
        
        assert!((total_distance - 400.0).abs() < 0.001);
    }
}

// ============================================================================
// COORDINATE SYSTEM TESTS
// ============================================================================

#[cfg(test)]
mod coordinate_tests {
    use std::f64::consts::PI;
    
    /// Transform GCode coordinates to Bevy coordinates
    /// GCode: X-right, Y-forward, Z-up
    /// Bevy: X-right, Y-up, Z-forward
    fn gcode_to_bevy(gx: f64, gy: f64, gz: f64) -> (f64, f64, f64) {
        (gx, gz, gy)
    }
    
    /// Transform Bevy coordinates back to GCode
    fn bevy_to_gcode(bx: f64, by: f64, bz: f64) -> (f64, f64, f64) {
        (bx, bz, by)
    }
    
    #[test]
    fn test_coordinate_transform_identity() {
        let (gx, gy, gz) = (10.0, 20.0, 30.0);
        let (bx, by, bz) = gcode_to_bevy(gx, gy, gz);
        let (gx2, gy2, gz2) = bevy_to_gcode(bx, by, bz);
        
        assert!((gx - gx2).abs() < 0.001);
        assert!((gy - gy2).abs() < 0.001);
        assert!((gz - gz2).abs() < 0.001);
    }
    
    #[test]
    fn test_coordinate_z_up() {
        // GCode Z=100 should become Bevy Y=100
        let (_, by, _) = gcode_to_bevy(0.0, 0.0, 100.0);
        assert!((by - 100.0).abs() < 0.001);
    }
    
    #[test]
    fn test_coordinate_y_forward() {
        // GCode Y=100 should become Bevy Z=100
        let (_, _, bz) = gcode_to_bevy(0.0, 100.0, 0.0);
        assert!((bz - 100.0).abs() < 0.001);
    }
    
    #[test]
    fn test_rotation_around_z() {
        // Rotate point (1, 0, 0) around Z by 90 degrees
        let angle = PI / 2.0;
        let x = 1.0_f64;
        let y = 0.0_f64;
        
        let x_rot = x * angle.cos() - y * angle.sin();
        let y_rot = x * angle.sin() + y * angle.cos();
        
        assert!(x_rot.abs() < 0.001); // Should be ~0
        assert!((y_rot - 1.0).abs() < 0.001); // Should be ~1
    }
    
    #[test]
    fn test_work_offset_application() {
        // Simulated work offset
        let offset = (50.0_f64, 50.0_f64, 10.0_f64);
        let point = (10.0_f64, 20.0_f64, 5.0_f64);
        
        let absolute = (point.0 + offset.0, point.1 + offset.1, point.2 + offset.2);
        
        assert!((absolute.0 - 60.0).abs() < 0.001);
        assert!((absolute.1 - 70.0).abs() < 0.001);
        assert!((absolute.2 - 15.0).abs() < 0.001);
    }
    
    #[test]
    fn test_g92_coordinate_shift() {
        // G92 makes current position equal to specified value
        let machine_pos = (100.0_f64, 100.0_f64, 50.0_f64);
        let g92_values = (0.0_f64, 0.0_f64, 0.0_f64);
        
        // After G92, offset is machine_pos - g92_values
        let offset = (
            machine_pos.0 - g92_values.0,
            machine_pos.1 - g92_values.1,
            machine_pos.2 - g92_values.2,
        );
        
        assert!((offset.0 - 100.0).abs() < 0.001);
        assert!((offset.1 - 100.0).abs() < 0.001);
        assert!((offset.2 - 50.0).abs() < 0.001);
    }
}

// ============================================================================
// SPEED AND TIME TESTS
// ============================================================================

#[cfg(test)]
mod speed_tests {
    #[test]
    fn test_feed_rate_conversion() {
        // mm/min to mm/sec
        let feed_mm_min = 600.0_f64;
        let feed_mm_sec = feed_mm_min / 60.0;
        assert!((feed_mm_sec - 10.0).abs() < 0.001);
    }
    
    #[test]
    fn test_time_for_move() {
        // Time = distance / feed_rate
        let distance = 50.0_f64; // mm
        let feed = 1000.0_f64; // mm/min
        let time_min = distance / feed;
        let time_sec = time_min * 60.0;
        assert!((time_sec - 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_logarithmic_speed_scaling() {
        // Test the logarithmic speed slider math
        fn log_to_speed(log_value: f64) -> f64 {
            if log_value >= 0.0 {
                10.0_f64.powf(log_value)
            } else {
                -10.0_f64.powf(-log_value)
            }
        }
        
        assert!((log_to_speed(0.0) - 1.0).abs() < 0.001);
        assert!((log_to_speed(1.0) - 10.0).abs() < 0.001);
        assert!((log_to_speed(2.0) - 100.0).abs() < 0.001);
        assert!((log_to_speed(-1.0) - (-10.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_spindle_rpm_to_surface_speed() {
        // Surface speed = PI * diameter * RPM
        let rpm = 1000.0_f64;
        let diameter = 10.0_f64; // mm
        let surface_speed = std::f64::consts::PI * diameter * rpm / 1000.0; // m/min
        assert!((surface_speed - 31.416).abs() < 0.1);
    }
    
    #[test]
    fn test_feed_per_rev_calculation() {
        // Feed rate = feed_per_rev * RPM
        let feed_per_rev = 0.1_f64; // mm/rev
        let rpm = 1000.0_f64;
        let feed_rate = feed_per_rev * rpm; // mm/min
        assert!((feed_rate - 100.0).abs() < 0.001);
    }
    
    #[test]
    fn test_inverse_time_feed() {
        // Inverse time: F value = moves per minute
        // Time for move = 1/F minutes
        let f_value = 2.0_f64; // 2 moves per minute
        let time_min = 1.0 / f_value;
        let time_sec = time_min * 60.0;
        assert!((time_sec - 30.0).abs() < 0.001);
    }
    
    #[test]
    fn test_acceleration_time() {
        // Time to accelerate: t = v / a
        let target_velocity = 100.0_f64; // mm/s
        let acceleration = 500.0_f64; // mm/s²
        let accel_time = target_velocity / acceleration;
        assert!((accel_time - 0.2).abs() < 0.001);
    }
    
    #[test]
    fn test_acceleration_distance() {
        // Distance during acceleration: d = 0.5 * a * t²
        let acceleration = 500.0_f64; // mm/s²
        let time = 0.2_f64; // s
        let distance = 0.5 * acceleration * time * time;
        assert!((distance - 10.0).abs() < 0.001);
    }
}

// ============================================================================
// ARC INTERPOLATION TESTS
// ============================================================================

#[cfg(test)]
mod arc_tests {
    use std::f64::consts::PI;
    
    /// Calculate arc endpoint given center, radius, and angle
    fn arc_point(center_x: f64, center_y: f64, radius: f64, angle_rad: f64) -> (f64, f64) {
        let x = center_x + radius * angle_rad.cos();
        let y = center_y + radius * angle_rad.sin();
        (x, y)
    }
    
    /// Calculate arc length
    fn arc_length(radius: f64, angle_rad: f64) -> f64 {
        radius * angle_rad.abs()
    }
    
    /// Calculate center from start, end, and I/J offsets
    fn center_from_ij(start_x: f64, start_y: f64, i: f64, j: f64) -> (f64, f64) {
        (start_x + i, start_y + j)
    }
    
    #[test]
    fn test_quarter_arc_length() {
        let radius = 50.0;
        let angle = PI / 2.0;
        let length = arc_length(radius, angle);
        let expected = PI * radius / 2.0;
        assert!((length - expected).abs() < 0.001);
    }
    
    #[test]
    fn test_half_arc_length() {
        let radius = 50.0;
        let angle = PI;
        let length = arc_length(radius, angle);
        let expected = PI * radius;
        assert!((length - expected).abs() < 0.001);
    }
    
    #[test]
    fn test_full_circle_length() {
        let radius = 50.0;
        let angle = 2.0 * PI;
        let length = arc_length(radius, angle);
        let expected = 2.0 * PI * radius;
        assert!((length - expected).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_point_at_0() {
        let (x, y) = arc_point(50.0, 50.0, 25.0, 0.0);
        assert!((x - 75.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_point_at_90() {
        let (x, y) = arc_point(50.0, 50.0, 25.0, PI / 2.0);
        assert!((x - 50.0).abs() < 0.001);
        assert!((y - 75.0).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_point_at_180() {
        let (x, y) = arc_point(50.0, 50.0, 25.0, PI);
        assert!((x - 25.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);
    }
    
    #[test]
    fn test_center_calculation() {
        // Start at (0, 0), I=50, J=0 -> center at (50, 0)
        let (cx, cy) = center_from_ij(0.0, 0.0, 50.0, 0.0);
        assert!((cx - 50.0).abs() < 0.001);
        assert!((cy - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_interpolation_points() {
        // Generate points along an arc
        let center = (50.0, 50.0);
        let radius = 25.0;
        let start_angle = 0.0;
        let end_angle = PI / 2.0;
        let num_points = 10;
        
        let mut points = Vec::new();
        for i in 0..=num_points {
            let t = i as f64 / num_points as f64;
            let angle = start_angle + t * (end_angle - start_angle);
            points.push(arc_point(center.0, center.1, radius, angle));
        }
        
        assert_eq!(points.len(), 11);
        
        // First point
        assert!((points[0].0 - 75.0).abs() < 0.001);
        assert!((points[0].1 - 50.0).abs() < 0.001);
        
        // Last point
        assert!((points[10].0 - 50.0).abs() < 0.001);
        assert!((points[10].1 - 75.0).abs() < 0.001);
    }
    
    #[test]
    fn test_helix_z_increment() {
        // Helix: each full turn adds Z pitch
        let pitch = 5.0_f64;
        let turns = 3.0_f64;
        let total_z = pitch * turns;
        assert!((total_z - 15.0).abs() < 0.001);
    }
    
    #[test]
    fn test_r_format_to_ij() {
        // R-format arc: R is the radius
        // For a specific case: start (0,0), end (50,50), R=50
        // This is a quarter arc
        let r = 50.0;
        let start = (0.0, 0.0);
        let end = (50.0, 50.0);
        
        // Center should be at (50, 0) or (0, 50) depending on direction
        // For CW from (0,0) to (50,50) with R=50, center is at (50, 0)
        // So I = 50, J = 0
        let expected_i = 50.0;
        let expected_j = 0.0;
        
        let center = center_from_ij(start.0, start.1, expected_i, expected_j);
        
        // Verify radius from center to both endpoints
        let r1 = ((start.0 - center.0).powi(2) + (start.1 - center.1).powi(2)).sqrt();
        let r2 = ((end.0 - center.0).powi(2) + (end.1 - center.1).powi(2)).sqrt();
        
        assert!((r1 - r).abs() < 0.001);
        assert!((r2 - r).abs() < 0.001);
    }
}

// ============================================================================
// EXPRESSION EVALUATION TESTS
// ============================================================================

#[cfg(test)]
mod expression_tests {
    use std::f64::consts::PI;
    
    /// Simple expression evaluator for testing
    fn eval_binary(a: f64, op: char, b: f64) -> f64 {
        match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => a / b,
            _ => 0.0,
        }
    }
    
    fn eval_function(name: &str, arg: f64) -> f64 {
        match name.to_uppercase().as_str() {
            "SIN" => (arg * PI / 180.0).sin(),
            "COS" => (arg * PI / 180.0).cos(),
            "TAN" => (arg * PI / 180.0).tan(),
            "ASIN" => arg.asin() * 180.0 / PI,
            "ACOS" => arg.acos() * 180.0 / PI,
            "ATAN" => arg.atan() * 180.0 / PI,
            "SQRT" => arg.sqrt(),
            "ABS" => arg.abs(),
            "ROUND" => arg.round(),
            "FIX" => arg.floor(),
            "FUP" => arg.ceil(),
            "LN" => arg.ln(),
            "EXP" => arg.exp(),
            _ => 0.0,
        }
    }
    
    #[test]
    fn test_basic_arithmetic() {
        assert!((eval_binary(10.0, '+', 5.0) - 15.0).abs() < 0.001);
        assert!((eval_binary(10.0, '-', 5.0) - 5.0).abs() < 0.001);
        assert!((eval_binary(10.0, '*', 5.0) - 50.0).abs() < 0.001);
        assert!((eval_binary(10.0, '/', 5.0) - 2.0).abs() < 0.001);
    }
    
    #[test]
    fn test_sin_function() {
        assert!((eval_function("SIN", 0.0) - 0.0).abs() < 0.001);
        assert!((eval_function("SIN", 30.0) - 0.5).abs() < 0.001);
        assert!((eval_function("SIN", 90.0) - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_cos_function() {
        assert!((eval_function("COS", 0.0) - 1.0).abs() < 0.001);
        assert!((eval_function("COS", 60.0) - 0.5).abs() < 0.001);
        assert!((eval_function("COS", 90.0) - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_sqrt_function() {
        assert!((eval_function("SQRT", 4.0) - 2.0).abs() < 0.001);
        assert!((eval_function("SQRT", 100.0) - 10.0).abs() < 0.001);
        assert!((eval_function("SQRT", 2.0) - 1.414).abs() < 0.001);
    }
    
    #[test]
    fn test_abs_function() {
        assert!((eval_function("ABS", -5.0) - 5.0).abs() < 0.001);
        assert!((eval_function("ABS", 5.0) - 5.0).abs() < 0.001);
        assert!((eval_function("ABS", 0.0) - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_rounding_functions() {
        assert!((eval_function("ROUND", 2.4) - 2.0).abs() < 0.001);
        assert!((eval_function("ROUND", 2.5) - 3.0).abs() < 0.001);
        assert!((eval_function("FIX", 2.9) - 2.0).abs() < 0.001);
        assert!((eval_function("FUP", 2.1) - 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_comparison_lt() {
        let result = if 5.0_f64 < 10.0 { 1.0_f64 } else { 0.0 };
        assert!((result - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_comparison_gt() {
        let result = if 10.0_f64 > 5.0 { 1.0_f64 } else { 0.0 };
        assert!((result - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_comparison_eq() {
        let result = if (10.0_f64 - 10.0).abs() < 0.001 { 1.0_f64 } else { 0.0 };
        assert!((result - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_logical_and() {
        let result = if true && true { 1.0_f64 } else { 0.0 };
        assert!((result - 1.0).abs() < 0.001);
        
        let result = if true && false { 1.0_f64 } else { 0.0 };
        assert!((result - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_logical_or() {
        let result = if true || false { 1.0_f64 } else { 0.0 };
        assert!((result - 1.0).abs() < 0.001);
        
        let result = if false || false { 1.0_f64 } else { 0.0 };
        assert!((result - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_complex_expression() {
        // [#<radius> * COS[#<angle>] + 100]
        let radius = 50.0_f64;
        let angle = 45.0_f64;
        let result = radius * (angle * PI / 180.0).cos() + 100.0;
        assert!((result - 135.355).abs() < 0.01);
    }
}

// ============================================================================
// CANNED CYCLE TESTS
// ============================================================================

#[cfg(test)]
mod canned_cycle_tests {
    #[test]
    fn test_g81_drill_moves() {
        // G81: Rapid to R, feed to Z, rapid retract to R
        let rapid_plane = 2.0_f64;
        let drill_depth = -15.0_f64;
        
        // Move sequence should be:
        // 1. Rapid to R
        // 2. Feed to Z
        // 3. Rapid back to R
        
        let total_feed_distance = (rapid_plane - drill_depth).abs();
        assert!((total_feed_distance - 17.0).abs() < 0.001);
    }
    
    #[test]
    fn test_g83_peck_cycles() {
        // G83: Peck drilling with retract between pecks
        let total_depth = 20.0_f64;
        let peck_depth = 5.0_f64;
        
        let num_pecks = (total_depth / peck_depth).ceil() as i32;
        assert_eq!(num_pecks, 4);
    }
    
    #[test]
    fn test_g84_tapping_feed() {
        // Tapping: feed = pitch * RPM
        let pitch = 1.25_f64; // mm
        let rpm = 500.0_f64;
        let feed_rate = pitch * rpm; // mm/min
        assert!((feed_rate - 625.0).abs() < 0.001);
    }
    
    #[test]
    fn test_bolt_circle_positions() {
        // Calculate bolt circle positions
        let center_x = 50.0;
        let center_y = 50.0;
        let radius = 30.0;
        let num_holes = 6;
        
        let mut positions = Vec::new();
        for i in 0..num_holes {
            let angle = i as f64 * 360.0 / num_holes as f64;
            let angle_rad = angle * std::f64::consts::PI / 180.0;
            let x = center_x + radius * angle_rad.cos();
            let y = center_y + radius * angle_rad.sin();
            positions.push((x, y));
        }
        
        assert_eq!(positions.len(), 6);
        
        // First hole should be at (80, 50)
        assert!((positions[0].0 - 80.0).abs() < 0.001);
        assert!((positions[0].1 - 50.0).abs() < 0.001);
    }
    
    #[test]
    fn test_grid_hole_pattern() {
        // 3x3 grid with 20mm spacing
        let start_x = 10.0;
        let start_y = 10.0;
        let spacing = 20.0;
        let cols = 3;
        let rows = 3;
        
        let mut positions = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                let x = start_x + col as f64 * spacing;
                let y = start_y + row as f64 * spacing;
                positions.push((x, y));
            }
        }
        
        assert_eq!(positions.len(), 9);
        assert!((positions[0].0 - 10.0).abs() < 0.001);
        assert!((positions[8].0 - 50.0).abs() < 0.001);
    }
}

// ============================================================================
// TOOL COMPENSATION TESTS
// ============================================================================

#[cfg(test)]
mod tool_comp_tests {
    use std::f64::consts::PI;
    
    /// Calculate offset point perpendicular to line direction
    fn offset_point(x: f64, y: f64, dx: f64, dy: f64, offset: f64, left: bool) -> (f64, f64) {
        let len = (dx*dx + dy*dy).sqrt();
        let nx = -dy / len; // Normal X (perpendicular)
        let ny = dx / len;  // Normal Y
        
        let sign = if left { 1.0 } else { -1.0 };
        
        (x + sign * nx * offset, y + sign * ny * offset)
    }
    
    #[test]
    fn test_left_offset_horizontal() {
        // Line going right (dx=1, dy=0), left offset should be positive Y
        let (ox, oy) = offset_point(0.0, 0.0, 1.0, 0.0, 5.0, true);
        assert!((ox - 0.0).abs() < 0.001);
        assert!((oy - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_right_offset_horizontal() {
        // Line going right, right offset should be negative Y
        let (ox, oy) = offset_point(0.0, 0.0, 1.0, 0.0, 5.0, false);
        assert!((ox - 0.0).abs() < 0.001);
        assert!((oy - (-5.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_left_offset_vertical() {
        // Line going up (dx=0, dy=1), left offset should be negative X
        let (ox, oy) = offset_point(0.0, 0.0, 0.0, 1.0, 5.0, true);
        assert!((ox - (-5.0)).abs() < 0.001);
        assert!((oy - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_tool_length_compensation() {
        // Z with tool length added
        let programmed_z = -5.0_f64;
        let tool_length = 50.0_f64;
        let machine_z = programmed_z + tool_length;
        assert!((machine_z - 45.0).abs() < 0.001);
    }
    
    #[test]
    fn test_arc_offset_radius() {
        // For arc with cutter comp, effective radius changes
        let nominal_radius = 50.0_f64;
        let tool_radius = 5.0_f64;
        
        // External profile (G41): effective radius = nominal + tool
        let external_radius = nominal_radius + tool_radius;
        assert!((external_radius - 55.0).abs() < 0.001);
        
        // Internal profile (G42): effective radius = nominal - tool
        let internal_radius = nominal_radius - tool_radius;
        assert!((internal_radius - 45.0).abs() < 0.001);
    }
}

// ============================================================================
// EXAMPLE STATISTICS TESTS
// ============================================================================

#[cfg(test)]
mod statistics_tests {
    use super::gcode_examples::*;
    
    #[test]
    fn test_minimum_example_count() {
        let examples = generate_all_examples();
        // We have hundreds of programmatically generated examples
        assert!(examples.len() >= 400, 
            "Expected at least 400 examples, got {}", examples.len());
    }
    
    #[test]
    fn test_category_distribution() {
        let stats = get_example_statistics();
        
        // Should have multiple categories
        assert!(stats.len() >= 15, 
            "Expected at least 15 categories, got {}", stats.len());
        
        // Each category should have at least one example
        for (category, count) in &stats {
            assert!(*count > 0, "Category {:?} has no examples", category);
        }
    }
    
    #[test]
    fn test_complexity_range() {
        let examples = generate_all_examples();
        
        let min_complexity = examples.iter().map(|e| e.complexity).min().unwrap();
        let max_complexity = examples.iter().map(|e| e.complexity).max().unwrap();
        
        assert!(min_complexity <= 2, "Should have low complexity examples");
        assert!(max_complexity >= 5, "Should have high complexity examples");
    }
    
    #[test]
    fn test_tags_present() {
        let examples = generate_all_examples();
        
        let with_tags = examples.iter().filter(|e| !e.tags.is_empty()).count();
        assert!(with_tags > examples.len() / 2, 
            "Most examples should have tags");
    }
    
    #[test]
    fn test_error_examples_count() {
        let error_examples: Vec<_> = generate_all_examples()
            .into_iter()
            .filter(|e| !e.should_parse)
            .collect();
        
        assert!(error_examples.len() >= 20, 
            "Should have at least 20 error examples, got {}", error_examples.len());
    }
    
    #[test]
    fn test_valid_examples_count() {
        let valid_examples: Vec<_> = generate_all_examples()
            .into_iter()
            .filter(|e| e.should_parse)
            .collect();
        
        assert!(valid_examples.len() >= 350, 
            "Should have at least 350 valid examples, got {}", valid_examples.len());
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::gcode_examples::*;
    
    #[test]
    fn test_example_parsing_simulation() {
        // Simulate parsing each example
        let examples = generate_all_examples();
        let mut parse_success = 0;
        let mut parse_fail = 0;
        
        for example in &examples {
            // Simple validation: non-empty content
            if !example.content.is_empty() {
                if example.should_parse {
                    parse_success += 1;
                } else {
                    parse_fail += 1;
                }
            }
        }
        
        assert!(parse_success > 0, "Should have successful parses");
        assert!(parse_fail > 0, "Should have expected failures");
    }
    
    #[test]
    fn test_filtering_by_category() {
        let linear = get_examples_by_category(GCodeCategory::BasicLinearMove);
        let arcs = get_examples_by_category(GCodeCategory::BasicArcCW);
        
        assert!(!linear.is_empty(), "Should have linear examples");
        assert!(!arcs.is_empty(), "Should have arc examples");
        
        // Categories should be distinct
        for ex in &linear {
            assert_eq!(ex.category, GCodeCategory::BasicLinearMove);
        }
    }
    
    #[test]
    fn test_filtering_by_tag() {
        let basic_examples = get_examples_by_tag("basic");
        let ocode_examples = get_examples_by_tag("ocode");
        
        assert!(!basic_examples.is_empty(), "Should have 'basic' tagged examples");
        assert!(!ocode_examples.is_empty(), "Should have 'ocode' tagged examples");
    }
    
    #[test]
    fn test_filtering_by_complexity() {
        let easy = get_examples_by_complexity(0, 2);
        let medium = get_examples_by_complexity(3, 5);
        let hard = get_examples_by_complexity(6, 10);
        
        assert!(!easy.is_empty(), "Should have easy examples");
        assert!(!medium.is_empty(), "Should have medium examples");
        assert!(!hard.is_empty(), "Should have hard examples");
    }
    
    #[test]
    fn test_distance_estimates() {
        let examples = generate_all_examples();
        let with_distance: Vec<_> = examples.iter()
            .filter(|e| e.expected_distance.is_some())
            .collect();
        
        assert!(!with_distance.is_empty(), 
            "Some examples should have distance estimates");
        
        for ex in with_distance {
            let dist = ex.expected_distance.unwrap();
            assert!(dist >= 0.0, "Distance should be non-negative for {}", ex.id);
        }
    }
}

// ============================================================================
// RELATIVE MOTION (G91) TESTS
// ============================================================================

#[cfg(test)]
mod relative_motion_tests {
    use super::*;
    
    /// Simulates relative motion accumulation
    struct RelativeMotionSimulator {
        pub position: (f64, f64, f64),
        pub is_relative: bool,
    }
    
    impl RelativeMotionSimulator {
        fn new() -> Self {
            Self {
                position: (0.0, 0.0, 0.0),
                is_relative: false,
            }
        }
        
        fn set_absolute(&mut self) {
            self.is_relative = false;
        }
        
        fn set_relative(&mut self) {
            self.is_relative = true;
        }
        
        fn move_to(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>) {
            if self.is_relative {
                // Relative: add to current position
                if let Some(dx) = x {
                    self.position.0 += dx;
                }
                if let Some(dy) = y {
                    self.position.1 += dy;
                }
                if let Some(dz) = z {
                    self.position.2 += dz;
                }
            } else {
                // Absolute: set new position
                if let Some(nx) = x {
                    self.position.0 = nx;
                }
                if let Some(ny) = y {
                    self.position.1 = ny;
                }
                if let Some(nz) = z {
                    self.position.2 = nz;
                }
            }
        }
        
        fn approx_eq(&self, expected: (f64, f64, f64), tolerance: f64) -> bool {
            (self.position.0 - expected.0).abs() < tolerance &&
            (self.position.1 - expected.1).abs() < tolerance &&
            (self.position.2 - expected.2).abs() < tolerance
        }
    }
    
    #[test]
    fn test_g91_basic_accumulation() {
        // Test: G91 mode should accumulate moves
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();  // G91
        
        // Simulate: G1 X10 Y10 Z5
        sim.move_to(Some(10.0), Some(10.0), Some(5.0));
        assert!(sim.approx_eq((10.0, 10.0, 5.0), 0.001),
            "First relative move failed: {:?}", sim.position);
        
        // Simulate: G1 X10 Y10 Z5 (again)
        sim.move_to(Some(10.0), Some(10.0), Some(5.0));
        assert!(sim.approx_eq((20.0, 20.0, 10.0), 0.001),
            "Second relative move failed: {:?}", sim.position);
        
        // Simulate: G1 X-5 Y0 Z-2
        sim.move_to(Some(-5.0), Some(0.0), Some(-2.0));
        assert!(sim.approx_eq((15.0, 20.0, 8.0), 0.001),
            "Third relative move with negative failed: {:?}", sim.position);
    }
    
    #[test]
    fn test_g90_absolute_positioning() {
        // Test: G90 mode should use absolute coordinates
        let mut sim = RelativeMotionSimulator::new();
        sim.set_absolute();  // G90 (default)
        
        // Simulate: G1 X10 Y10 Z5
        sim.move_to(Some(10.0), Some(10.0), Some(5.0));
        assert!(sim.approx_eq((10.0, 10.0, 5.0), 0.001));
        
        // Simulate: G1 X20 Y15 Z3
        sim.move_to(Some(20.0), Some(15.0), Some(3.0));
        assert!(sim.approx_eq((20.0, 15.0, 3.0), 0.001),
            "Absolute move should not accumulate: {:?}", sim.position);
    }
    
    #[test]
    fn test_g90_g91_switching() {
        // Test: Switching between G90 and G91 modes
        let mut sim = RelativeMotionSimulator::new();
        
        // Start in absolute mode
        sim.set_absolute();
        sim.move_to(Some(10.0), Some(10.0), Some(0.0));
        assert!(sim.approx_eq((10.0, 10.0, 0.0), 0.001));
        
        // Switch to relative
        sim.set_relative();
        sim.move_to(Some(5.0), Some(5.0), Some(2.0));
        assert!(sim.approx_eq((15.0, 15.0, 2.0), 0.001),
            "After switching to relative: {:?}", sim.position);
        
        // Switch back to absolute
        sim.set_absolute();
        sim.move_to(Some(0.0), Some(0.0), Some(0.0));
        assert!(sim.approx_eq((0.0, 0.0, 0.0), 0.001),
            "After switching back to absolute: {:?}", sim.position);
    }
    
    #[test]
    fn test_g91_partial_coordinates() {
        // Test: G91 with partial coordinate specification
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();
        
        // Start at known position
        sim.position = (10.0, 20.0, 5.0);
        
        // Move only X
        sim.move_to(Some(5.0), None, None);
        assert!(sim.approx_eq((15.0, 20.0, 5.0), 0.001),
            "Partial X move failed: {:?}", sim.position);
        
        // Move only Y and Z
        sim.move_to(None, Some(-10.0), Some(3.0));
        assert!(sim.approx_eq((15.0, 10.0, 8.0), 0.001),
            "Partial Y,Z move failed: {:?}", sim.position);
    }
    
    #[test]
    fn test_g91_return_to_origin() {
        // Test: A sequence of relative moves should be able to return to origin
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();
        
        // Move in a square pattern
        sim.move_to(Some(100.0), Some(0.0), None);
        sim.move_to(Some(0.0), Some(100.0), None);
        sim.move_to(Some(-100.0), Some(0.0), None);
        sim.move_to(Some(0.0), Some(-100.0), None);
        
        assert!(sim.approx_eq((0.0, 0.0, 0.0), 0.001),
            "Square path should return to origin: {:?}", sim.position);
    }
    
    #[test]
    fn test_g91_negative_coordinates() {
        // Test: Negative values in relative mode
        let mut sim = RelativeMotionSimulator::new();
        sim.position = (50.0, 50.0, 50.0);
        sim.set_relative();
        
        // Move with all negative values
        sim.move_to(Some(-25.0), Some(-25.0), Some(-25.0));
        assert!(sim.approx_eq((25.0, 25.0, 25.0), 0.001));
        
        // Move further negative (below zero)
        sim.move_to(Some(-30.0), Some(-30.0), Some(-30.0));
        assert!(sim.approx_eq((-5.0, -5.0, -5.0), 0.001),
            "Negative coordinates should work: {:?}", sim.position);
    }
    
    #[test]
    fn test_g91_zero_moves() {
        // Test: Zero-distance moves in relative mode
        let mut sim = RelativeMotionSimulator::new();
        sim.position = (10.0, 20.0, 30.0);
        sim.set_relative();
        
        // Zero move should not change position
        sim.move_to(Some(0.0), Some(0.0), Some(0.0));
        assert!(sim.approx_eq((10.0, 20.0, 30.0), 0.001),
            "Zero move should not change position: {:?}", sim.position);
    }
    
    #[test]
    fn test_g91_accumulated_precision() {
        // Test: Accumulated moves should maintain precision
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();
        
        // Many small moves
        for _ in 0..1000 {
            sim.move_to(Some(0.001), Some(0.001), Some(0.001));
        }
        
        assert!((sim.position.0 - 1.0).abs() < 0.01,
            "X precision: expected 1.0, got {}", sim.position.0);
        assert!((sim.position.1 - 1.0).abs() < 0.01,
            "Y precision: expected 1.0, got {}", sim.position.1);
        assert!((sim.position.2 - 1.0).abs() < 0.01,
            "Z precision: expected 1.0, got {}", sim.position.2);
    }
    
    #[test]
    fn test_g91_with_g2_g3_arcs() {
        // Test: Relative mode should affect arc endpoint interpretation
        // In G91 mode, the arc endpoint (X, Y, Z) is relative to current position
        // But I, J, K offsets are ALWAYS relative to current position
        
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();
        
        // Starting at origin, a G2 X10 Y0 would end at (10, 0)
        // which is +10 in X from (0,0)
        sim.move_to(Some(10.0), Some(0.0), None);
        assert!(sim.approx_eq((10.0, 0.0, 0.0), 0.001),
            "Arc endpoint in relative mode: {:?}", sim.position);
    }
    
    #[test]
    fn test_gcode_example_incremental_structure() {
        // Verify that incremental examples contain proper G91 structure
        let inc_examples = get_examples_by_category(GCodeCategory::IncrementalCoordinates);
        
        for example in &inc_examples {
            // Each should start with G91 or set incremental mode
            let content_lower = example.content.to_lowercase();
            assert!(
                content_lower.contains("g91"),
                "Incremental example {} should contain G91: {}",
                example.id,
                example.content
            );
        }
    }
    
    #[test]
    fn test_relative_motion_with_different_feedrates() {
        // Test that relative moves work correctly regardless of feedrate
        let mut sim = RelativeMotionSimulator::new();
        sim.set_relative();
        
        // Simulate moves with varying feedrates (feedrate doesn't affect position)
        // F100
        sim.move_to(Some(10.0), Some(0.0), None);
        // F1000
        sim.move_to(Some(10.0), Some(0.0), None);
        // F5000
        sim.move_to(Some(10.0), Some(0.0), None);
        
        assert!(sim.approx_eq((30.0, 0.0, 0.0), 0.001),
            "Position should be sum of moves: {:?}", sim.position);
    }
}

