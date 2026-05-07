//! Trajectory management and interaction
//!
//! Manages the computed trajectory data, handles point selection,
//! and provides query interfaces for visualization.

use bevy::prelude::*;

use crate::gcode::{TrajectoryGenerator, MachineConfig, KinematicsType};

pub struct TrajectoryPlugin;

impl Plugin for TrajectoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrajectoryData::default())
           .insert_resource(InitialTrajectoryComputed(false))
           .add_event::<TrajectoryUpdateEvent>()
           .add_event::<PointClickEvent>()
           .add_systems(Update, (handle_trajectory_updates, compute_initial_trajectory));
    }
}

/// Flag to track if initial trajectory has been computed
#[derive(Resource)]
struct InitialTrajectoryComputed(bool);

/// Event triggered when trajectory needs to be recalculated
#[derive(Event)]
pub struct TrajectoryUpdateEvent {
    /// Optional line range filter
    pub selected_lines: Option<(usize, usize)>,
}

/// Event triggered when a trajectory point is clicked
#[derive(Event)]
pub struct PointClickEvent {
    pub point_index: usize,
}

/// A point on the trajectory with full information
#[derive(Clone, Debug)]
pub struct TrajectoryPoint {
    /// Position in 3D space
    pub position: Vec3,
    
    /// Time from start of program
    pub time: f32,
    
    /// Feed rate at this point (mm/min)
    pub feed_rate: f32,
    
    /// Source line number in GCode
    pub line_number: usize,
    
    /// Is this a rapid move
    pub is_rapid: bool,
    
    /// Is this a move or dwell
    pub is_motion: bool,
}

/// Complete trajectory data
#[derive(Resource, Default)]
pub struct TrajectoryData {
    /// Desired trajectory (from GCode)
    pub desired: Vec<TrajectoryPoint>,
    
    /// Actual trajectory (from machine feedback)
    pub actual: Vec<TrajectoryPoint>,
    
    /// Simulated trajectory (from simulation)
    pub simulated: Vec<TrajectoryPoint>,
    
    /// Motion segments for display
    pub segments: Vec<MotionSegmentInfo>,
    
    /// Bounds of the trajectory (min, max)
    pub bounds: (Vec3, Vec3),
    
    /// Total duration in seconds
    pub total_duration: f32,
    
    /// Total distance in mm
    pub total_distance: f32,
    
    /// Currently selected point index
    pub selected_point: Option<usize>,
    
    /// Selected point info for display
    pub selected_point_info: Option<PointInfo>,
    
    /// Time resolution for trajectory computation
    pub time_resolution: f32,
    
    /// Max spatial deviation (alternative to time resolution)
    pub max_deviation: Option<f32>,
}

/// Information about a motion segment
#[derive(Clone, Debug)]
pub struct MotionSegmentInfo {
    pub start_index: usize,
    pub end_index: usize,
    pub line_number: usize,
    pub is_rapid: bool,
    pub feed_rate: f32,
}

/// Detailed info about a selected point
#[derive(Clone, Debug)]
pub struct PointInfo {
    pub position: Vec3,
    pub time: f32,
    pub line_number: usize,
    pub gcode_line: String,
    pub feed_rate: f32,
}

impl TrajectoryData {
    /// Compute trajectory from GCode content
    pub fn compute_from_gcode(
        &mut self,
        gcode_content: &str,
        config: &MachineConfig,
        selected_lines: Option<(usize, usize)>,
        time_resolution: f32,
        max_deviation: Option<f32>,
    ) -> Result<(), String> {
        self.time_resolution = time_resolution;
        self.max_deviation = max_deviation;
        
        // Filter content by selected lines if specified
        let content_to_parse = if let Some((start, end)) = selected_lines {
            gcode_content
                .lines()
                .enumerate()
                .filter(|(idx, _)| *idx >= start && *idx <= end)
                .map(|(_, line)| line)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            gcode_content.to_string()
        };
        
        // Use the C API to generate trajectory
        let mut generator = TrajectoryGenerator::new().map_err(|e| format!("{:?}", e))?;
        let points = generator.generate_from_gcode(
            &content_to_parse,
            config.max_velocity_linear,
            config.max_acceleration,
            config.max_jerk,
            time_resolution as f64,
        ).map_err(|e| format!("{:?}", e))?;
        
        // Convert to our internal format
        self.desired.clear();
        self.segments.clear();
        
        let mut min_bound = Vec3::splat(f32::MAX);
        let mut max_bound = Vec3::splat(f32::MIN);
        let mut total_dist = 0.0;
        let mut last_pos = Vec3::ZERO;
        
        let line_offset = selected_lines.map(|(s, _)| s).unwrap_or(0);
        
        for (i, pt) in points.iter().enumerate() {
            let pos = Vec3::new(pt.position.x as f32, pt.position.y as f32, pt.position.z as f32);
            
            min_bound = min_bound.min(pos);
            max_bound = max_bound.max(pos);
            
            if i > 0 {
                total_dist += (pos - last_pos).length();
            }
            last_pos = pos;
            
            // Compute feed rate from velocity
            let vel = Vec3::new(pt.velocity.x as f32, pt.velocity.y as f32, pt.velocity.z as f32);
            let feed_rate = vel.length() * 60.0; // Convert to mm/min
            
            self.desired.push(TrajectoryPoint {
                position: pos,
                time: pt.time as f32,
                feed_rate,
                line_number: pt.block_index as usize + line_offset,
                is_rapid: pt.motion_type == 0, // 0 = Rapid
                // motion_type: 0=Rapid, 1=Linear, 2=ArcCW, 3=ArcCCW, 4=Spline
                is_motion: pt.motion_type <= 4,
            });
        }
        
        self.bounds = (min_bound, max_bound);
        self.total_distance = total_dist;
        self.total_duration = points.last().map(|p| p.time as f32).unwrap_or(0.0);
        
        // Build segment info
        self.build_segments();
        
        Ok(())
    }
    
    fn build_segments(&mut self) {
        if self.desired.is_empty() {
            return;
        }
        
        let mut current_segment = MotionSegmentInfo {
            start_index: 0,
            end_index: 0,
            line_number: self.desired[0].line_number,
            is_rapid: self.desired[0].is_rapid,
            feed_rate: self.desired[0].feed_rate,
        };
        
        for (i, point) in self.desired.iter().enumerate().skip(1) {
            if point.line_number != current_segment.line_number || 
               point.is_rapid != current_segment.is_rapid {
                current_segment.end_index = i - 1;
                self.segments.push(current_segment.clone());
                
                current_segment = MotionSegmentInfo {
                    start_index: i,
                    end_index: i,
                    line_number: point.line_number,
                    is_rapid: point.is_rapid,
                    feed_rate: point.feed_rate,
                };
            }
        }
        
        current_segment.end_index = self.desired.len() - 1;
        self.segments.push(current_segment);
    }
    
    /// Get point info at a specific time
    pub fn point_at_time(&self, time: f32) -> Option<TrajectoryPoint> {
        if self.desired.is_empty() {
            return None;
        }
        
        // Binary search for the time
        let idx = self.desired.partition_point(|p| p.time < time);
        
        if idx == 0 {
            return Some(self.desired[0].clone());
        }
        if idx >= self.desired.len() {
            return Some(self.desired.last().unwrap().clone());
        }
        
        // Interpolate between points
        let p0 = &self.desired[idx - 1];
        let p1 = &self.desired[idx];
        let t = (time - p0.time) / (p1.time - p0.time);
        
        Some(TrajectoryPoint {
            position: p0.position.lerp(p1.position, t),
            time,
            feed_rate: p0.feed_rate + (p1.feed_rate - p0.feed_rate) * t,
            line_number: p0.line_number,
            is_rapid: p0.is_rapid,
            is_motion: p0.is_motion,
        })
    }
    
    /// Find the closest point to a ray (for picking)
    pub fn closest_point_to_ray(
        &self,
        ray_origin: Vec3,
        ray_direction: Vec3,
        max_distance: f32,
    ) -> Option<(usize, f32)> {
        let mut best_index = None;
        let mut best_dist = max_distance;
        
        for (i, point) in self.desired.iter().enumerate() {
            // Distance from point to ray
            let to_point = point.position - ray_origin;
            let proj_length = to_point.dot(ray_direction);
            
            if proj_length < 0.0 {
                continue; // Point is behind ray
            }
            
            let closest_on_ray = ray_origin + ray_direction * proj_length;
            let dist = (point.position - closest_on_ray).length();
            
            if dist < best_dist {
                best_dist = dist;
                best_index = Some(i);
            }
        }
        
        best_index.map(|i| (i, best_dist))
    }
    
    /// Select a point and populate info
    pub fn select_point(&mut self, index: usize, gcode_content: &str) {
        if index >= self.desired.len() {
            return;
        }
        
        let point = &self.desired[index];
        let line = gcode_content
            .lines()
            .nth(point.line_number)
            .unwrap_or("")
            .to_string();
        
        self.selected_point = Some(index);
        self.selected_point_info = Some(PointInfo {
            position: point.position,
            time: point.time,
            line_number: point.line_number,
            gcode_line: line,
            feed_rate: point.feed_rate,
        });
    }
    
    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_point = None;
        self.selected_point_info = None;
    }
    
    /// Set actual trajectory (from external source)
    pub fn set_actual_trajectory(&mut self, points: Vec<TrajectoryPoint>) {
        self.actual = points;
    }
    
    /// Get center of trajectory bounds
    pub fn center(&self) -> Vec3 {
        (self.bounds.0 + self.bounds.1) / 2.0
    }
    
    /// Get size of trajectory bounds
    pub fn size(&self) -> Vec3 {
        self.bounds.1 - self.bounds.0
    }
}

fn handle_trajectory_updates(
    mut events: EventReader<TrajectoryUpdateEvent>,
    mut trajectory_data: ResMut<TrajectoryData>,
    mut state: ResMut<crate::app::AppState>,
) {
    for event in events.read() {
        // Recompute trajectory when events are received
        let limits = &state.config.machine.motion_limits;
        let volume = &state.config.machine.work_volume;
        
        let config = MachineConfig {
            min_x: volume.min_x,
            max_x: volume.max_x,
            min_y: volume.min_y,
            max_y: volume.max_y,
            min_z: volume.min_z,
            max_z: volume.max_z,
            min_a: -360.0,
            max_a: 360.0,
            min_b: -360.0,
            max_b: 360.0,
            min_c: -360.0,
            max_c: 360.0,
            max_velocity_linear: limits.rapid_feed_rate,
            max_velocity_angular: 3600.0,
            max_acceleration: limits.max_acceleration_xy,
            max_jerk: limits.max_jerk,
            default_feed_rate: limits.default_feed_rate,
            rapid_feed_rate: limits.rapid_feed_rate,
            use_metric: true,
            kinematics_type: KinematicsType::Cartesian,
        };
        
        let time_resolution = state.config.trajectory.time_resolution as f32;
        if let Err(e) = trajectory_data.compute_from_gcode(
            &state.gcode_content,
            &config,
            event.selected_lines,
            time_resolution,
            None,
        ) {
            log::error!("Failed to compute trajectory: {}", e);
        } else {
            log::info!(
                "Trajectory computed: {} points, duration: {:.2}s",
                trajectory_data.desired.len(),
                trajectory_data.total_duration
            );

            // Check for simple logic issues and report to AppState diagnostics
            let mut issues = Vec::new();
            // Check for any points where linear motion was marked incorrectly
            for p in &trajectory_data.desired {
                if p.is_motion == false && p.is_rapid == false {
                    // Non-rapid non-motion point - suspicious
                    issues.push(format!("Suspicious point at t={:.4}s (line {}): not marked as motion", p.time, p.line_number + 1));
                }
            }
            state.diagnostics = issues;
        }
    }
}

/// System to compute trajectory on startup if gcode content is present
fn compute_initial_trajectory(
    mut computed: ResMut<InitialTrajectoryComputed>,
    mut trajectory_data: ResMut<TrajectoryData>,
    state: Res<crate::app::AppState>,
) {
    if computed.0 || state.gcode_content.is_empty() {
        return;
    }
    
    computed.0 = true;
    
    // Compute trajectory from initial gcode content
    let limits = &state.config.machine.motion_limits;
    let volume = &state.config.machine.work_volume;
    
    let config = MachineConfig {
        min_x: volume.min_x,
        max_x: volume.max_x,
        min_y: volume.min_y,
        max_y: volume.max_y,
        min_z: volume.min_z,
        max_z: volume.max_z,
        min_a: -360.0,
        max_a: 360.0,
        min_b: -360.0,
        max_b: 360.0,
        min_c: -360.0,
        max_c: 360.0,
        max_velocity_linear: limits.rapid_feed_rate,
        max_velocity_angular: 3600.0,
        max_acceleration: limits.max_acceleration_xy,
        max_jerk: limits.max_jerk,
        default_feed_rate: limits.default_feed_rate,
        rapid_feed_rate: limits.rapid_feed_rate,
        use_metric: true,
        kinematics_type: KinematicsType::Cartesian,
    };
    
    let time_resolution = state.config.trajectory.time_resolution as f32;
    if let Err(e) = trajectory_data.compute_from_gcode(
        &state.gcode_content,
        &config,
        None,
        time_resolution,
        None,
    ) {
        log::error!("Failed to compute initial trajectory: {}", e);
    } else {
        log::info!(
            "Initial trajectory computed: {} points, duration: {:.2}s",
            trajectory_data.desired.len(),
            trajectory_data.total_duration
        );
    }
}

/// Compute trajectory with max spatial deviation constraint
pub fn compute_with_max_deviation(
    gcode_content: &str,
    config: &MachineConfig,
    max_deviation: f32,
) -> Result<Vec<TrajectoryPoint>, String> {
    // Start with coarse resolution
    let mut resolution = 0.1_f64;
    let mut points = Vec::new();
    
    loop {
        let mut generator = TrajectoryGenerator::new().map_err(|e| format!("{:?}", e))?;
        let gcode_points = generator.generate_from_gcode(
            gcode_content,
            config.max_velocity_linear as f64,
            config.max_acceleration as f64,
            config.max_jerk as f64,
            resolution,
        )
            .map_err(|e| format!("{:?}", e))?;
            
        points = gcode_points.into_iter()
            .map(|p| {
                let vel = Vec3::new(p.velocity.x as f32, p.velocity.y as f32, p.velocity.z as f32);
                TrajectoryPoint {
                    position: Vec3::new(p.position.x as f32, p.position.y as f32, p.position.z as f32),
                    time: p.time as f32,
                    feed_rate: vel.length() * 60.0,
                    line_number: p.block_index as usize,
                    is_rapid: p.motion_type == 0, // 0 = Rapid
                    // motion_type: 0=Rapid, 1=Linear, 2=ArcCW, 3=ArcCCW, 4=Spline
                    is_motion: p.motion_type <= 4,
                }
            })
            .collect();
        
        // Check maximum deviation between consecutive points
        let mut max_dev = 0.0_f32;
        for window in points.windows(2) {
            let dist = (window[1].position - window[0].position).length();
            max_dev = max_dev.max(dist);
        }
        
        if max_dev <= max_deviation {
            break;
        }
        
        resolution *= 0.5;
        
        if resolution < 0.0001 {
            break; // Prevent infinite loop
        }
    }
    
    Ok(points)
}
