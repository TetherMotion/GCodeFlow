//! C++ implementation for the FFI bridge (cxx)
//! Uses the full GCode namespace API

#include <cmath>
#include <fstream>
#include <sstream>

#include "gcode_ffi.hpp"
#include "gcodeflow/src/gcode.rs.h"
#include "tether/gcode/motion/InterpolationStrategy.hpp"

namespace gcode_ffi {

// Thread-local error tracking
static thread_local std::string g_lastError;

void setLastError(const std::string& msg) {
    g_lastError = msg;
}

rust::String last_error() {
    return rust::String(g_lastError);
}

// Helper to convert GCode::Position to FfiPosition
static FfiPosition to_ffi_position(const GCode::Position& p) {
    FfiPosition out;
    out.x = p.coords[0];
    out.y = p.coords[1];
    out.z = p.coords[2];
    out.a = p.coords[3];
    out.b = p.coords[4];
    out.c = p.coords[5];
    out.u = p.coords[6];
    out.v = p.coords[7];
    out.w = p.coords[8];
    return out;
}

// FfiParser -------------------------------------------------------------------

FfiParser::FfiParser()
    : variables_()
    , lexer_(std::make_unique<GCode::Lexer>())
    , parser_(std::make_unique<GCode::Parser>(variables_))
{
}

bool FfiParser::parse_string(rust::Str gcode) const {
    blocks_.clear();
    std::string source(gcode.data(), gcode.size());
    
    // Split into lines and parse each
    std::istringstream stream(source);
    std::string line;
    
    while (std::getline(stream, line)) {
        if (line.empty()) {
            continue;
        }

        if (GCode::isEmptyOrComment(line.c_str())) {
            continue;
        }
        
        // Parse into block
        GCode::Block block;
        GCode::Error err = parser_->parseLine(line.c_str(), block);
        if (err) {
            setLastError(std::string(err.message.data()));
            return false;
        }
        
        blocks_.push_back(std::move(block));
    }
    
    return true;
}

bool FfiParser::parse_file(rust::Str path) const {
    std::string filename(path.data(), path.size());
    std::ifstream file(filename);
    if (!file.is_open()) {
        setLastError("Failed to open file: " + filename);
        return false;
    }
    
    std::stringstream buffer;
    buffer << file.rdbuf();
    const std::string content = buffer.str();
    return parse_string(rust::Str(content.data(), content.size()));
}

std::size_t FfiParser::block_count() const {
    return blocks_.size();
}

rust::String FfiParser::get_block_original_text(size_t index) const {
    if (index >= blocks_.size()) {
        return rust::String("");
    }
    return rust::String(blocks_[index].originalText.data());
}

// FfiInterpreter --------------------------------------------------------------

FfiInterpreter::FfiInterpreter()
    : config_()
    , segments_()
    , currentPosition_()
{
    config_.maxVelocity = 6000.0;
    config_.maxAcceleration = 1000.0;
    config_.maxJerk = 10000.0;
}

void FfiInterpreter::configure(double max_vel, double max_accel, double max_jerk) {
    config_.maxVelocity = max_vel;
    config_.maxAcceleration = max_accel;
    config_.maxJerk = max_jerk;
}

bool FfiInterpreter::load_blocks(const FfiParser& parser) {
    segments_.clear();
    currentPosition_ = GCode::Position{};
    
    // Process each block to extract motion segments
    for (const auto& block : parser.blocks_) {
        // Check for motion G-codes
        bool hasMotion = false;
        GCode::MotionMode motionMode = GCode::MotionMode::LINEAR;
        
        for (int i = 0; i < block.gCodeCount; ++i) {
            const int16_t raw = block.gCodes[static_cast<size_t>(i)];
            const int major = static_cast<int>(raw / 10);
            if (major == 0) {
                motionMode = GCode::MotionMode::RAPID;
                hasMotion = true;
            } else if (major == 1) {
                motionMode = GCode::MotionMode::LINEAR;
                hasMotion = true;
            } else if (major == 2) {
                motionMode = GCode::MotionMode::CW_ARC;
                hasMotion = true;
            } else if (major == 3) {
                motionMode = GCode::MotionMode::CCW_ARC;
                hasMotion = true;
            }
        }
        
        // Check for axis words
        GCode::Position targetPos = currentPosition_;
        bool hasAxisWords = false;
        
        for (size_t i = 0; i < block.words.size(); ++i) {
            const auto& word = block.words[i];
            if (!word.present) {
                continue;
            }
            switch (word.letter) {
                case GCode::WordLetter::X:
                    targetPos.coords[0] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::Y:
                    targetPos.coords[1] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::Z:
                    targetPos.coords[2] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::A:
                    targetPos.coords[3] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::B:
                    targetPos.coords[4] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::C:
                    targetPos.coords[5] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::U:
                    targetPos.coords[6] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::V:
                    targetPos.coords[7] = word.value;
                    hasAxisWords = true;
                    break;
                case GCode::WordLetter::W:
                    targetPos.coords[8] = word.value;
                    hasAxisWords = true;
                    break;
                default:
                    break;
            }
        }
        
        // Get feed rate
        double feedRate = config_.maxVelocity;
        if (block.hasWord(GCode::WordLetter::F)) {
            feedRate = block.getWord(GCode::WordLetter::F);
        }
        
        // Create segment if we have motion
        if (hasAxisWords) {
            GCode::MotionSegment seg;
            
            // Set motion type
            switch (motionMode) {
                case GCode::MotionMode::RAPID:
                    seg.type = GCode::MotionSegment::Type::RAPID;
                    break;
                case GCode::MotionMode::LINEAR:
                    seg.type = GCode::MotionSegment::Type::LINEAR;
                    break;
                case GCode::MotionMode::CW_ARC:
                    seg.type = GCode::MotionSegment::Type::ARC_CW;
                    break;
                case GCode::MotionMode::CCW_ARC:
                    seg.type = GCode::MotionSegment::Type::ARC_CCW;
                    break;
                default:
                    seg.type = GCode::MotionSegment::Type::LINEAR;
                    break;
            }
            
            seg.endPosition = targetPos;
            seg.feedRate = feedRate;
            seg.lineNumber = block.lineNumber;
            
            // Calculate segment length
            double dx = targetPos.coords[0] - currentPosition_.coords[0];
            double dy = targetPos.coords[1] - currentPosition_.coords[1];
            double dz = targetPos.coords[2] - currentPosition_.coords[2];
            double length = std::sqrt(dx*dx + dy*dy + dz*dz);
            seg.duration = (length > 0 && feedRate > 0) ? (length / feedRate * 60.0) : 0.0;
            
            segments_.push_back(seg);
            currentPosition_ = targetPos;
        }
    }
    
    return true;
}

// Syntax highlighting ---------------------------------------------------------

rust::Vec<FfiHighlightSpan> highlight_line(rust::Str line) {
    const std::string_view sv(line.data(), line.size());
    const auto spans = GCode::highlightLine(sv);

    rust::Vec<FfiHighlightSpan> out;
    out.reserve(spans.size());

    for (const auto& s : spans) {
        if (s.length == 0) {
            continue;
        }
        FfiHighlightSpan span;
        span.start = s.start;
        span.length = s.length;
        span.kind = static_cast<uint8_t>(s.kind);
        out.push_back(span);
    }

    return out;
}

std::size_t FfiInterpreter::segment_count() const {
    return segments_.size();
}

FfiMotionSegment FfiInterpreter::get_segment(std::size_t index) const {
    FfiMotionSegment out{};
    if (index >= segments_.size()) {
        return out;
    }
    
    const auto& seg = segments_[index];
    
    // Calculate start position (end of previous segment or origin)
    GCode::Position startPos{};
    if (index > 0) {
        startPos = segments_[index - 1].endPosition;
    }
    
    out.start = to_ffi_position(startPos);
    out.end = to_ffi_position(seg.endPosition);
    out.center = to_ffi_position(seg.centerOffset);
    out.feed_rate = seg.feedRate;
    
    // Calculate segment length
    double dx = seg.endPosition.coords[0] - startPos.coords[0];
    double dy = seg.endPosition.coords[1] - startPos.coords[1];
    double dz = seg.endPosition.coords[2] - startPos.coords[2];
    out.segment_length = std::sqrt(dx*dx + dy*dy + dz*dz);
    out.segment_time = seg.duration;
    
    // Motion type mapping
    switch (seg.type) {
        case GCode::MotionSegment::Type::RAPID:
            out.motion_type = 0;
            out.is_rapid = true;
            break;
        case GCode::MotionSegment::Type::LINEAR:
            out.motion_type = 1;
            out.is_rapid = false;
            break;
        case GCode::MotionSegment::Type::ARC_CW:
            out.motion_type = 2;
            out.is_rapid = false;
            break;
        case GCode::MotionSegment::Type::ARC_CCW:
            out.motion_type = 3;
            out.is_rapid = false;
            break;
        default:
            out.motion_type = 1;
            out.is_rapid = false;
            break;
    }
    
    out.block_index = static_cast<int32_t>(index);
    out.plane = 0; // XY plane
    out.arc_radius = 0.0;
    out.arc_sweep = 0.0;
    
    return out;
}

rust::Vec<FfiMotionSegment> FfiInterpreter::get_all_segments() const {
    rust::Vec<FfiMotionSegment> out;
    out.reserve(segments_.size());
    for (std::size_t i = 0; i < segments_.size(); ++i) {
        out.push_back(get_segment(i));
    }
    return out;
}

// FfiTrajectoryGenerator ------------------------------------------------------

FfiTrajectoryGenerator::FfiTrajectoryGenerator()
    : config_()
    , points_()
    , totalDuration_(0.0)
{
    config_.timeResolution = 0.001;
    config_.maxChordDeviation = 0.01;
}

void FfiTrajectoryGenerator::configure(double time_step, double max_deviation) {
    config_.timeResolution = time_step;
    config_.maxChordDeviation = max_deviation;
}

bool FfiTrajectoryGenerator::generate(const FfiInterpreter& interp) {
    points_.clear();
    totalDuration_ = 0.0;
    
    if (interp.segments_.empty()) {
        return true;
    }
    
    // Use factory to create interpolation strategy
    auto strategy = GCode::InterpolationStrategyFactory::create(config_);
    if (!strategy) {
        setLastError("Failed to create interpolation strategy");
        return false;
    }
    
    // Build planning segments from interpreter segments
    GCode::InterpolationContext ctx;
    ctx.config = config_;
    
    GCode::Position currentPos{};
    for (size_t i = 0; i < interp.segments_.size(); ++i) {
        const auto& seg = interp.segments_[i];
        
        GCode::PlanningSegment planSeg;
        planSeg.start = currentPos;
        planSeg.end = seg.endPosition;
        planSeg.feedRate = seg.feedRate;
        planSeg.blockIndex = static_cast<int32_t>(i);
        
        switch (seg.type) {
            case GCode::MotionSegment::Type::RAPID:
                planSeg.motionType = GCode::SegmentMotionType::Rapid;
                planSeg.isRapid = true;
                break;
            case GCode::MotionSegment::Type::LINEAR:
                planSeg.motionType = GCode::SegmentMotionType::Linear;
                break;
            case GCode::MotionSegment::Type::ARC_CW:
                planSeg.motionType = GCode::SegmentMotionType::ArcCW;
                planSeg.center = seg.centerOffset;
                break;
            case GCode::MotionSegment::Type::ARC_CCW:
                planSeg.motionType = GCode::SegmentMotionType::ArcCCW;
                planSeg.center = seg.centerOffset;
                break;
            default:
                planSeg.motionType = GCode::SegmentMotionType::Linear;
                break;
        }
        
        // Calculate segment length
        double dx = planSeg.end.coords[0] - planSeg.start.coords[0];
        double dy = planSeg.end.coords[1] - planSeg.start.coords[1];
        double dz = planSeg.end.coords[2] - planSeg.start.coords[2];
        planSeg.segmentLength = std::sqrt(dx*dx + dy*dy + dz*dz);
        
        ctx.segments.push_back(planSeg);
        currentPos = seg.endPosition;
    }
    
    // Interpolate all segments
    std::vector<GCode::TrajectoryPoint> allPoints;
    GCode::InterpolationResult result = strategy->interpolateAll(ctx, allPoints);
    
    if (!result.success) {
        setLastError(result.errorMessage);
        return false;
    }
    
    points_ = std::move(allPoints);
    totalDuration_ = result.totalDuration;
    
    return true;
}

std::size_t FfiTrajectoryGenerator::point_count() const {
    return points_.size();
}

double FfiTrajectoryGenerator::duration() const {
    return totalDuration_;
}

FfiTrajectoryPoint FfiTrajectoryGenerator::get_point(std::size_t index) const {
    FfiTrajectoryPoint out{};
    if (index >= points_.size()) {
        return out;
    }
    
    const auto& pt = points_[index];
    out.time = pt.time;
    out.position = to_ffi_position(pt.position);
    out.velocity = to_ffi_position(pt.velocity);
    out.acceleration = to_ffi_position(pt.acceleration);
    out.block_index = pt.blockIndex;
    out.segment_index = pt.segmentIndex;
    
    // Motion type mapping
    switch (pt.motionType) {
        case GCode::SegmentMotionType::Rapid:
            out.motion_type = 0;
            break;
        case GCode::SegmentMotionType::Linear:
            out.motion_type = 1;
            break;
        case GCode::SegmentMotionType::ArcCW:
            out.motion_type = 2;
            break;
        case GCode::SegmentMotionType::ArcCCW:
            out.motion_type = 3;
            break;
        default:
            out.motion_type = 1;
            break;
    }
    
    out.is_interpolated = pt.isInterpolated;
    
    return out;
}

rust::Vec<FfiTrajectoryPoint> FfiTrajectoryGenerator::get_all_points() const {
    rust::Vec<FfiTrajectoryPoint> out;
    out.reserve(points_.size());
    for (std::size_t i = 0; i < points_.size(); ++i) {
        out.push_back(get_point(i));
    }
    return out;
}

// High-resolution sampling for plotting ---------------------------------------

rust::Vec<FfiTrajectoryPoint> FfiTrajectoryGenerator::sample_at_interval(double interval_seconds) const {
    rust::Vec<FfiTrajectoryPoint> out;
    
    if (points_.empty() || interval_seconds <= 0.0) {
        return out;
    }
    
    double current_time = 0.0;
    const double end_time = totalDuration_;
    
    // Reserve approximate capacity
    out.reserve(static_cast<size_t>(end_time / interval_seconds) + 2);
    
    while (current_time <= end_time) {
        FfiTrajectoryPoint pt = query_at_time(current_time);
        out.push_back(pt);
        current_time += interval_seconds;
    }
    
    // Always include the last point
    if (out.empty() || std::abs(out.back().time - end_time) > 1e-6) {
        out.push_back(query_at_time(end_time));
    }
    
    return out;
}

rust::Vec<FfiTrajectoryPoint> FfiTrajectoryGenerator::sample_adaptive(double max_deviation_mm) const {
    rust::Vec<FfiTrajectoryPoint> out;
    
    if (points_.empty() || max_deviation_mm <= 0.0) {
        return out;
    }
    
    // Start with existing points and refine where needed
    out.reserve(points_.size() * 2);
    
    for (size_t i = 0; i < points_.size(); ++i) {
        out.push_back(get_point(i));
        
        if (i + 1 < points_.size()) {
            const auto& pt1 = points_[i];
            const auto& pt2 = points_[i + 1];
            
            // Calculate position deviation
            double dx = pt2.position.coords[0] - pt1.position.coords[0];
            double dy = pt2.position.coords[1] - pt1.position.coords[1];
            double dz = pt2.position.coords[2] - pt1.position.coords[2];
            double dist = std::sqrt(dx*dx + dy*dy + dz*dz);
            
            // If distance exceeds threshold, add intermediate points
            if (dist > max_deviation_mm) {
                int num_subdivisions = static_cast<int>(std::ceil(dist / max_deviation_mm));
                for (int j = 1; j < num_subdivisions; ++j) {
                    double frac = static_cast<double>(j) / num_subdivisions;
                    double interp_time = pt1.time + frac * (pt2.time - pt1.time);
                    out.push_back(query_at_time(interp_time));
                }
            }
        }
    }
    
    return out;
}

FfiTrajectoryPoint FfiTrajectoryGenerator::query_at_time(double time_seconds) const {
    FfiTrajectoryPoint out{};
    
    if (points_.empty()) {
        return out;
    }
    
    // Clamp to valid range
    if (time_seconds <= 0.0) {
        return get_point(0);
    }
    if (time_seconds >= totalDuration_) {
        return get_point(points_.size() - 1);
    }
    
    // Binary search for the time interval
    size_t left = 0;
    size_t right = points_.size() - 1;
    
    while (left < right) {
        size_t mid = (left + right) / 2;
        if (points_[mid].time < time_seconds) {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    
    // Linear interpolation between adjacent points
    if (left == 0) {
        return get_point(0);
    }
    
    const auto& pt1 = points_[left - 1];
    const auto& pt2 = points_[left];
    
    double dt = pt2.time - pt1.time;
    if (dt < 1e-9) {
        return get_point(left);
    }
    
    double frac = (time_seconds - pt1.time) / dt;
    
    // Interpolate all fields
    out.time = time_seconds;
    
    for (size_t i = 0; i < GCode::MAX_AXES; ++i) {
        out.position.x = pt1.position.coords[0] + frac * (pt2.position.coords[0] - pt1.position.coords[0]);
        out.position.y = pt1.position.coords[1] + frac * (pt2.position.coords[1] - pt1.position.coords[1]);
        out.position.z = pt1.position.coords[2] + frac * (pt2.position.coords[2] - pt1.position.coords[2]);
        out.position.a = pt1.position.coords[3] + frac * (pt2.position.coords[3] - pt1.position.coords[3]);
        out.position.b = pt1.position.coords[4] + frac * (pt2.position.coords[4] - pt1.position.coords[4]);
        out.position.c = pt1.position.coords[5] + frac * (pt2.position.coords[5] - pt1.position.coords[5]);
        out.position.u = pt1.position.coords[6] + frac * (pt2.position.coords[6] - pt1.position.coords[6]);
        out.position.v = pt1.position.coords[7] + frac * (pt2.position.coords[7] - pt1.position.coords[7]);
        out.position.w = pt1.position.coords[8] + frac * (pt2.position.coords[8] - pt1.position.coords[8]);
        
        out.velocity.x = pt1.velocity.coords[0] + frac * (pt2.velocity.coords[0] - pt1.velocity.coords[0]);
        out.velocity.y = pt1.velocity.coords[1] + frac * (pt2.velocity.coords[1] - pt1.velocity.coords[1]);
        out.velocity.z = pt1.velocity.coords[2] + frac * (pt2.velocity.coords[2] - pt1.velocity.coords[2]);
        out.velocity.a = pt1.velocity.coords[3] + frac * (pt2.velocity.coords[3] - pt1.velocity.coords[3]);
        out.velocity.b = pt1.velocity.coords[4] + frac * (pt2.velocity.coords[4] - pt1.velocity.coords[4]);
        out.velocity.c = pt1.velocity.coords[5] + frac * (pt2.velocity.coords[5] - pt1.velocity.coords[5]);
        out.velocity.u = pt1.velocity.coords[6] + frac * (pt2.velocity.coords[6] - pt1.velocity.coords[6]);
        out.velocity.v = pt1.velocity.coords[7] + frac * (pt2.velocity.coords[7] - pt1.velocity.coords[7]);
        out.velocity.w = pt1.velocity.coords[8] + frac * (pt2.velocity.coords[8] - pt1.velocity.coords[8]);
        
        out.acceleration.x = pt1.acceleration.coords[0] + frac * (pt2.acceleration.coords[0] - pt1.acceleration.coords[0]);
        out.acceleration.y = pt1.acceleration.coords[1] + frac * (pt2.acceleration.coords[1] - pt1.acceleration.coords[1]);
        out.acceleration.z = pt1.acceleration.coords[2] + frac * (pt2.acceleration.coords[2] - pt1.acceleration.coords[2]);
        out.acceleration.a = pt1.acceleration.coords[3] + frac * (pt2.acceleration.coords[3] - pt1.acceleration.coords[3]);
        out.acceleration.b = pt1.acceleration.coords[4] + frac * (pt2.acceleration.coords[4] - pt1.acceleration.coords[4]);
        out.acceleration.c = pt1.acceleration.coords[5] + frac * (pt2.acceleration.coords[5] - pt1.acceleration.coords[5]);
        out.acceleration.u = pt1.acceleration.coords[6] + frac * (pt2.acceleration.coords[6] - pt1.acceleration.coords[6]);
        out.acceleration.v = pt1.acceleration.coords[7] + frac * (pt2.acceleration.coords[7] - pt1.acceleration.coords[7]);
        out.acceleration.w = pt1.acceleration.coords[8] + frac * (pt2.acceleration.coords[8] - pt1.acceleration.coords[8]);
    }
    
    out.block_index = pt1.blockIndex;
    out.segment_index = pt1.segmentIndex;
    out.motion_type = static_cast<uint8_t>(pt1.motionType);
    out.is_interpolated = true;
    
    return out;
}

bool FfiTrajectoryGenerator::get_block_range_for_lines(
    std::size_t start_line, 
    std::size_t end_line, 
    std::size_t& out_start_block, 
    std::size_t& out_end_block) const 
{
    if (points_.empty()) {
        return false;
    }
    
    // Find first and last block indices within the line range
    bool found_start = false;
    bool found_end = false;
    
    for (const auto& pt : points_) {
        size_t line = static_cast<size_t>(pt.blockIndex);
        
        if (line >= start_line && line <= end_line) {
            if (!found_start) {
                out_start_block = static_cast<size_t>(pt.blockIndex);
                found_start = true;
            }
            out_end_block = static_cast<size_t>(pt.blockIndex);
            found_end = true;
        }
    }
    
    return found_start && found_end;
}

// Factory functions -----------------------------------------------------------

std::unique_ptr<FfiParser> new_parser() {
    return std::make_unique<FfiParser>();
}

std::unique_ptr<FfiInterpreter> new_interpreter() {
    return std::make_unique<FfiInterpreter>();
}

std::unique_ptr<FfiTrajectoryGenerator> new_trajectory_generator() {
    return std::make_unique<FfiTrajectoryGenerator>();
}

} // namespace gcode_ffi
