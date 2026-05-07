# GCodeFlow

A comprehensive GCode visualization and simulation tool built with Bevy and egui.

## Features

### 3D Visualization
- Real-time 3D rendering of G-code trajectories
- Multiple camera modes: Perspective, Orthographic, 2D views (XY, XZ, YZ planes)
- Configurable grid and axis display
- Tool visualization with multiple tool types (End Mill, Ball Nose, V-Bit, Drill, 3D Printer Nozzle)

### Trace Coloring
Color trajectories by different modes:
- **By Move Type**: Rapids (red), Feeds (green), Arcs (blue)
- **By Speed**: Gradient from low to high feed rates
- **By Z Height**: Gradient from low to high Z positions
- **By Time**: Gradient from start to end of program
- **By Acceleration**: Highlight aggressive motion
- **By Accuracy**: Show deviation from ideal path

### Interpolation Control
Fine-tune trajectory generation:
- **Fixed Time Step**: Generate points at regular time intervals
- **Fixed Deviation**: Generate points based on maximum path deviation
- **Arc Segments**: Control smoothness of arc interpolation
- Quick presets: Fast Preview, Balanced, High Detail

### Simulation
- Real-time playback with adjustable speed
- Loop playback option
- Tool tracking along trajectory

### Editor
- Syntax highlighting for G-code
- Line numbers
- File operations (Open, Save, Save As)

## CLI Usage

GCodeFlow includes several CLI commands for testing and verification:

### Verify Command
Verify trajectory generation with optional expectations:

```bash
# Basic verification
gcodeflow verify --gcode "G0 X10 Y10\nG1 X20 Y20 F1000" --format verbose

# With expectations (fails if not met)
gcodeflow verify --file test.gcode \
  --expect-x 100 \
  --expect-y 50 \
  --expect-points 500 \
  --format json

# Output formats: summary, json, verbose
```

### Points Command
Generate trajectory points as CSV:

```bash
gcodeflow points --file test.gcode --resolution 0.01 --format csv
```

### Debug Command
Show detailed motion segment information:

```bash
gcodeflow debug --file test.gcode --verbose
```

### Emulate Command
Simulate tool movement with timestamps:

```bash
gcodeflow emulate --file test.gcode --resolution 0.01
```

### Highlight Command
Syntax highlight G-code:

```bash
gcodeflow highlight --file test.gcode --format ansi
gcodeflow highlight --gcode "G1 X10 F100" --format html
```

## Configuration

Settings are stored in `gcodeflow.yaml`. Key sections:

- **camera**: Projection mode, tracking, sensitivity
- **trace**: Colors, visibility, line style
- **tool**: Tool type, scale, color
- **trajectory**: Time resolution, deviation limits

## Building

```bash
cargo build --release
```

## Testing

```bash
# Run all tests
cargo test

# Run relative motion tests
cargo test relative_motion

# Run with verbose output
cargo test -- --nocapture
```

## License

MIT License
