//! Main application setup and Bevy app configuration

use anyhow::Result;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowPlugin, WindowResolution};
use bevy::render::settings::{WgpuSettings, Backends};
use bevy_egui::EguiPlugin;
use std::path::PathBuf;

use crate::camera::CameraPlugin;
use crate::config::{AppConfig, CameraPreset};
#[cfg(feature = "editor")]
use crate::editor::EditorPlugin;
use crate::kinematics::KinematicsPlugin;
use crate::plot_view::PlotViewPlugin;
use crate::rendering::RenderingPlugin;
use crate::simulation::{SimulationPlugin, SimulationState};
use crate::trajectory::TrajectoryPlugin;
use crate::ui::UiPlugin;
use crate::Args;

/// Main application state
#[derive(Resource)]
pub struct AppState {
    /// Current GCode content
    pub gcode_content: String,
    
    /// Current file path
    pub current_file: Option<PathBuf>,
    
    /// Whether content has been modified
    pub modified: bool,
    
    /// Application configuration
    pub config: AppConfig,
    
    /// Command line arguments
    pub args: Args,
    
    /// Screenshot requested
    pub screenshot_requested: bool,
    
    /// Exit after screenshot
    pub exit_after_screenshot: bool,
    
    /// Frame counter for screenshot timing
    pub frame_count: u32,
    
    /// Simulation state
    pub simulation: SimulationState,
    /// Diagnostics messages (logic issues found during processing)
    pub diagnostics: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            gcode_content: String::new(),
            current_file: None,
            modified: false,
            config: AppConfig::default(),
            args: Args::default(),
            screenshot_requested: false,
            exit_after_screenshot: false,
            frame_count: 0,
            simulation: SimulationState::new(),
            diagnostics: Vec::new(),
        }
    }
}

/// Screenshot capture state
#[derive(Resource, Default)]
pub struct ScreenshotState {
    pub path: Option<PathBuf>,
    pub capture_next_frame: bool,
    pub exit_after: bool,
}

/// Run the application
pub fn run(args: Args) -> Result<()> {
    // Load configuration
    let config = AppConfig::load(&args.config).unwrap_or_default();
    
    // Prepare initial state
    let mut state = AppState {
        gcode_content: String::new(),
        current_file: args.input.clone(),
        modified: false,
        config,
        args: args.clone(),
        screenshot_requested: args.screenshot.is_some(),
        exit_after_screenshot: args.screenshot.is_some(),
        frame_count: 0,
        simulation: SimulationState::new(),
            diagnostics: Vec::new(),
    };

    // Load input file if specified
    if let Some(ref path) = args.input {
        if path.exists() {
            state.gcode_content = std::fs::read_to_string(path)?;
            log::info!("Loaded GCode file: {:?}", path);
        }
    }

    // Set up camera from args
    if let Some((pos, target)) = CameraPreset::from_name(&args.camera_angle) {
        state.config.camera.position = pos;
        state.config.camera.target = target;
    }

    if let Some(ref pos_str) = args.camera_pos {
        if let Ok(pos) = parse_vec3(pos_str) {
            state.config.camera.position = pos;
        }
    }

    if let Some(ref target_str) = args.camera_target {
        if let Ok(target) = parse_vec3(target_str) {
            state.config.camera.target = target;
        }
    }

    // Screenshot state
    let screenshot_state = ScreenshotState {
        path: args.screenshot.clone(),
        capture_next_frame: false,
        exit_after: args.screenshot.is_some(),
    };

    // Build Bevy app
    let mut app = App::new();

    // Configure window
    let window_plugin = if args.headless {
        WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            close_when_requested: false,
        }
    } else {
        WindowPlugin {
            primary_window: Some(Window {
                title: "GCodeFlow - GCode Visualization".to_string(),
                resolution: WindowResolution::new(args.width as f32, args.height as f32),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }
    };

    app.add_plugins(
        DefaultPlugins
            .set(window_plugin)
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(
                    WgpuSettings {
                        backends: Some(Backends::VULKAN | Backends::METAL | Backends::DX12),
                        ..default()
                    }
                ),
                synchronous_pipeline_compilation: false,
            })
    );

    // Add egui plugin
    app.add_plugins(EguiPlugin);

    // Add our plugins
    #[cfg(feature = "editor")]
    app.add_plugins((
        CameraPlugin,
        EditorPlugin,
        KinematicsPlugin,
        PlotViewPlugin,
        RenderingPlugin,
        SimulationPlugin,
        TrajectoryPlugin,
        UiPlugin,
    ));

    #[cfg(not(feature = "editor"))]
    app.add_plugins((
        CameraPlugin,
        KinematicsPlugin,
        PlotViewPlugin,
        RenderingPlugin,
        SimulationPlugin,
        TrajectoryPlugin,
        UiPlugin,
    ));

    // Insert resources
    app.insert_resource(state);
    app.insert_resource(screenshot_state);

    // Add systems
    app.add_systems(Update, handle_screenshot);
    app.add_systems(Update, update_frame_count);

    // Run
    app.run();

    Ok(())
}

fn parse_vec3(s: &str) -> Result<[f32; 3]> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("Expected 3 comma-separated values");
    }
    Ok([
        parts[0].trim().parse()?,
        parts[1].trim().parse()?,
        parts[2].trim().parse()?,
    ])
}

fn update_frame_count(mut state: ResMut<AppState>) {
    state.frame_count += 1;
}

fn handle_screenshot(
    mut screenshot_state: ResMut<ScreenshotState>,
    state: Res<AppState>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    // Wait a few frames for rendering to stabilize
    if state.frame_count > 10 && screenshot_state.path.is_some() && !screenshot_state.capture_next_frame {
        screenshot_state.capture_next_frame = true;
        log::info!("Screenshot scheduled for next frame");
    }

    // After capture, exit if requested
    if screenshot_state.capture_next_frame && state.frame_count > 15 {
        if screenshot_state.exit_after {
            log::info!("Exiting after screenshot");
            exit.send(bevy::app::AppExit::Success);
        }
        screenshot_state.capture_next_frame = false;
    }
}
