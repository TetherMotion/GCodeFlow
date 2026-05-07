//! 2D Plot view for trajectory analysis
//!
//! Provides interactive 2D plotting of motion parameters (position, velocity, jerk)
//! over time or as cartesian projections.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Legend, Line, Plot, PlotPoints, Points};

use crate::app::AppState;
use crate::gcode::{Position, TrajectoryPoint, TrajectoryGenerator};
use crate::trajectory::TrajectoryData;

pub struct PlotViewPlugin;

impl Plugin for PlotViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlotViewState::default())
            .add_systems(Update, plot_view_system);
    }
}

/// Plot view state
#[derive(Resource)]
pub struct PlotViewState {
    /// Whether the plot window is open
    pub open: bool,
    
    /// Plot mode: Time or Cartesian2D
    pub mode: PlotMode,
    
    /// Selected curves for time mode
    pub show_position: bool,
    pub show_velocity: bool,
    pub show_acceleration: bool,
    pub show_jerk: bool,
    
    /// Selected axis for time mode (0=X, 1=Y, 2=Z, 3=E)
    pub selected_axis: usize,
    
    /// Selected axes for cartesian mode (X, Y by default)
    pub cartesian_axis_x: usize,
    pub cartesian_axis_y: usize,
    
    /// Cached plot data
    cached_data: Option<CachedPlotData>,
    
    /// Selected point for detail view
    pub selected_point_index: Option<usize>,
    
    /// Auto-fit enabled
    pub auto_fit: bool,
    
    /// Last selected gcode lines (for cache invalidation)
    last_selected_lines: Option<(usize, usize)>,
}

impl Default for PlotViewState {
    fn default() -> Self {
        Self {
            open: false,
            mode: PlotMode::Time,
            show_position: true,
            show_velocity: true,
            show_acceleration: false,
            show_jerk: false,
            selected_axis: 0, // X axis
            cartesian_axis_x: 0, // X
            cartesian_axis_y: 1, // Y
            cached_data: None,
            selected_point_index: None,
            auto_fit: true,
            last_selected_lines: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlotMode {
    /// Plot position/velocity/jerk vs time
    Time,
    /// Plot 2D cartesian projection
    Cartesian2D,
}

/// Cached plot data for efficient rendering
struct CachedPlotData {
    /// Time-based data (stored as [f64; 2] for egui_plot)
    time_position: Vec<[f64; 2]>,
    time_velocity: Vec<[f64; 2]>,
    time_acceleration: Vec<[f64; 2]>,
    time_jerk: Vec<[f64; 2]>,
    
    /// Cartesian projection data
    cartesian_analytical: Vec<[f64; 2]>,
    cartesian_sampled_1ms: Vec<[f64; 2]>,
    
    /// Full trajectory points for queries
    trajectory_points: Vec<TrajectoryPoint>,
    
    /// Bounds for auto-fit
    time_bounds: (f64, f64, f64, f64), // (min_x, max_x, min_y, max_y)
    cartesian_bounds: (f64, f64, f64, f64),
}

fn plot_view_system(
    mut contexts: EguiContexts,
    mut plot_state: ResMut<PlotViewState>,
    app_state: Res<AppState>,
    trajectory: Res<TrajectoryData>,
) {
    if !plot_state.open {
        return;
    }
    
    let ctx = contexts.ctx_mut();
    
    // Check if we need to update cached data
    let selected_lines = None; // TODO: get from editor state
    let needs_update = plot_state.cached_data.is_none() 
        || plot_state.last_selected_lines != selected_lines;
    
    if needs_update {
        update_plot_data(&mut plot_state, &app_state, &trajectory, selected_lines);
        plot_state.last_selected_lines = selected_lines;

        // If a native plot window exists and is open, push the latest cached data
        if let Some(ref cached) = plot_state.cached_data.as_ref() {
            if let Some(handle) = crate::native_plot_window::current_handle() {
                if handle.is_open.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = handle.sender.send(crate::native_plot_window::PlotMessage::Cartesian(cached.cartesian_sampled_1ms.clone()));
                    let _ = handle.sender.send(crate::native_plot_window::PlotMessage::Time(cached.time_position.clone()));
                }
            }
        }
    }
    
    let mut window_open = plot_state.open;
    
    egui::Window::new("2D Plot Analysis")
        .open(&mut window_open)
        .default_width(800.0)
        .default_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Top controls
            ui.horizontal(|ui| {
                ui.selectable_value(&mut plot_state.mode, PlotMode::Time, "Time-based");
                ui.selectable_value(&mut plot_state.mode, PlotMode::Cartesian2D, "2D Projection");
                ui.separator();
                ui.checkbox(&mut plot_state.auto_fit, "Auto-fit");
            });
            
            ui.separator();
            
            // Mode-specific controls
            match plot_state.mode {
                PlotMode::Time => {
                    ui.horizontal(|ui| {
                        ui.label("Curves:");
                        ui.checkbox(&mut plot_state.show_position, "Position");
                        ui.checkbox(&mut plot_state.show_velocity, "Velocity");
                        ui.checkbox(&mut plot_state.show_acceleration, "Acceleration");
                        ui.checkbox(&mut plot_state.show_jerk, "Jerk");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Axis:");
                        ui.selectable_value(&mut plot_state.selected_axis, 0, "X");
                        ui.selectable_value(&mut plot_state.selected_axis, 1, "Y");
                        ui.selectable_value(&mut plot_state.selected_axis, 2, "Z");
                        ui.selectable_value(&mut plot_state.selected_axis, 3, "E");
                    });
                }
                PlotMode::Cartesian2D => {
                    ui.horizontal(|ui| {
                        ui.label("Horizontal axis:");
                        ui.selectable_value(&mut plot_state.cartesian_axis_x, 0, "X");
                        ui.selectable_value(&mut plot_state.cartesian_axis_x, 1, "Y");
                        ui.selectable_value(&mut plot_state.cartesian_axis_x, 2, "Z");
                        
                        ui.separator();
                        
                        ui.label("Vertical axis:");
                        ui.selectable_value(&mut plot_state.cartesian_axis_y, 0, "X");
                        ui.selectable_value(&mut plot_state.cartesian_axis_y, 1, "Y");
                        ui.selectable_value(&mut plot_state.cartesian_axis_y, 2, "Z");
                    });
                }
            }
            
            ui.separator();
            
            // Plot area
            let plot_response = match plot_state.mode {
                PlotMode::Time => render_time_plot(ui, &plot_state),
                PlotMode::Cartesian2D => render_cartesian_plot(ui, &plot_state),
            };
            
            // Handle point selection from plot interaction
            if let Some(plot_resp) = plot_response {
                if let Some(pointer_pos) = plot_resp.response.hover_pos() {
                    // Find nearest point
                    if let Some(ref cached) = plot_state.cached_data {
                        let nearest_idx = find_nearest_point(&cached.trajectory_points, 
                            pointer_pos, &plot_state);
                        
                        if plot_resp.response.clicked() {
                            plot_state.selected_point_index = Some(nearest_idx);
                        }
                    }
                }
            }
            
            // Show detail popup if point is selected
            if let Some(idx) = plot_state.selected_point_index {
                if let Some(ref cached) = plot_state.cached_data {
                    if idx < cached.trajectory_points.len() {
                        show_point_detail_popup(ui, &cached.trajectory_points[idx]);
                    }
                }
            }
        });
    
    plot_state.open = window_open;
}

fn update_plot_data(
    plot_state: &mut PlotViewState,
    app_state: &AppState,
    trajectory: &TrajectoryData,
    selected_lines: Option<(usize, usize)>,
) {
    // Create a trajectory generator and generate high-resolution samples
    let mut generator = match TrajectoryGenerator::new() {
        Ok(g) => g,
        Err(e) => {
            log::error!("Failed to create trajectory generator: {:?}", e);
            return;
        }
    };
    
    // Use machine config from app_state
    let max_vel = app_state.config.machine.motion_limits.max_velocity_xy;
    let max_accel = app_state.config.machine.motion_limits.max_acceleration_xy;
    let max_jerk = app_state.config.machine.motion_limits.max_jerk;
    
    // Always use the full G-code content when generating the trajectory so that the
    // interpreter has the proper context (initial position) even when the user selects
    // a subset of lines. We'll filter the generated samples to only include points that
    // belong to the selected line range (if any).
    let full_gcode = app_state.gcode_content.clone();

    // Generate trajectory points from the full program
    let trajectory_points = match generator.generate_from_gcode(
        &full_gcode,
        max_vel,
        max_accel,
        max_jerk,
        0.001, // 1ms resolution for base trajectory
    ) {
        Ok(points) => points,
        Err(e) => {
            log::error!("Failed to generate trajectory: {:?}", e);
            return;
        }
    };

    if trajectory_points.is_empty() {
        return;
    }

    // If there is a selected line range, map it to block indices so we can filter samples
    let selected_block_range: Option<(usize, usize)> = if let Some((start_line, end_line)) = selected_lines {
        generator.get_block_range_for_lines(start_line, end_line)
    } else {
        None
    };
    
    let axis_idx = plot_state.selected_axis;
    
    // Sample at 2 points per pixel for analytical curve (approximate with 1ms for now)
    // In production, this would be calculated based on plot width in pixels
    let analytical_samples = generator.sample_at_interval(0.001);
    
    // Sample at 1ms for discrete points
    let discrete_samples = generator.sample_at_interval(0.001);
    
    // Build time-based curves
    let mut time_position = Vec::new();
    let mut time_velocity = Vec::new();
    let mut time_acceleration = Vec::new();
    let mut time_jerk = Vec::new();
    
    let mut min_time = f64::MAX;
    let mut max_time = f64::MIN;
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    
    for pt in &analytical_samples {
        // If user selected a line range, only include points whose block index falls
        // within the mapped block range (preserves interpreter context while showing
        // only the selected lines).
        if let Some((bstart, bend)) = selected_block_range {
            let blk = pt.block_index as isize;
            if blk < bstart as isize || blk > bend as isize {
                continue;
            }
        }

        let time = pt.time;
        let pos = get_axis_value(&pt.position, axis_idx);
        let vel = get_axis_value(&pt.velocity, axis_idx);
        let acc = get_axis_value(&pt.acceleration, axis_idx);

        // Compute jerk (derivative of acceleration)
        // Simplified: use finite difference if we have enough points
        let jerk = 0.0; // TODO: implement proper jerk calculation

        time_position.push([time, pos]);
        time_velocity.push([time, vel]);
        time_acceleration.push([time, acc]);
        time_jerk.push([time, jerk]);

        min_time = min_time.min(time);
        max_time = max_time.max(time);
        min_val = min_val.min(pos.min(vel).min(acc));
        max_val = max_val.max(pos.max(vel).max(acc));
    }    
    // Build cartesian projection curves
    let mut cartesian_analytical = Vec::new();
    let mut cartesian_sampled_1ms = Vec::new();
    
    let cart_x_idx = plot_state.cartesian_axis_x;
    let cart_y_idx = plot_state.cartesian_axis_y;
    
    let mut min_cart_x = f64::MAX;
    let mut max_cart_x = f64::MIN;
    let mut min_cart_y = f64::MAX;
    let mut max_cart_y = f64::MIN;
    
    for pt in &analytical_samples {
        if let Some((bstart, bend)) = selected_block_range {
            let blk = pt.block_index as isize;
            if blk < bstart as isize || blk > bend as isize {
                continue;
            }
        }

        let x = get_axis_value(&pt.position, cart_x_idx);
        let y = get_axis_value(&pt.position, cart_y_idx);

        cartesian_analytical.push([x, y]);

        min_cart_x = min_cart_x.min(x);
        max_cart_x = max_cart_x.max(x);
        min_cart_y = min_cart_y.min(y);
        max_cart_y = max_cart_y.max(y);
    }

    // Add discrete samples for 1ms interval
    for pt in &discrete_samples {
        if let Some((bstart, bend)) = selected_block_range {
            let blk = pt.block_index as isize;
            if blk < bstart as isize || blk > bend as isize {
                continue;
            }
        }

        let x = get_axis_value(&pt.position, cart_x_idx);
        let y = get_axis_value(&pt.position, cart_y_idx);
        cartesian_sampled_1ms.push([x, y]);
    }
    
    plot_state.cached_data = Some(CachedPlotData {
        time_position,
        time_velocity,
        time_acceleration,
        time_jerk,
        cartesian_analytical,
        cartesian_sampled_1ms,
        trajectory_points,
        time_bounds: (min_time, max_time, min_val, max_val),
        cartesian_bounds: (min_cart_x, max_cart_x, min_cart_y, max_cart_y),
    });
}

fn render_time_plot(ui: &mut egui::Ui, plot_state: &PlotViewState) -> Option<egui_plot::PlotResponse<()>> {
    let cached = plot_state.cached_data.as_ref()?;
    
    let mut plot = Plot::new("time_plot")
        .legend(Legend::default())
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .x_axis_label("Time (s)")
        .y_axis_label(format!("{} Axis", axis_name(plot_state.selected_axis)));
    
    if plot_state.auto_fit {
        plot = plot.auto_bounds(egui::Vec2b::TRUE);
    }
    
    let response = plot.show(ui, |plot_ui| {
        if plot_state.show_position && !cached.time_position.is_empty() {
            let line = Line::new(PlotPoints::from(cached.time_position.clone()))
                .color(egui::Color32::from_rgb(0, 120, 215))
                .name("Position (mm)");
            plot_ui.line(line);
        }
        
        if plot_state.show_velocity && !cached.time_velocity.is_empty() {
            let line = Line::new(PlotPoints::from(cached.time_velocity.clone()))
                .color(egui::Color32::from_rgb(0, 200, 0))
                .name("Velocity (mm/s)");
            plot_ui.line(line);
        }
        
        if plot_state.show_acceleration && !cached.time_acceleration.is_empty() {
            let line = Line::new(PlotPoints::from(cached.time_acceleration.clone()))
                .color(egui::Color32::from_rgb(255, 165, 0))
                .name("Acceleration (mm/s²)");
            plot_ui.line(line);
        }
        
        if plot_state.show_jerk && !cached.time_jerk.is_empty() {
            let line = Line::new(PlotPoints::from(cached.time_jerk.clone()))
                .color(egui::Color32::from_rgb(255, 0, 0))
                .name("Jerk (mm/s³)");
            plot_ui.line(line);
        }
    });
    
    Some(response)
}

fn render_cartesian_plot(ui: &mut egui::Ui, plot_state: &PlotViewState) -> Option<egui_plot::PlotResponse<()>> {
    let cached = plot_state.cached_data.as_ref()?;
    
    let mut plot = Plot::new("cartesian_plot")
        .legend(Legend::default())
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .x_axis_label(format!("{} Axis (mm)", axis_name(plot_state.cartesian_axis_x)))
        .y_axis_label(format!("{} Axis (mm)", axis_name(plot_state.cartesian_axis_y)))
        .data_aspect(1.0); // Equal aspect ratio for cartesian plots
    
    if plot_state.auto_fit {
        plot = plot.auto_bounds(egui::Vec2b::TRUE);
    }
    
    let response = plot.show(ui, |plot_ui| {
        if !cached.cartesian_analytical.is_empty() {
            let line = Line::new(PlotPoints::from(cached.cartesian_analytical.clone()))
                .color(egui::Color32::from_rgb(0, 120, 215))
                .name("Analytical Path");
            plot_ui.line(line);
        }
        
        if !cached.cartesian_sampled_1ms.is_empty() {
            let points = Points::new(PlotPoints::from(cached.cartesian_sampled_1ms.clone()))
                .color(egui::Color32::from_rgb(255, 0, 0))
                .radius(2.0)
                .name("1ms Samples");
            plot_ui.points(points);
        }
    });
    
    Some(response)
}

fn find_nearest_point(
    points: &[TrajectoryPoint],
    hover_pos: egui::Pos2,
    plot_state: &PlotViewState,
) -> usize {
    if points.is_empty() {
        return 0;
    }
    
    let mut min_dist = f64::MAX;
    let mut nearest_idx = 0;
    
    for (i, pt) in points.iter().enumerate() {
        let dist = match plot_state.mode {
            PlotMode::Time => {
                let axis_val = get_axis_value(&pt.position, plot_state.selected_axis);
                let dx = pt.time - hover_pos.x as f64;
                let dy = axis_val - hover_pos.y as f64;
                dx * dx + dy * dy
            }
            PlotMode::Cartesian2D => {
                let x = get_axis_value(&pt.position, plot_state.cartesian_axis_x);
                let y = get_axis_value(&pt.position, plot_state.cartesian_axis_y);
                let dx = x - hover_pos.x as f64;
                let dy = y - hover_pos.y as f64;
                dx * dx + dy * dy
            }
        };
        
        if dist < min_dist {
            min_dist = dist;
            nearest_idx = i;
        }
    }
    
    nearest_idx
}

fn show_point_detail_popup(ui: &mut egui::Ui, point: &TrajectoryPoint) {
    egui::Window::new("Point Details")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.heading("Trajectory Point");
            ui.separator();
            
            ui.label(format!("Time: {:.3} s", point.time));
            ui.separator();
            
            ui.label("Position (mm):");
            ui.label(format!("  X: {:.3}", point.position.x));
            ui.label(format!("  Y: {:.3}", point.position.y));
            ui.label(format!("  Z: {:.3}", point.position.z));
            ui.separator();
            
            ui.label("Velocity (mm/s):");
            ui.label(format!("  X: {:.3}", point.velocity.x));
            ui.label(format!("  Y: {:.3}", point.velocity.y));
            ui.label(format!("  Z: {:.3}", point.velocity.z));
            let vel_mag = (point.velocity.x.powi(2) + point.velocity.y.powi(2) + point.velocity.z.powi(2)).sqrt();
            ui.label(format!("  Magnitude: {:.3}", vel_mag));
            ui.separator();
            
            ui.label("Acceleration (mm/s²):");
            ui.label(format!("  X: {:.3}", point.acceleration.x));
            ui.label(format!("  Y: {:.3}", point.acceleration.y));
            ui.label(format!("  Z: {:.3}", point.acceleration.z));
            let acc_mag = (point.acceleration.x.powi(2) + point.acceleration.y.powi(2) + point.acceleration.z.powi(2)).sqrt();
            ui.label(format!("  Magnitude: {:.3}", acc_mag));
            ui.separator();
            
            ui.label(format!("Block Index: {}", point.block_index));
            ui.label(format!("Segment Index: {}", point.segment_index));
        });
}

fn axis_name(axis: usize) -> &'static str {
    match axis {
        0 => "X",
        1 => "Y",
        2 => "Z",
        3 => "E",
        _ => "?",
    }
}

fn get_axis_value(pos: &Position, axis: usize) -> f64 {
    match axis {
        0 => pos.x,
        1 => pos.y,
        2 => pos.z,
        3 => pos.a, // Using A for E (extruder)
        _ => 0.0,
    }
}
