//! Simulation system for GCode playback
//!
//! Provides real-time and stepped simulation with feed rate override,
//! bidirectional time control, and trajectory tracking.

use bevy::prelude::*;

use crate::app::AppState;
use crate::trajectory::{TrajectoryData, TrajectoryPoint};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            simulation_update,
        ));
    }
}

/// Simulation state stored in AppState
#[derive(Clone, Default)]
pub struct SimulationState {
    /// Current simulation time in seconds
    pub current_time: f32,
    
    /// Is simulation playing
    pub playing: bool,
    
    /// Playback speed multiplier (can be negative)
    pub speed: f32,
    
    /// Feed rate override (0.0 to 2.0+, 1.0 = 100%)
    pub feed_rate_override: f32,
    
    /// Current position from simulation
    pub current_position: Vec3,
    
    /// Current line number being executed
    pub current_line: usize,
    
    /// Loop playback
    pub loop_playback: bool,
    
    /// Show simulation panel
    pub show_panel: bool,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            playing: false,
            speed: 1.0,
            feed_rate_override: 1.0,
            current_position: Vec3::ZERO,
            current_line: 0,
            loop_playback: false,
            show_panel: true,
        }
    }
}

fn simulation_update(
    time: Res<Time>,
    mut state: ResMut<AppState>,
    trajectory: Res<TrajectoryData>,
) {
    let sim = &mut state.simulation;

    if sim.playing {
        // Update time with speed and feed rate override
        let effective_speed = sim.speed * sim.feed_rate_override;
        sim.current_time += time.delta_seconds() * effective_speed;

        // Clamp or loop time
        if sim.current_time >= trajectory.total_duration {
            if sim.loop_playback {
                sim.current_time = 0.0;
            } else {
                sim.current_time = trajectory.total_duration;
                sim.playing = false;
            }
        } else if sim.current_time < 0.0 {
            if sim.loop_playback {
                sim.current_time = trajectory.total_duration;
            } else {
                sim.current_time = 0.0;
                sim.playing = false;
            }
        }
    }

    // Always keep derived fields in sync (even when paused/scrubbing).
    sync_simulation_state(sim, &trajectory);
}

/// Keep `current_position` and `current_line` consistent with `current_time`.
pub(crate) fn sync_simulation_state(sim: &mut SimulationState, trajectory: &TrajectoryData) {
    if let Some(point) = trajectory.point_at_time(sim.current_time) {
        sim.current_position = point.position;
        sim.current_line = point.line_number;
    }
}

// NOTE: Simulation UI is rendered by `UiPlugin` so it can live inside the
// middle-bottom pane without being duplicated.

/// Step to the next GCode segment
pub(crate) fn step_to_next_segment(sim: &mut SimulationState, trajectory: &TrajectoryData) {
    let current_line = sim.current_line;
    
    // Find next point with a different line number
    for point in &trajectory.desired {
        if point.time > sim.current_time && point.line_number != current_line {
            sim.current_time = point.time;
            return;
        }
    }
    
    // If no next segment, go to end
    sim.current_time = trajectory.total_duration;
}

/// Step to the previous GCode segment
pub(crate) fn step_to_previous_segment(sim: &mut SimulationState, trajectory: &TrajectoryData) {
    let current_line = sim.current_line;
    
    // Find previous point with a different line number
    for point in trajectory.desired.iter().rev() {
        if point.time < sim.current_time && point.line_number != current_line {
            // Go to the start of this segment
            for p in &trajectory.desired {
                if p.line_number == point.line_number {
                    sim.current_time = p.time;
                    return;
                }
            }
        }
    }
    
    // If no previous segment, go to start
    sim.current_time = 0.0;
}

/// Generate simulated trajectory with dynamics
pub fn generate_simulated_trajectory(
    desired: &[TrajectoryPoint],
    max_acceleration: f32,
    max_jerk: f32,
) -> Vec<TrajectoryPoint> {
    if desired.is_empty() {
        return Vec::new();
    }
    
    let mut simulated = Vec::with_capacity(desired.len());
    
    let mut velocity = Vec3::ZERO;
    let mut position = desired[0].position;
    let mut time = 0.0_f32;
    
    for (i, point) in desired.iter().enumerate() {
        if i == 0 {
            simulated.push(point.clone());
            continue;
        }
        
        let target = point.position;
        let dt = point.time - desired[i - 1].time;
        
        if dt <= 0.0 {
            simulated.push(point.clone());
            continue;
        }
        
        // Simple velocity-limited following
        let direction = target - position;
        let distance = direction.length();
        
        if distance > 0.0 {
            let desired_velocity = direction / dt;
            let max_velocity_change = max_acceleration * dt;
            
            // Limit acceleration
            let velocity_change = desired_velocity - velocity;
            if velocity_change.length() > max_velocity_change {
                velocity = velocity + velocity_change.normalize() * max_velocity_change;
            } else {
                velocity = desired_velocity;
            }
            
            position = position + velocity * dt;
        }
        
        time += dt;
        
        simulated.push(TrajectoryPoint {
            position,
            time,
            feed_rate: point.feed_rate,
            line_number: point.line_number,
            is_rapid: point.is_rapid,
            is_motion: point.is_motion,
        });
    }
    
    simulated
}
