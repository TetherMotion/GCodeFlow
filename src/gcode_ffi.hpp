//! C++ bridge for cxx to bind GCode API to Rust
//!
//! This header declares opaque wrapper classes that wrap the GCode namespace types
//! and provide factory functions for Rust/cxx interop.

#pragma once

#include "tether/gcode/GCodeLexer.hpp"
#include "tether/gcode/GCodeParser.hpp"
#include "tether/gcode/GCodeTypes.hpp"
#include "tether/gcode/motion/InterpolationStrategy.hpp"
#include "rust/cxx.h"

#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace gcode_ffi {

// Forward declarations for cxx-generated struct types
struct FfiPosition;
struct FfiMotionSegment;
struct FfiTrajectoryPoint;
struct FfiHighlightSpan;

// Thread-local error tracking
void setLastError(const std::string& msg);
rust::String last_error();

// Syntax highlighting (C++ lexer-based)
rust::Vec<FfiHighlightSpan> highlight_line(rust::Str line);

// Opaque types exposed via unique_ptr
class FfiParser {
public:
    FfiParser();

    bool parse_string(rust::Str gcode) const;
    bool parse_file(rust::Str path) const;
    std::size_t block_count() const;
    rust::String get_block_original_text(size_t index) const;

private:
    mutable GCode::VariableSystem variables_;
    mutable std::unique_ptr<GCode::Lexer> lexer_;
    mutable std::unique_ptr<GCode::Parser> parser_;
    mutable std::vector<GCode::Block> blocks_;
    friend class FfiInterpreter;
};

struct FfiInterpreterConfig {
    double maxVelocity;
    double maxAcceleration;
    double maxJerk;
};

class FfiInterpreter {
public:
    FfiInterpreter();

    void configure(double max_vel, double max_accel, double max_jerk);
    bool load_blocks(const FfiParser& parser);
    std::size_t segment_count() const;
    FfiMotionSegment get_segment(std::size_t index) const;
    rust::Vec<FfiMotionSegment> get_all_segments() const;

private:
    FfiInterpreterConfig config_;
    std::vector<GCode::MotionSegment> segments_;
    GCode::Position currentPosition_;
    friend class FfiTrajectoryGenerator;
};

class FfiTrajectoryGenerator {
public:
    FfiTrajectoryGenerator();

    void configure(double time_step, double max_deviation);
    bool generate(const FfiInterpreter& interp);
    std::size_t point_count() const;
    double duration() const;
    FfiTrajectoryPoint get_point(std::size_t index) const;
    rust::Vec<FfiTrajectoryPoint> get_all_points() const;
    
    // High-resolution sampling for plotting
    rust::Vec<FfiTrajectoryPoint> sample_at_interval(double interval_seconds) const;
    rust::Vec<FfiTrajectoryPoint> sample_adaptive(double max_deviation_mm) const;
    
    // Query specific time
    FfiTrajectoryPoint query_at_time(double time_seconds) const;
    
    // Get block index range for gcode line
    bool get_block_range_for_lines(std::size_t start_line, std::size_t end_line, 
                                     std::size_t& out_start_block, std::size_t& out_end_block) const;

private:
    GCode::InterpolationConfig config_;
    std::vector<GCode::TrajectoryPoint> points_;
    double totalDuration_;
};

// Factory functions exposed to Rust via cxx
std::unique_ptr<FfiParser> new_parser();
std::unique_ptr<FfiInterpreter> new_interpreter();
std::unique_ptr<FfiTrajectoryGenerator> new_trajectory_generator();

} // namespace gcode_ffi
