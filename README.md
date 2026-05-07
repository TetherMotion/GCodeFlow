# GCodeFlow

A comprehensive GCode visualization and simulation tool built with Bevy and egui. GCodeFlow provides real-time 3D rendering of G-code trajectories, advanced interpolation algorithms, and powerful analysis tools for CNC machining and 3D printing applications.

## Overview

GCodeFlow is designed to help CNC operators, machinists, and 3D printing enthusiasts visualize and analyze G-code programs before running them on actual hardware. It provides:

- **Real-time 3D visualization** of tool paths with multiple camera perspectives
- **Advanced trajectory interpolation** using G64 continuous path control
- **2D plot analysis** for position, velocity, acceleration, and jerk over time
- **Interactive editor** with syntax highlighting
- **Comprehensive CLI tools** for verification and debugging
- **High-performance C++ backend** with Rust frontend

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

### 2D Plot Analysis
- Time-based plotting of position, velocity, acceleration, and jerk
- Cartesian 2D projections (XY, XZ, YZ planes)
- Interactive point selection with detailed kinematic information
- Auto-fit and manual zoom/pan controls

## Architecture

GCodeFlow uses a hybrid Rust/C++ architecture:

```
┌─────────────────────────────────────────┐
│         Rust Frontend (Bevy/egui)       │
│  - UI, 3D rendering, user interaction  │
│  - Plot visualization                   │
│  - File I/O and configuration          │
└──────────────────┬──────────────────────┘
                   │ cxx FFI bridge
                   ↓
┌─────────────────────────────────────────┐
│         C++ Backend (Tether)            │
│  - G-code parsing and interpretation    │
│  - Trajectory generation                │
│  - Interpolation algorithms             │
│  - Motion planning                     │
└─────────────────────────────────────────┘
```

This architecture leverages:
- **Rust** for safe, fast UI development with Bevy game engine
- **C++** for computationally intensive trajectory calculations
- **cxx** for safe Rust-C++ interop

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

## Installation

### Prerequisites

- **Rust** 1.70 or later (install from [rustup.rs](https://rustup.rs/))
- **C++ Compiler** with C++17 support (gcc, clang, or MSVC)
- **CMake** 3.15 or later
- **pkg-config** (Linux) or vcpkg (Windows)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/TetherMotion/GCodeFlow.git
cd GCodeFlow

# Build in release mode
cargo build --release

# The binary will be at target/release/gcodeflow
```

### Development Build

```bash
# Build with optimizations for dependencies but faster debug builds
cargo build

# Run the application
cargo run -- --input examples/002_square.gcode
```

## Usage

### GUI Application

Launch the GUI application:

```bash
cargo run --release -- [OPTIONS]
```

Options:
- `--input <FILE>`: Load G-code file on startup
- `--config <FILE>`: Use custom configuration file

### CLI Commands

GCodeFlow includes several CLI commands for testing and verification:

#### Verify Command
Verify trajectory generation with optional expectations:

```bash
# Basic verification
cargo run --release -- verify --gcode "G0 X10 Y10\nG1 X20 Y20 F1000" --format verbose

# With expectations (fails if not met)
cargo run --release -- verify --file test.gcode \
  --expect-x 100 \
  --expect-y 50 \
  --expect-points 500 \
  --format json

# Output formats: summary, json, verbose
```

#### Points Command
Generate trajectory points as CSV:

```bash
cargo run --release -- points --file test.gcode --resolution 0.01 --format csv
```

#### Debug Command
Show detailed motion segment information:

```bash
cargo run --release -- debug --file test.gcode --verbose
```

#### Emulate Command
Simulate tool movement with timestamps:

```bash
cargo run --release -- emulate --file test.gcode --resolution 0.01
```

#### Highlight Command
Syntax highlight G-code:

```bash
cargo run --release -- highlight --file test.gcode --format ansi
cargo run --release -- highlight --gcode "G1 X10 F100" --format html
```

## Configuration

Settings are stored in `gcodeflow.yaml`. Key sections:

- **camera**: Projection mode, tracking, sensitivity
- **trace**: Colors, visibility, line style
- **tool**: Tool type, scale, color
- **trajectory**: Time resolution, deviation limits

Example configuration:

```yaml
camera:
  projection: perspective
  tracking: false
  sensitivity: 0.5

trace:
  color_mode: move_type
  show_rapids: true
  show_feeds: true
  line_width: 2.0

tool:
  type: end_mill
  scale: 1.0
  color: "#FF0000"

trajectory:
  time_resolution: 0.001
  max_deviation: 0.01
  arc_segments: 32
```

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test relative_motion

# Run with verbose output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Benchmarking

```bash
# Run benchmarks
cargo run --release --bin gcode_benchmark
```

## Examples

The `examples/` directory contains various G-code examples demonstrating different features:

- `000_empty.gcode`: Empty file
- `001_single_line.gcode`: Single linear move
- `002_square.gcode`: Basic square pattern
- `010_rapids.gcode`: Rapid movements
- `020_arcs.gcode`: Arc commands
- `100_rounded_square.gcode`: Rounded corners
- `101_circle.gcode`: Circle generation
- `110_helix.gcode`: Helical interpolation
- `120_pocketing.gcode`: Pocket milling
- `200_ocode_sub.gcode`: O-code subroutines
- `300_nested_loops.gcode`: Nested loop structures
- `320_3d_surface.gcode`: 3D surface machining
- `400_complete_part.gcode`: Complete part example
- `500_5axis_impeller.gcode`: 5-axis machining

See `examples/INDEX.txt` for a complete list with descriptions.

## Documentation

- [2D Plot Feature Documentation](docs/2D_PLOT_FEATURE.md)
- [Plot Quick Reference](docs/PLOT_QUICK_REFERENCE.md)
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)

## Project Structure

```
GCodeFlow/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library interface
│   ├── app.rs               # Bevy app setup
│   ├── gcode.rs             # G-code processing (Rust)
│   ├── gcode_ffi.cpp        # C++ FFI implementation
│   ├── gcode_ffi.hpp        # C++ FFI declarations
│   ├── ui.rs                # UI systems
│   ├── rendering.rs         # 3D rendering
│   ├── plot_view.rs         # 2D plot visualization
│   ├── simulation.rs        # Trajectory simulation
│   ├── trajectory.rs        # Trajectory data structures
│   └── bin/
│       └── benchmark.rs     # Benchmark binary
├── examples/                # G-code examples
├── scripts/                 # Utility scripts
├── tests/                   # Integration tests
├── docs/                    # Documentation
├── Cargo.toml               # Rust dependencies
└── build.rs                 # Build script for C++ code
```

## Performance

Based on benchmark measurements:

| Operation | Dataset | Time | Notes |
|-----------|---------|------|-------|
| Trajectory generation | 10k lines | < 50ms | Including parsing |
| sample_at_interval | 10k points, 1k samples | < 100ms | Binary search overhead |
| query_at_time | 10k points | < 0.1ms | Single query |
| 3D rendering | 1k points | < 16ms | 60 FPS achievable |
| Plot rendering | 1k points | < 16ms | egui_plot overhead |

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write tests for new features
- Update documentation as needed

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Built with [Bevy](https://bevyengine.org/) game engine
- UI powered by [egui](https://github.com/emilk/egui)
- Plotting with [egui_plot](https://github.com/emilk/egui_plot)
- C++ interop via [cxx](https://github.com/dtolnay/cxx)

## Related Projects

- [Tether](https://github.com/TetherMotion/Tether) - C++ G-code library
- [ESP32EtherCAT](https://github.com/techoverflow/ESP32EtherCAT) - ESP32 EtherCAT master

## Support

For issues, questions, or contributions:
- Open an issue on GitHub
- Check existing documentation in `docs/`
- Review example files in `examples/`
