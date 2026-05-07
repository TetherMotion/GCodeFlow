//! Main UI system with settings panels
//!
//! Provides the main UI layout, settings menus, and configuration panels.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::app::AppState;
use crate::camera::{CameraController, ViewPreset};
use crate::config::{
    AppConfig, KinematicsType, ProjectionMode, TrackingMode, TraceColorMode,
};
use crate::plot_view::PlotViewState;
use crate::trajectory::{TrajectoryData, TrajectoryUpdateEvent};
use crate::editor::{EditorState, EditorChangeEvent};

use crossbeam_channel;


pub struct UiPlugin;

#[derive(Resource, Default, Clone)]
pub struct BenchmarkState {
    pub open: bool,
    pub running: bool,
    pub use_current_buffer: bool,
    pub selected_file: Option<std::path::PathBuf>,
    pub last_result: Option<gcodeflow::benchmark::BenchmarkResult>,
    pub last_error: Option<String>,
    // Receiver for an in-flight benchmark task
    pub pending: Option<crossbeam_channel::Receiver<Result<gcodeflow::benchmark::BenchmarkResult, String>>>,
}



impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::default())
            .insert_resource(BenchmarkState::default())
            // Render the entire egui hierarchy in a single system so panel order is
            // deterministic: top menu (fixed) -> side panels -> center (3D+controls).
            .add_systems(Update, ui_root_system);
    }
}

/// UI state for panel visibility
#[derive(Resource, Default)]
pub struct UiState {
    pub show_settings: bool,
    pub show_machine_settings: bool,
    pub show_trace_settings: bool,
    pub show_tool_settings: bool,
    pub show_interpolation_settings: bool,
    pub show_about: bool,
    pub show_point_info: bool,
}

fn ui_root_system(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    mut ui_state: ResMut<UiState>,
    mut plot_state: ResMut<PlotViewState>,
    trajectory: Res<TrajectoryData>,
    mut trajectory_events: EventWriter<TrajectoryUpdateEvent>,
    mut camera_query: Query<(&mut CameraController, &mut Transform, &mut Projection)>,
    mut editor_state: ResMut<EditorState>,
    mut editor_change_events: EventWriter<EditorChangeEvent>,
    mut middle_split: Local<f32>,
    mut commands: Commands,
    native_plot_handle: Option<Res<crate::native_plot_window::NativePlotWindowHandle>>,
    mut benchmark_state: ResMut<BenchmarkState>,
) {
    let ctx = contexts.ctx_mut();

    // Default split: prioritize 3D view.
    if *middle_split <= 0.01 {
        *middle_split = 0.78;
    }

    // Superstructure (non-resizable)
    top_menu_impl(
        ctx,
        &mut *state,
        &mut *ui_state,
        &mut *plot_state,
        &mut trajectory_events,
        &*trajectory,
        &mut camera_query,
        &mut *benchmark_state,
    );

    // Draw benchmark modal if open
    crate::benchmark_ui::draw_benchmark_window(ctx, &mut *state, &mut *editor_state, &mut *benchmark_state);

    // If a benchmark task is pending, poll for completion and update the state
    if let Some(rx) = benchmark_state.pending.as_ref() {
        match rx.try_recv() {
            Ok(Ok(res)) => {
                benchmark_state.last_result = Some(res);
                benchmark_state.last_error = None;
                benchmark_state.running = false;
                benchmark_state.pending = None;
            }
            Ok(Err(e)) => {
                benchmark_state.last_error = Some(e);
                benchmark_state.last_result = None;
                benchmark_state.running = false;
                benchmark_state.pending = None;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                // still running
            }
            Err(_) => {
                benchmark_state.last_error = Some("Benchmark channel closed unexpectedly".to_string());
                benchmark_state.running = false;
                benchmark_state.pending = None;
            }
        }
    }

    // Main structure (resizable)
    info_panel_impl(ctx, &*trajectory, &mut *state);

    // Right: editor is always right-most.
    crate::editor::editor_panel(
        ctx,
        &mut *state,
        &mut *editor_state,
        &mut editor_change_events,
        &mut trajectory_events,
    );

    // Optional right-side settings panels (stack to the left of the editor).
    settings_panel_impl(ctx, &mut *state, &mut *ui_state);
    machine_settings_panel_impl(ctx, &mut *state, &mut *ui_state);
    trace_settings_panel_impl(ctx, &mut *state, &mut *ui_state, &mut trajectory_events);
    tool_settings_panel_impl(ctx, &mut *state, &mut *ui_state);
    interpolation_settings_panel_impl(ctx, &mut *state, &mut *ui_state, &mut trajectory_events);

    // Middle: 3D renderer region (top) + simulation controls (bottom)
    center_panel(ctx, &mut *state, &*trajectory, &mut *middle_split);
}

fn top_menu_impl(
    ctx: &egui::Context,
    state: &mut AppState,
    ui_state: &mut UiState,
    plot_state: &mut PlotViewState,
    trajectory_events: &mut EventWriter<TrajectoryUpdateEvent>,
    trajectory: &TrajectoryData,
    camera_query: &mut Query<(&mut CameraController, &mut Transform, &mut Projection)>,
    benchmark_state: &mut BenchmarkState,
) {
    let win_w = ctx.available_rect().width();

    egui::TopBottomPanel::top("menu_bar")
        .frame(egui::Frame::none()) // make the panel edge-to-edge so it fills full window width
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(win_w);
                egui::menu::bar(ui, |ui| {
                    // File menu
                    ui.menu_button("File", |ui| {
                        if ui.button("📂 Open...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("GCode", &["gcode", "nc", "ngc", "g"])
                                .add_filter("All files", &["*"])
                                .pick_file()
                            {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        state.gcode_content = content;
                                        state.current_file = Some(path);
                                        state.modified = false;
                                        trajectory_events.send(TrajectoryUpdateEvent { selected_lines: None });
                                    }
                                    Err(e) => log::error!("Failed to open file: {}", e),
                                }
                            }
                            ui.close_menu();
                        }

                        if ui.button("💾 Save").clicked() {
                            if let Some(ref path) = state.current_file {
                                if let Err(e) = std::fs::write(path, &state.gcode_content) {
                                    log::error!("Failed to save: {}", e);
                                } else {
                                    state.modified = false;
                                }
                            }
                            ui.close_menu();
                        }

                        if ui.button("💾 Save As...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("GCode", &["gcode", "nc", "ngc"]) 
                                .save_file()
                            {
                                if let Err(e) = std::fs::write(&path, &state.gcode_content) {
                                    log::error!("Failed to save: {}", e);
                                } else {
                                    state.current_file = Some(path);
                                    state.modified = false;
                                }
                            }
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button("📷 Export Screenshot...").clicked() {
                            // Screenshot export would go here
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button("🚪 Exit").clicked() {
                            std::process::exit(0);
                        }
                    }); // File

                    // Edit menu
                    ui.menu_button("Edit", |ui| {
                        if ui.button("⚙ Settings...").clicked() {
                            ui_state.show_settings = true;
                            ui.close_menu();
                        }
                    }); // Edit

                    // View menu
                    ui.menu_button("View", |ui| {
                        ui.checkbox(&mut state.config.visualization.show_grid, "Show Grid");
                        ui.checkbox(&mut state.config.visualization.show_axes, "Show Axes");
                        ui.checkbox(&mut state.config.trace.show_desired, "Show Desired Trace");
                        ui.checkbox(&mut state.config.trace.show_actual, "Show Actual Trace");
                        ui.checkbox(&mut state.config.simulation.show_simulated, "Show Simulated Trace");
                        ui.checkbox(&mut state.config.tool.show_tool, "Show Tool");
                    }); // View

                    // Tools menu
                    ui.menu_button("Tools", |ui| {
                        if ui.button("📈 G-Code Benchmark...").clicked() {
                            benchmark_state.open = true;
                            ui.close_menu();
                        }
                    }); // Tools

                    // Projection menu
                    ui.menu_button("Projection", |ui| {
                        if ui.selectable_label(matches!(state.config.camera.projection, ProjectionMode::Perspective), "Perspective").clicked() {
                            state.config.camera.projection = ProjectionMode::Perspective; ui.close_menu(); }
                        if ui.selectable_label(matches!(state.config.camera.projection, ProjectionMode::Orthographic), "Orthographic").clicked() {
                            state.config.camera.projection = ProjectionMode::Orthographic; ui.close_menu(); }

                        ui.separator(); ui.label("2D Views:");

                        if ui.selectable_label(matches!(state.config.camera.projection, ProjectionMode::View2D_XY), "2D Top (XY plane)").clicked() {
                            state.config.camera.projection = ProjectionMode::View2D_XY; ui.close_menu(); }
                        if ui.selectable_label(matches!(state.config.camera.projection, ProjectionMode::View2D_XZ), "2D Front (XZ plane)").clicked() {
                            state.config.camera.projection = ProjectionMode::View2D_XZ; ui.close_menu(); }
                        if ui.selectable_label(matches!(state.config.camera.projection, ProjectionMode::View2D_YZ), "2D Side (YZ plane)").clicked() {
                            state.config.camera.projection = ProjectionMode::View2D_YZ; ui.close_menu(); }
                    }); // Projection

                    ui.separator();

                    // 2D Plot button -> spawn separate native window
                    if ui.button("📊 2D Plot...").clicked() {
                        let _ = crate::native_plot_window::spawn_native_plot_window();
                        ui.close_menu();
                    }

                    // Benchmark quick-launch
                    if ui.button("⚡ Run Benchmark...").clicked() {
                        benchmark_state.open = true;
                        benchmark_state.last_result = None;
                        benchmark_state.last_error = None;
                        benchmark_state.use_current_buffer = true;
                        ui.close_menu();
                    }

                    // Camera presets
                    ui.menu_button("Camera Preset", |ui| {
                        for (mut controller, mut _transform, mut _projection) in camera_query.iter_mut() {
                            let center = trajectory.center();
                            let size = trajectory.size().length().max(100.0);

                            if ui.button("Isometric").clicked() { controller.set_preset(ViewPreset::Isometric); controller.focus = center; controller.radius = size * 1.5; ui.close_menu(); }
                            if ui.button("Top (XY)").clicked() { controller.set_preset(ViewPreset::Top); controller.focus = center; controller.radius = size * 1.5; ui.close_menu(); }
                            if ui.button("Front (XZ)").clicked() { controller.set_preset(ViewPreset::Front); controller.focus = center; controller.radius = size * 1.5; ui.close_menu(); }
                            if ui.button("Right (YZ)").clicked() { controller.set_preset(ViewPreset::Right); controller.focus = center; controller.radius = size * 1.5; ui.close_menu(); }
                            if ui.button("Fit to Model").clicked() { controller.set_preset(ViewPreset::Isometric); controller.focus = center; controller.radius = size * 1.5; ui.close_menu(); }
                        }
                    }); // Camera Preset

                    // Machine menu
                    ui.menu_button("Machine", |ui| {
                        if ui.button("⚙ Machine Settings...").clicked() { ui_state.show_machine_settings = true; ui.close_menu(); }
                        if ui.button("🎨 Trace Settings...").clicked() { ui_state.show_trace_settings = true; ui.close_menu(); }
                        if ui.button("🔧 Tool Settings...").clicked() { ui_state.show_tool_settings = true; ui.close_menu(); }
                        if ui.button("📐 Interpolation Settings...").clicked() { ui_state.show_interpolation_settings = true; ui.close_menu(); }
                    }); // Machine

                    // Simulation menu
                    ui.menu_button("Simulation", |ui| {
                        let sim = &mut state.simulation;
                        if ui.button(if sim.playing { "⏸ Pause" } else { "▶ Play" }).clicked() { sim.playing = !sim.playing; ui.close_menu(); }
                        if ui.button("⏮ Reset").clicked() { sim.current_time = 0.0; ui.close_menu(); }
                        ui.separator(); ui.checkbox(&mut sim.loop_playback, "Loop Playback");
                    }); // Simulation

                    // Help menu
                    ui.menu_button("Help", |ui| { if ui.button("About GCodeFlow").clicked() { ui_state.show_about = true; ui.close_menu(); } }); // Help

                    // Right-aligned status with fixed minimum width to prevent jumping
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if state.modified { ui.label("● Modified"); } else { ui.label("✓ Saved  "); }
                        if let Some(ref path) = state.current_file { ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string()); } else { ui.label("(no file)"); }
                    }); // status
                }); // menu::bar
            }); // horizontal
        }); // panel
}

fn settings_panel_impl(ctx: &egui::Context, state: &mut AppState, ui_state: &mut UiState) {
    if !ui_state.show_settings {
        return;
    }
    
    egui::SidePanel::right("settings_panel")
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Settings");
                if ui.button("Close").clicked() {
                    ui_state.show_settings = false;
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| { 
                ui.collapsing("View", |ui| {
                    ui.checkbox(&mut state.config.visualization.show_grid, "Show Grid");
                    ui.checkbox(&mut state.config.visualization.show_axes, "Show Axes");
                    ui.checkbox(&mut state.config.visualization.show_bounds, "Show Bounds");
                });
                
                ui.collapsing("Camera", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Projection:");
                        egui::ComboBox::from_id_source("projection")
                            .selected_text(format!("{:?}", state.config.camera.projection))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut state.config.camera.projection,
                                    ProjectionMode::Perspective,
                                    "Perspective"
                                );
                                ui.selectable_value(
                                    &mut state.config.camera.projection,
                                    ProjectionMode::Orthographic,
                                    "Orthographic"
                                );
                            });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Tracking Mode:");
                        egui::ComboBox::from_id_source("tracking")
                            .selected_text(format!("{:?}", state.config.camera.tracking_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut state.config.camera.tracking_mode,
                                    TrackingMode::None,
                                    "None (Free)"
                                );
                                ui.selectable_value(
                                    &mut state.config.camera.tracking_mode,
                                    TrackingMode::ToolEndpoint,
                                    "Follow Tool"
                                );
                                ui.selectable_value(
                                    &mut state.config.camera.tracking_mode,
                                    TrackingMode::DesiredPath,
                                    "Follow Desired Path"
                                );
                                ui.selectable_value(
                                    &mut state.config.camera.tracking_mode,
                                    TrackingMode::ActualPath,
                                    "Follow Actual Path"
                                );
                                ui.selectable_value(
                                    &mut state.config.camera.tracking_mode,
                                    TrackingMode::SimulatedPath,
                                    "Follow Simulated Path"
                                );
                            });
                    });
                    
                    ui.add(egui::Slider::new(&mut state.config.camera.fov, 30.0..=120.0)
                        .text("FOV"));
                    
                    ui.add(egui::Slider::new(&mut state.config.camera.orbit_speed, 0.1..=5.0)
                        .text("Orbit Sensitivity"));
                    
                    ui.add(egui::Slider::new(&mut state.config.camera.pan_speed, 0.1..=5.0)
                        .text("Pan Sensitivity"));
                    
                    ui.add(egui::Slider::new(&mut state.config.camera.zoom_speed, 0.1..=5.0)
                        .text("Zoom Sensitivity"));
                });
                
                ui.collapsing("Editor", |ui| {
                    ui.add(egui::Slider::new(&mut state.config.editor.font_size, 8.0..=24.0)
                        .text("Font Size"));
                    
                    ui.checkbox(&mut state.config.editor.show_line_numbers, "Show Line Numbers");
                    ui.checkbox(&mut state.config.editor.word_wrap, "Word Wrap");
                    ui.checkbox(&mut state.config.autosave, "Auto Save");
                    
                    if state.config.autosave {
                        ui.add(egui::Slider::new(&mut state.config.autosave_interval, 10.0..=300.0)
                            .text("Autosave Interval (s)"));
                    }
                });
                
                ui.collapsing("Trajectory", |ui| {
                    ui.add(egui::Slider::new(&mut state.config.trajectory.time_resolution, 0.001..=0.1)
                        .text("Time Resolution (s)")
                        .logarithmic(true));
                    
                    ui.checkbox(&mut state.config.trajectory.use_max_deviation, "Use Max Deviation");
                    
                    if state.config.trajectory.use_max_deviation {
                        ui.add(egui::Slider::new(&mut state.config.trajectory.max_deviation, 0.01..=1.0)
                            .text("Max Deviation (mm)"));
                    }
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Save Settings").clicked() {
                        if let Err(e) = state.config.save(std::path::Path::new("gcodeflow.yaml")) {
                            log::error!("Failed to save settings: {}", e);
                        }
                    }
                    
                    if ui.button("Reset to Defaults").clicked() {
                        state.config = AppConfig::default();
                    }
                });
            });
        });
}

fn machine_settings_panel_impl(ctx: &egui::Context, state: &mut AppState, ui_state: &mut UiState) {
    if !ui_state.show_machine_settings {
        return;
    }
    
    egui::SidePanel::right("machine_settings_panel")
        .resizable(true)
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Machine Settings");
                if ui.button("Close").clicked() {
                    ui_state.show_machine_settings = false;
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| { 
                let machine = &mut state.config.machine;
                
                // Kinematics type
                ui.horizontal(|ui| {
                    ui.label("Kinematics Type:");
                    egui::ComboBox::from_id_source("kinematics_type")
                        .selected_text(format!("{:?}", machine.kinematics_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::Cartesian, "Cartesian");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::CoreXY, "CoreXY");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::CoreXZ, "CoreXZ");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::Delta, "Delta");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::Scara, "SCARA");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::FiveAxisBC, "5-Axis (B/C)");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::FiveAxisAC, "5-Axis (A/C)");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::FiveAxisAB, "5-Axis (A/B)");
                            ui.selectable_value(&mut machine.kinematics_type, KinematicsType::Custom, "Custom");
                        });
                });
                
                ui.separator();
                
                // Work envelope
                ui.collapsing("Work Envelope", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("X Range:");
                        ui.add(egui::DragValue::new(&mut machine.work_volume.min_x).prefix("Min: ").speed(1.0));
                        ui.add(egui::DragValue::new(&mut machine.work_volume.max_x).prefix("Max: ").speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y Range:");
                        ui.add(egui::DragValue::new(&mut machine.work_volume.min_y).prefix("Min: ").speed(1.0));
                        ui.add(egui::DragValue::new(&mut machine.work_volume.max_y).prefix("Max: ").speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Z Range:");
                        ui.add(egui::DragValue::new(&mut machine.work_volume.min_z).prefix("Min: ").speed(1.0));
                        ui.add(egui::DragValue::new(&mut machine.work_volume.max_z).prefix("Max: ").speed(1.0));
                    });
                });
                
                // Feed rates
                ui.collapsing("Feed Rates", |ui| {
                    ui.add(egui::Slider::new(&mut machine.motion_limits.max_velocity_xy, 100.0..=10000.0)
                        .text("Max XY Velocity (mm/s)"));
                    ui.add(egui::Slider::new(&mut machine.motion_limits.default_feed_rate, 100.0..=5000.0)
                        .text("Default Feed Rate (mm/min)"));
                    ui.add(egui::Slider::new(&mut machine.motion_limits.rapid_feed_rate, 1000.0..=20000.0)
                        .text("Rapid Rate (mm/min)"));
                });
                
                // Acceleration
                ui.collapsing("Acceleration", |ui| {
                    ui.add(egui::Slider::new(&mut machine.motion_limits.max_acceleration_xy, 100.0..=10000.0)
                        .text("Max XY Acceleration (mm/s²)"));
                    ui.add(egui::Slider::new(&mut machine.motion_limits.max_jerk, 10.0..=1000.0)
                        .text("Max Jerk (mm/s³)"));
                });
                
                // Kinematics-specific settings (simplified - using default values)
                match machine.kinematics_type {
                    KinematicsType::Delta => {
                        ui.collapsing("Delta Parameters", |ui| {
                            ui.label("Delta rod length: 300mm (default)");
                            ui.label("Delta radius: 150mm (default)");
                            ui.label("Edit values in gcodeflow.yaml to customize.");
                        });
                    }
                    KinematicsType::Scara => {
                        ui.collapsing("SCARA Parameters", |ui| {
                            ui.label("Arm 1 length: 200mm (default)");
                            ui.label("Arm 2 length: 200mm (default)");
                            ui.label("Edit values in gcodeflow.yaml to customize.");
                        });
                    }
                    _ => {}
                }
                
                // Units (stored in axes config)
                // TODO: Add default_units_mm and absolute_positioning to MachineSettings if needed
            });
        });
}

fn trace_settings_panel_impl(
    ctx: &egui::Context,
    state: &mut AppState,
    ui_state: &mut UiState,
    trajectory_events: &mut EventWriter<TrajectoryUpdateEvent>,
) {
    if !ui_state.show_trace_settings {
        return;
    }
    
    egui::SidePanel::right("trace_settings_panel")
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Trace Settings");
                if ui.button("Close").clicked() {
                    ui_state.show_trace_settings = false;
                }
            });
            ui.separator();

            let trace = &mut state.config.trace;
            let mut changed = false;
            
            ui.collapsing("Visibility", |ui| { 
                changed |= ui.checkbox(&mut trace.show_desired, "Show Desired Trace").changed();
                changed |= ui.checkbox(&mut trace.show_actual, "Show Actual Trace").changed();
                changed |= ui.checkbox(&mut trace.show_simulated, "Show Simulated Trace").changed();
                changed |= ui.checkbox(&mut trace.show_rapids, "Show Rapid Moves").changed();
                changed |= ui.checkbox(&mut trace.show_retracts, "Show Retracts").changed();
            });
            
            ui.collapsing("Color Mode", |ui| {
                ui.label("How to color the trajectory:");
                
                let modes = [
                    (TraceColorMode::ByMoveType, "By Move Type", "Color by rapid/feed/arc"),
                    (TraceColorMode::BySpeed, "By Speed", "Color gradient based on feed rate"),
                    (TraceColorMode::ByZHeight, "By Z Height", "Color gradient based on Z position"),
                    (TraceColorMode::ByTime, "By Time", "Color gradient over time"),
                    (TraceColorMode::ByAcceleration, "By Acceleration", "Color by motion intensity"),
                    (TraceColorMode::ByAccuracy, "By Accuracy", "Show deviation from ideal"),
                ];
                
                for (mode, label, description) in modes {
                    if ui.selectable_label(trace.color_mode == mode, label).on_hover_text(description).clicked() {
                        trace.color_mode = mode;
                        changed = true;
                    }
                }
                
                ui.separator();
                
                // Show relevant color pickers based on mode
                match trace.color_mode {
                    TraceColorMode::ByMoveType => {
                        changed |= color_edit(ui, "Feed", &mut trace.desired_feed_color);
                        changed |= color_edit(ui, "Rapid", &mut trace.desired_rapid_color);
                        changed |= color_edit(ui, "Arc", &mut trace.arc_color);
                    }
                    TraceColorMode::BySpeed => {
                        changed |= color_edit(ui, "Low Speed", &mut trace.low_speed_color);
                        changed |= color_edit(ui, "High Speed", &mut trace.high_speed_color);
                    }
                    TraceColorMode::ByZHeight => {
                        changed |= color_edit(ui, "Low Z", &mut trace.low_z_color);
                        changed |= color_edit(ui, "High Z", &mut trace.high_z_color);
                    }
                    TraceColorMode::ByTime => {
                        changed |= color_edit(ui, "Start", &mut trace.start_time_color);
                        changed |= color_edit(ui, "End", &mut trace.end_time_color);
                    }
                    TraceColorMode::ByAcceleration => {
                        changed |= color_edit(ui, "Low Acceleration", &mut trace.low_accel_color);
                        changed |= color_edit(ui, "High Acceleration", &mut trace.high_accel_color);
                    }
                    TraceColorMode::ByAccuracy => {
                        changed |= color_edit(ui, "High Accuracy", &mut trace.high_accuracy_color);
                        changed |= color_edit(ui, "Low Accuracy", &mut trace.low_accuracy_color);
                    }
                }
            });
            
            ui.collapsing("Other Colors", |ui| {
                changed |= color_edit(ui, "Actual", &mut trace.actual_color);
                changed |= color_edit(ui, "Simulated", &mut trace.simulated_color);
            });
            
            ui.collapsing("Line Style", |ui| {
                changed |= ui.add(egui::Slider::new(&mut trace.line_width, 0.5..=100.0)
                    .text("Line Width")).changed();
                changed |= ui.checkbox(&mut trace.use_tubes, "Use 3D Tubes").changed();
                if trace.use_tubes {
                    changed |= ui.add(egui::Slider::new(&mut trace.tube_radius, 0.1..=10.0)
                        .text("Tube Radius")).changed();
                }
                changed |= ui.checkbox(&mut trace.show_points, "Show Points").changed();
                if trace.show_points {
                    changed |= ui.add(egui::Slider::new(&mut trace.point_size, 1.0..=50.0)
                        .text("Point Size")).changed();
                }
            });
            
            ui.collapsing("Render Quality", |ui| {
                ui.label("Controls point density in 3D view (separate from trajectory computation).");
                ui.add_space(4.0);
                
                let render = &mut trace.render;
                
                changed |= ui.checkbox(
                    &mut render.adaptive_step_size,
                    "Adaptive Step Size"
                ).on_hover_text(
                    "Automatically adjust point density based on view scale.\n\
                     Uses perspective to calculate what mm corresponds to a fraction of a pixel."
                ).changed();
                
                if render.adaptive_step_size {
                    ui.horizontal(|ui| {
                        ui.label("Pixel fraction:");
                        changed |= ui.add(
                            egui::Slider::new(&mut render.pixel_fraction, 0.1..=1.0)
                                .fixed_decimals(2)
                        ).on_hover_text(
                            "Fraction of a pixel to use as max deviation.\n\
                             0.33 = 1/3 pixel (default, high quality)\n\
                             1.0 = 1 pixel (faster, lower quality)"
                        ).changed();
                    });
                    
                    // Show computed deviation
                    ui.horizontal(|ui| {
                        ui.label("Computed deviation:");
                        ui.monospace(format!("{:.4} mm", render.computed_deviation));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Max deviation:");
                        changed |= ui.add(
                            egui::Slider::new(&mut render.max_deviation, 0.001..=10.0)
                                .logarithmic(true)
                                .suffix(" mm")
                        ).on_hover_text(
                            "Maximum allowed deviation from ideal path.\n\
                             Lower = more points = smoother curves"
                        ).changed();
                    });
                }
                
                ui.add_space(4.0);
                ui.separator();
                ui.label("Point limits:");
                
                ui.horizontal(|ui| {
                    ui.label("Min points/segment:");
                    changed |= ui.add(
                        egui::DragValue::new(&mut render.min_points_per_segment)
                            .range(2..=100)
                    ).changed();
                });
                
                ui.horizontal(|ui| {
                    ui.label("Max points/segment:");
                    changed |= ui.add(
                        egui::DragValue::new(&mut render.max_points_per_segment)
                            .range(100..=100000)
                    ).changed();
                });
                
                ui.add_space(4.0);
                ui.separator();
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    if ui.button("Ultra").on_hover_text("Maximum quality, pixel-perfect").clicked() {
                        render.adaptive_step_size = true;
                        render.pixel_fraction = 0.1;
                        render.max_points_per_segment = 50000;
                        changed = true;
                    }
                    if ui.button("High").on_hover_text("High quality (default)").clicked() {
                        render.adaptive_step_size = true;
                        render.pixel_fraction = 0.33;
                        render.max_points_per_segment = 10000;
                        changed = true;
                    }
                    if ui.button("Medium").on_hover_text("Balanced performance").clicked() {
                        render.adaptive_step_size = true;
                        render.pixel_fraction = 0.5;
                        render.max_points_per_segment = 5000;
                        changed = true;
                    }
                    if ui.button("Fast").on_hover_text("Faster rendering, less detail").clicked() {
                        render.adaptive_step_size = false;
                        render.max_deviation = 0.5;
                        render.max_points_per_segment = 2000;
                        changed = true;
                    }
                });
            });
            
            // Trigger trajectory re-render when settings change
            if changed {
                trajectory_events.send(TrajectoryUpdateEvent { selected_lines: None });
            }
        });
}

fn tool_settings_panel_impl(ctx: &egui::Context, state: &mut AppState, ui_state: &mut UiState) {
    if !ui_state.show_tool_settings {
        return;
    }
    
    egui::SidePanel::right("tool_settings_panel")
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tool Settings");
                if ui.button("Close").clicked() {
                    ui_state.show_tool_settings = false;
                }
            });
            ui.separator();

            let tool = &mut state.config.tool;
            
            ui.checkbox(&mut tool.show_tool, "Show Tool");
            
            ui.separator();
            
            ui.add(egui::Slider::new(&mut tool.scale, 0.1..=5.0)
                .text("Scale"));
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Tool Model:");
                egui::ComboBox::from_id_source("tool_model")
                    .selected_text(format!("{:?}", tool.tool_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::EndMill, "End Mill (Pointed)");
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::BallNose, "Ball End Mill");
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::VBit, "V-Bit");
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::Drill, "Drill");
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::Nozzle3DPrinter, "3D Printer Nozzle");
                        ui.selectable_value(&mut tool.tool_type, crate::config::ToolType::Custom, "Custom (STEP/STL)");
                    });
            });
            
            ui.separator();
            
            ui.label("Custom Model:");
            ui.horizontal(|ui| {
                let path_text = tool.custom_model_path.as_deref().unwrap_or("None");
                ui.label(path_text);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("3D Models", &["step", "stp", "stl"])
                        .pick_file()
                    {
                        tool.custom_model_path = Some(path.to_string_lossy().to_string());
                    }
                }
            });
            
            ui.separator();
            
            ui.label("Tool Color:");
            let mut color = egui::Color32::from_rgba_unmultiplied(
                (tool.tool_color[0] * 255.0) as u8,
                (tool.tool_color[1] * 255.0) as u8,
                (tool.tool_color[2] * 255.0) as u8,
                (tool.tool_color[3] * 255.0) as u8,
            );
            if ui.color_edit_button_srgba(&mut color).changed() {
                tool.tool_color = [
                    color.r() as f32 / 255.0,
                    color.g() as f32 / 255.0,
                    color.b() as f32 / 255.0,
                    color.a() as f32 / 255.0,
                ];
            }
        });
}

fn info_panel_impl(
    ctx: &egui::Context,
    trajectory: &TrajectoryData,
    state: &mut AppState,
) {
    egui::SidePanel::left("info_panel")
        .resizable(true)    // Allow dragging to resize
        .default_width(250.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Trajectory Info");
            ui.separator();
            
            ui.label(format!("Points: {}", trajectory.desired.len()));
            ui.label(format!("Segments: {}", trajectory.segments.len()));
            ui.label(format!("Duration: {:.3}s", trajectory.total_duration));
            ui.label(format!("Distance: {:.1}mm", trajectory.total_distance));
            
            ui.separator();
            
            ui.label("Bounds:");
            let min = trajectory.bounds.0;
            let max = trajectory.bounds.1;
            // Format bounds, using ∞ for extreme values
            let format_bound = |v: f32| -> String {
                if v >= f32::MAX * 0.5 { "∞".to_string() }
                else if v <= f32::MIN * 0.5 { "-∞".to_string() }
                else { format!("{:.1}", v) }
            };
            if trajectory.desired.is_empty() {
                ui.label("  (no trajectory)");
            } else {
                ui.label(format!("  X: {} to {}", format_bound(min.x), format_bound(max.x)));
                ui.label(format!("  Y: {} to {}", format_bound(min.y), format_bound(max.y)));
                ui.label(format!("  Z: {} to {}", format_bound(min.z), format_bound(max.z)));
            }
            
            ui.separator();
            
            ui.heading("Current Position");
            // Use monospace with fixed-width formatting to prevent jumping
            let pos = state.simulation.current_position;
            ui.monospace(format!("X: {:>10.3}", pos.x));
            ui.monospace(format!("Y: {:>10.3}", pos.y));
            ui.monospace(format!("Z: {:>10.3}", pos.z));
            
            ui.separator();

            // Diagnostics (logic issues) - show prominently if any
            if !state.diagnostics.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(255, 80, 80), "Logic Issues Detected:");
                for issue in &state.diagnostics {
                    ui.label(format!("• {}", issue));
                }
                ui.separator();
            }
            
            ui.monospace(format!("Time: {:>10.3}s", state.simulation.current_time));
            ui.monospace(format!("Line: {:>10}", state.simulation.current_line + 1));
            
            ui.separator();
            
            // Z Level Filter section
            ui.heading("Z Level Filter");
            let z_filter = &mut state.config.visualization.z_filter;
            let current_z = state.simulation.current_position.z;
            
            ui.checkbox(&mut z_filter.enabled, "Enable Z filtering");
            
            if z_filter.enabled {
                ui.add_space(4.0);
                ui.label(format!("Reference Z: {:.3} mm", current_z));
                ui.add_space(4.0);
                
                // Below control
                ui.horizontal(|ui| {
                    ui.label("Show below:");
                    let mut below_all = z_filter.below.is_infinite();
                    if ui.checkbox(&mut below_all, "All").changed() {
                        z_filter.below = if below_all { f32::INFINITY } else { 10.0 };
                    }
                });
                if !z_filter.below.is_infinite() {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.add(egui::DragValue::new(&mut z_filter.below)
                            .speed(0.5)
                            .range(0.0..=1000.0)
                            .suffix(" mm"));
                    });
                }
                
                // Above control
                ui.horizontal(|ui| {
                    ui.label("Show above:");
                    let mut above_all = z_filter.above.is_infinite();
                    if ui.checkbox(&mut above_all, "All").changed() {
                        z_filter.above = if above_all { f32::INFINITY } else { 10.0 };
                    }
                });
                if !z_filter.above.is_infinite() {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.add(egui::DragValue::new(&mut z_filter.above)
                            .speed(0.5)
                            .range(0.0..=1000.0)
                            .suffix(" mm"));
                    });
                }
                
                // Show effective range
                ui.add_space(4.0);
                let lower = if z_filter.below.is_infinite() { "-∞".to_string() } else { format!("{:.1}", current_z - z_filter.below) };
                let upper = if z_filter.above.is_infinite() { "∞".to_string() } else { format!("{:.1}", current_z + z_filter.above) };
                ui.label(format!("Visible range: {} to {} mm", lower, upper));
                
                // Quick presets
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.small_button("±1mm").clicked() {
                        z_filter.below = 1.0;
                        z_filter.above = 1.0;
                    }
                    if ui.small_button("±5mm").clicked() {
                        z_filter.below = 5.0;
                        z_filter.above = 5.0;
                    }
                    if ui.small_button("±10mm").clicked() {
                        z_filter.below = 10.0;
                        z_filter.above = 10.0;
                    }
                    if ui.small_button("All").clicked() {
                        z_filter.below = f32::INFINITY;
                        z_filter.above = f32::INFINITY;
                    }
                });
            }
            
            ui.separator();
            if let Some(ref info) = trajectory.selected_point_info {
                ui.heading("Selected Point");
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.label(format!("X: {:.3}  Y: {:.3}  Z: {:.3}", info.position.x, info.position.y, info.position.z));
                });
                ui.horizontal(|ui| { ui.label("Time:"); ui.label(format!("{:.4}s", info.time)); });
                ui.horizontal(|ui| { ui.label("Line:"); ui.label(format!("{}", info.line_number + 1)); });
                ui.horizontal(|ui| { ui.label("GCode:"); ui.monospace(&info.gcode_line); });
                ui.horizontal(|ui| { ui.label("Feed Rate:"); ui.label(format!("{:.1} mm/min", info.feed_rate)); });
                ui.separator();
            }
            }); // close ScrollArea
        });
}

// NOTE: `Selected Point` floating window was removed in favor of showing selection
// details inside the left info panel to avoid overlapping the 3D renderer.
fn point_info_window(
    _contexts: EguiContexts,
    _trajectory: Res<TrajectoryData>,
    _mut_ui_state: ResMut<UiState>,
) {
    // intentionally left empty - point info is now shown in the `info_panel`
}

fn info_panel(
    mut contexts: EguiContexts,
    trajectory: Res<TrajectoryData>,
    mut state: ResMut<AppState>,
) {
    let ctx = contexts.ctx_mut();
    info_panel_impl(ctx, &*trajectory, &mut *state);
}

/// Top-level UI composer that arranges the full layout:
/// - Top: menu bar (fixed height)
/// - Left: info panel (resizable)
/// - Center: 3D view + simulation controls (resizable split)
/// - Right: editor panel (remains in its own system as well)
fn ui_layout(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    trajectory: Res<TrajectoryData>,
    mut middle_split: Local<f32>,
) {
    let ctx = contexts.ctx_mut();

    // The top menu, left info and right editor panels are separate systems.
    // This system composes the central area (3D view + simulation controls).
    center_panel(ctx, &mut *state, &*trajectory, &mut *middle_split);
}

fn color_edit(ui: &mut egui::Ui, label: &str, color: &mut [f32; 4]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let mut c = egui::Color32::from_rgba_unmultiplied(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        if ui.color_edit_button_srgba(&mut c).changed() {
            *color = [
                c.r() as f32 / 255.0,
                c.g() as f32 / 255.0,
                c.b() as f32 / 255.0,
                c.a() as f32 / 255.0,
            ];
            changed = true;
        }
    });
    changed
}

/// Interpolation settings panel for trajectory generation
fn interpolation_settings_panel_impl(
    ctx: &egui::Context,
    state: &mut AppState,
    ui_state: &mut UiState,
    trajectory_events: &mut EventWriter<TrajectoryUpdateEvent>,
) {
    if !ui_state.show_interpolation_settings {
        return;
    }
    
    egui::SidePanel::right("interpolation_settings_panel")
        .resizable(true)
        .default_width(450.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Interpolation Settings");
                if ui.button("Close").clicked() {
                    ui_state.show_interpolation_settings = false;
                }
            });
            ui.separator();
            
            let mut view_changed = false;
            
            // =========== 3D VIEW SETTINGS ===========
            ui.collapsing("3D View Interpolation", |ui| {
                ui.label("Controls how the trajectory is sampled for 3D visualization.");
                ui.add_space(4.0);
                
                let traj = &mut state.config.trajectory;
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(!traj.use_max_deviation, "Fixed Time Step").clicked() {
                        traj.use_max_deviation = false;
                        view_changed = true;
                    }
                    if ui.selectable_label(traj.use_max_deviation, "Fixed Deviation").clicked() {
                        traj.use_max_deviation = true;
                        view_changed = true;
                    }
                });
                
                ui.add_space(4.0);
                
                if !traj.use_max_deviation {
                    ui.label("Samples the path at regular time intervals.");
                    
                    view_changed |= ui.add(
                        egui::Slider::new(&mut traj.time_resolution, 0.001..=0.1)
                            .text("Time Step (s)")
                            .logarithmic(true)
                    ).changed();
                    
                    ui.label(format!("{:.0} points per second", 1.0 / traj.time_resolution));
                } else {
                    ui.label("Samples the path ensuring maximum deviation from ideal.");
                    
                    view_changed |= ui.add(
                        egui::Slider::new(&mut traj.max_deviation, 0.001..=1.0)
                            .text("Max Deviation (mm)")
                            .logarithmic(true)
                    ).changed();
                }
                
                ui.add_space(4.0);
                ui.label("Quick presets:");
                ui.horizontal(|ui| {
                    if ui.small_button("Fast").on_hover_text("50ms, quick preview").clicked() {
                        traj.time_resolution = 0.05;
                        traj.use_max_deviation = false;
                        view_changed = true;
                    }
                    if ui.small_button("Normal").on_hover_text("10ms, balanced").clicked() {
                        traj.time_resolution = 0.01;
                        traj.use_max_deviation = false;
                        view_changed = true;
                    }
                    if ui.small_button("Fine").on_hover_text("1ms, detailed").clicked() {
                        traj.time_resolution = 0.001;
                        traj.use_max_deviation = false;
                        view_changed = true;
                    }
                    if ui.small_button("Adaptive").on_hover_text("0.1mm deviation").clicked() {
                        traj.max_deviation = 0.1;
                        traj.use_max_deviation = true;
                        view_changed = true;
                    }
                });
            });
            
            ui.add_space(8.0);
            
            // =========== MACHINE OUTPUT SETTINGS ===========
            ui.collapsing("Machine Output Interpolation", |ui| {
                ui.label("Controls how trajectory points are generated for machine commands.");
                ui.add_space(4.0);
                
                let mout = &mut state.config.machine_output;
                
                ui.heading("Point Sampling");
                ui.horizontal(|ui| {
                    if ui.selectable_label(!mout.use_max_deviation, "Fixed Time Step").clicked() {
                        mout.use_max_deviation = false;
                    }
                    if ui.selectable_label(mout.use_max_deviation, "Fixed Deviation").clicked() {
                        mout.use_max_deviation = true;
                    }
                });
                
                ui.add_space(4.0);
                
                if !mout.use_max_deviation {
                    ui.label("Output points at regular time intervals.");
                    
                    ui.add(
                        egui::Slider::new(&mut mout.time_resolution, 0.0001..=0.01)
                            .text("Time Step (s)")
                            .logarithmic(true)
                    );
                    
                    ui.label(format!("{:.0} points per second", 1.0 / mout.time_resolution));
                } else {
                    ui.label("Output points with maximum spatial deviation.");
                    
                    ui.add(
                        egui::Slider::new(&mut mout.max_deviation, 0.001..=0.1)
                            .text("Max Deviation (mm)")
                            .logarithmic(true)
                    );
                }
                
                ui.add_space(8.0);
                ui.separator();
                
                // Arc linearization settings
                ui.heading("Arc Linearization");
                ui.checkbox(&mut mout.linearize_arcs, "Convert G2/G3 arcs to linear segments");
                
                if mout.linearize_arcs {
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!mout.arc_use_deviation, "Fixed Segments").clicked() {
                            mout.arc_use_deviation = false;
                        }
                        if ui.selectable_label(mout.arc_use_deviation, "Max Deviation").clicked() {
                            mout.arc_use_deviation = true;
                        }
                    });
                    
                    ui.add_space(4.0);
                    
                    if !mout.arc_use_deviation {
                        ui.add(
                            egui::Slider::new(&mut mout.arc_segments_per_circle, 8..=360)
                                .text("Segments per 360°")
                        );
                        ui.label(format!("≈ {:.1}° per segment", 360.0 / mout.arc_segments_per_circle as f32));
                    } else {
                        ui.add(
                            egui::Slider::new(&mut mout.arc_max_deviation, 0.001..=0.1)
                                .text("Max Chord Deviation (mm)")
                                .logarithmic(true)
                        );
                        ui.label("Chord deviation from ideal arc");
                    }
                }
                
                ui.add_space(8.0);
                ui.separator();
                ui.label("Machine presets:");
                ui.horizontal(|ui| {
                    if ui.small_button("CNC Router").on_hover_text("High precision, keep arcs").clicked() {
                        mout.time_resolution = 0.001;
                        mout.use_max_deviation = true;
                        mout.max_deviation = 0.005;
                        mout.linearize_arcs = false;
                    }
                    if ui.small_button("3D Printer").on_hover_text("Linearize arcs, moderate precision").clicked() {
                        mout.time_resolution = 0.01;
                        mout.use_max_deviation = true;
                        mout.max_deviation = 0.05;
                        mout.linearize_arcs = true;
                        mout.arc_use_deviation = true;
                        mout.arc_max_deviation = 0.05;
                    }
                    if ui.small_button("Laser").on_hover_text("High speed, linearize").clicked() {
                        mout.time_resolution = 0.0005;
                        mout.use_max_deviation = false;
                        mout.linearize_arcs = true;
                        mout.arc_segments_per_circle = 180;
                        mout.arc_use_deviation = false;
                    }
                });
            });
            
            if view_changed {
                trajectory_events.send(TrajectoryUpdateEvent { selected_lines: None });
            }
        });
}

// ---------------------------------------------------------------------------
// New UI composition helpers
// ---------------------------------------------------------------------------

/// Render the central area: top is the 3D renderer area, bottom is the simulation controls.
fn center_panel(
    ctx: &egui::Context,
    state: &mut AppState,
    trajectory: &TrajectoryData,
    middle_ratio: &mut f32,
) {
    // Important: Keep the center panel background transparent so the Bevy 3D render
    // remains visible in the top region.
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
        let avail = ui.available_size();
        let sep_h = 6.0;
        let min_top = 80.0;
        let min_bottom = 140.0;

        // Clamp split so both regions remain usable, but never allocate more than available.
        let (top_h, bottom_h) = if avail.y <= (min_top + min_bottom + sep_h) {
            // Very small height: keep a small controls area and give the rest to the 3D view.
            let bottom_h = ((avail.y - sep_h).max(0.0) * 0.35).clamp(40.0, (avail.y - sep_h).max(0.0));
            let top_h = (avail.y - sep_h - bottom_h).max(0.0);
            (top_h, bottom_h)
        } else {
            let top_h = (avail.y * *middle_ratio).clamp(min_top, avail.y - min_bottom - sep_h);
            let bottom_h = avail.y - top_h - sep_h;
            (top_h, bottom_h)
        };

        // Top: reserve space for the 3D renderer; do not paint an opaque background.
        let (_top_rect, _top_resp) = ui.allocate_exact_size(egui::Vec2::new(avail.x, top_h), egui::Sense::hover());

        // Draggable separator (stable drag: use drag-start ratio + total delta).
        let sep_id = ui.make_persistent_id("center_split_separator");
        let (sep_rect, sep_resp) = ui.allocate_exact_size(egui::Vec2::new(avail.x, sep_h), egui::Sense::drag());
        ui.painter().rect_filled(
            sep_rect,
            0.0,
            ui.visuals().widgets.noninteractive.bg_fill,
        );

        if sep_resp.drag_started() {
            let start = *middle_ratio;
            ctx.memory_mut(|m| m.data.insert_temp(sep_id, start));
        }
        if sep_resp.dragged() {
            let start = ctx
                .memory(|m| m.data.get_temp::<f32>(sep_id))
                .unwrap_or(*middle_ratio);
            *middle_ratio = (start + sep_resp.drag_delta().y / avail.y).clamp(0.1, 0.95);
        }

        // Bottom: simulation controls framed for readability.
        let (bottom_rect, _bottom_resp) = ui.allocate_exact_size(egui::Vec2::new(avail.x, bottom_h), egui::Sense::hover());
        ui.allocate_ui_at_rect(bottom_rect, |ui| {
            ui.set_min_size(bottom_rect.size());
            egui::Frame::none()
                .fill(ui.visuals().panel_fill)
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    render_simulation_controls(ui, state, trajectory);
                });
        });
    });
}

/// Render the simulation controls into the provided UI (extracted from previous `simulation_ui`).
fn render_simulation_controls(ui: &mut egui::Ui, state: &mut AppState, trajectory: &TrajectoryData) {
    let mut time_changed = false;

    ui.horizontal(|ui| {
        // Time step buttons
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Step by:");
                ui.horizontal(|ui| {
                    for &step in &[-1000.0, -100.0, -10.0, -1.0, -0.1, -0.01] {
                        if ui.small_button(&format!("{:+.2}s", step)).clicked() {
                            state.simulation.current_time = 
                                (state.simulation.current_time + step).clamp(0.0, trajectory.total_duration);
                            time_changed = true;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    for &step in &[0.01, 0.1, 1.0, 10.0, 100.0, 1000.0] {
                        if ui.small_button(&format!("+{:.2}s", step)).clicked() {
                            state.simulation.current_time = 
                                (state.simulation.current_time + step).clamp(0.0, trajectory.total_duration);
                            time_changed = true;
                        }
                    }
                });
            });
        });

        ui.separator();

        // Speed & transport group
        ui.group(|ui| {
            ui.set_min_width(240.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Transport controls
                    if ui.small_button("⏮").on_hover_text("Jump to start").clicked() {
                        state.simulation.current_time = 0.0;
                        time_changed = true;
                    }
                    if ui.small_button("⏪").on_hover_text("Step backward").clicked() {
                        crate::simulation::step_to_previous_segment(&mut state.simulation, &trajectory);
                        time_changed = true;
                    }
                    let play_icon = if state.simulation.playing { "⏸" } else { "▶" };
                    if ui.small_button(play_icon).clicked() {
                        state.simulation.playing = !state.simulation.playing;
                    }
                    if ui.small_button("⏩").on_hover_text("Step forward").clicked() {
                        crate::simulation::step_to_next_segment(&mut state.simulation, &trajectory);
                        time_changed = true;
                    }
                    if ui.small_button("⏭").on_hover_text("Jump to end").clicked() {
                        state.simulation.current_time = trajectory.total_duration;
                        time_changed = true;
                    }
                    ui.separator();
                    ui.toggle_value(&mut state.simulation.loop_playback, "🔁").on_hover_text("Loop playback");
                });

                ui.separator();

                // Speed display
                let effective_speed = state.simulation.speed * state.simulation.feed_rate_override;
                let speed_color = if effective_speed < 0.0 {
                    egui::Color32::from_rgb(255, 100, 100)
                } else if effective_speed > 1.0 {
                    egui::Color32::from_rgb(100, 255, 100)
                } else {
                    egui::Color32::WHITE
                };

                ui.horizontal(|ui| {
                    ui.label("Speed: ");
                    ui.colored_label(speed_color, format!("{:>+8.2}× ({:>+6.0}%)", state.simulation.speed, effective_speed * 100.0));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.simulation.speed, -1000.0..=1000.0).text("Speed").suffix("×"));
                });

                ui.horizontal(|ui| {
                    for &spd in &[-100.0, -10.0, -1.0, 0.0, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0, 1000.0] {
                        let label = if spd == 0.0 { "⏸".to_string() } else { format!("{}×", spd) };
                        if ui.selectable_label((state.simulation.speed - spd).abs() < 0.01, label).clicked() {
                            state.simulation.speed = spd;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Set speed:");
                    let mut speed_str = format!("{:.2}", state.simulation.speed);
                    if ui.text_edit_singleline(&mut speed_str).changed() {
                        if let Ok(new_speed) = speed_str.parse::<f32>() {
                            state.simulation.speed = new_speed;
                        }
                    }
                    ui.label("×");
                });
            });
        });

        ui.separator();

        // Feed override
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Feed Override:");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut state.simulation.feed_rate_override, 0.0..=2.0).suffix("×").fixed_decimals(2));
                    for &fro in &[0.5, 1.0, 1.5, 2.0] {
                        if ui.small_button(&format!("{}%", (fro * 100.0) as i32)).clicked() {
                            state.simulation.feed_rate_override = fro;
                        }
                    }
                });
            });
        });
    });

    ui.separator();

    // Timeline & Pos
    ui.horizontal(|ui| {
        ui.label("Time:");

        let mut time = state.simulation.current_time;
        let response = ui.add(egui::Slider::new(&mut time, 0.0..=trajectory.total_duration.max(0.001)).suffix("s").fixed_decimals(3).show_value(true));
        if response.changed() {
            state.simulation.current_time = time;
            time_changed = true;
        }

        ui.label(format!("/ {:.3}s", trajectory.total_duration));

        ui.separator();

        let pos = state.simulation.current_position;
        ui.monospace(format!("X:{:>8.3}  Y:{:>8.3}  Z:{:>8.3}  Line:{:>5}", pos.x, pos.y, pos.z, state.simulation.current_line + 1));
    });

    if time_changed {
        crate::simulation::sync_simulation_state(&mut state.simulation, trajectory);
    }

    // Progress
    let total = trajectory.total_duration;
    if total > 0.0 {
        let progress = state.simulation.current_time / total;
        ui.add(egui::ProgressBar::new(progress).show_percentage().animate(state.simulation.playing));
    }
}

