use gcodeflow::gcode::TrajectoryGenerator;

#[test]
fn get_block_range_for_lines_simple_moves() {
    let mut gen = TrajectoryGenerator::new().expect("Failed to create generator");

    // Simple two-line program
    let gcode = "G0 X100 Y100\nG0 Y0\n";

    let points = gen.generate_from_gcode(gcode, 6000.0, 1000.0, 10000.0, 0.001)
        .expect("Generation failed");

    assert!(!points.is_empty(), "Generator returned no trajectory points");

    // Query block range for the first line (line indices are zero-based here)
    let range = gen.get_block_range_for_lines(0, 0);

    assert!(range.is_some(), "get_block_range_for_lines returned None");

    let (start_block, end_block) = range.unwrap();
    assert!(start_block <= end_block, "start_block > end_block");

    // Now generate from only the second line and verify context is not preserved
    let mut gen2 = TrajectoryGenerator::new().expect("Failed to create generator");
    let second_line = "G0 Y0\n";
    let points2 = gen2.generate_from_gcode(second_line, 6000.0, 1000.0, 10000.0, 0.001)
        .expect("Generation failed for second line");

    // Since generation is stateless and starts at origin, a single "G0 Y0" will produce
    // either zero movement or a single point at origin rather than moving to X=100 (context lost).
    // Confirm that points returned are at or near origin
    assert!(!points2.is_empty(), "Generator returned no points for single line");
    let first_pt = &points2[0];
    assert_eq!(first_pt.block_index, 0);
    assert!(first_pt.position.x.abs() < 1e-6 && first_pt.position.y.abs() < 1e-6, "Single-line generation did not start at origin as expected");
}

