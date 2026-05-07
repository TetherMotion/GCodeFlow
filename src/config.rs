//! Configuration management for GCodeFlow
//!
//! Handles loading, saving, and managing application settings persisted in gcodeflow.yaml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;
use glam::{Vec3, Vec4};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// General application settings
    pub general: GeneralConfig,
    
    /// Machine/kinematics configuration
    pub machine: MachineSettings,
    
    /// Visualization settings
    pub visualization: VisualizationConfig,
    
    /// Camera settings
    pub camera: CameraConfig,
    
    /// Editor settings
    pub editor: EditorConfig,
    
    /// Simulation settings
    pub simulation: SimulationConfig,
    
    /// Trace display settings
    pub trace: TraceConfig,
    
    /// Tool display settings
    pub tool: ToolConfig,
    
    /// Trajectory computation settings (3D visualization)
    pub trajectory: TrajectoryConfig,
    
    /// Machine output settings (for generating machine commands)
    pub machine_output: MachineOutputConfig,
    
    /// Autosave enabled (legacy, use general.autosave_enabled)
    #[serde(default)]
    pub autosave: bool,
    
    /// Autosave interval (legacy, use general.autosave_interval)
    #[serde(default = "default_autosave_interval")]
    pub autosave_interval: f32,
    
    /// Recently opened files
    pub recent_files: Vec<String>,
}

fn default_autosave_interval() -> f32 {
    30.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            machine: MachineSettings::default(),
            visualization: VisualizationConfig::default(),
            camera: CameraConfig::default(),
            editor: EditorConfig::default(),
            simulation: SimulationConfig::default(),
            trace: TraceConfig::default(),
            tool: ToolConfig::default(),
            trajectory: TrajectoryConfig::default(),
            machine_output: MachineOutputConfig::default(),
            autosave: true,
            autosave_interval: 30.0,
            recent_files: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add a file to recent files list
    pub fn add_recent_file(&mut self, path: &str) {
        self.recent_files.retain(|f| f != path);
        self.recent_files.insert(0, path.to_string());
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Enable autosave
    pub autosave_enabled: bool,
    
    /// Autosave interval in seconds
    pub autosave_interval: u32,
    
    /// Last opened file
    pub last_file: Option<String>,
    
    /// Show welcome dialog
    pub show_welcome: bool,
    
    /// Window size
    pub window_width: u32,
    pub window_height: u32,
    
    /// Theme (light/dark)
    pub theme: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
            autosave_interval: 30,
            last_file: None,
            show_welcome: true,
            window_width: 1920,
            window_height: 1080,
            theme: "dark".to_string(),
        }
    }
}

/// Machine and kinematics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MachineSettings {
    /// Machine name/type
    pub name: String,
    
    /// Kinematics type
    pub kinematics_type: KinematicsType,
    
    /// Work volume
    pub work_volume: WorkVolume,
    
    /// Motion limits
    pub motion_limits: MotionLimits,
    
    /// Axis configuration
    pub axes: AxesConfig,
    
    /// Home position
    pub home_position: [f64; 9],
}

impl Default for MachineSettings {
    fn default() -> Self {
        Self {
            name: "Generic 3D Printer".to_string(),
            kinematics_type: KinematicsType::Cartesian,
            work_volume: WorkVolume::default(),
            motion_limits: MotionLimits::default(),
            axes: AxesConfig::default(),
            home_position: [0.0; 9],
        }
    }
}

/// Kinematics type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KinematicsType {
    #[default]
    Cartesian,
    CoreXY,
    CoreXZ,
    Delta,
    Scara,
    FiveAxisBC,
    FiveAxisAC,
    FiveAxisAB,
    Custom,
}

/// Work volume definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkVolume {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl Default for WorkVolume {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            max_x: 220.0,
            min_y: 0.0,
            max_y: 220.0,
            min_z: 0.0,
            max_z: 250.0,
        }
    }
}

/// Motion limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionLimits {
    pub max_velocity_xy: f64,
    pub max_velocity_z: f64,
    pub max_velocity_e: f64,
    pub max_acceleration_xy: f64,
    pub max_acceleration_z: f64,
    pub max_jerk: f64,
    pub default_feed_rate: f64,
    pub rapid_feed_rate: f64,
}

impl Default for MotionLimits {
    fn default() -> Self {
        Self {
            max_velocity_xy: 300.0,  // mm/s
            max_velocity_z: 10.0,    // mm/s
            max_velocity_e: 50.0,    // mm/s
            max_acceleration_xy: 3000.0,  // mm/s²
            max_acceleration_z: 100.0,    // mm/s²
            max_jerk: 10000.0,
            default_feed_rate: 1500.0,  // mm/min
            rapid_feed_rate: 6000.0,    // mm/min
        }
    }
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AxesConfig {
    pub x_steps_per_mm: f64,
    pub y_steps_per_mm: f64,
    pub z_steps_per_mm: f64,
    pub e_steps_per_mm: f64,
    pub x_inverted: bool,
    pub y_inverted: bool,
    pub z_inverted: bool,
    pub e_inverted: bool,
}

impl Default for AxesConfig {
    fn default() -> Self {
        Self {
            x_steps_per_mm: 80.0,
            y_steps_per_mm: 80.0,
            z_steps_per_mm: 400.0,
            e_steps_per_mm: 93.0,
            x_inverted: false,
            y_inverted: false,
            z_inverted: false,
            e_inverted: false,
        }
    }
}

/// Z-level filter configuration for trajectory display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ZFilterConfig {
    /// Whether Z filtering is enabled
    pub enabled: bool,
    
    /// Distance below current tool Z to show (mm). f32::INFINITY means no lower limit.
    pub below: f32,
    
    /// Distance above current tool Z to show (mm). f32::INFINITY means no upper limit.
    pub above: f32,
}

impl Default for ZFilterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            below: f32::INFINITY,
            above: f32::INFINITY,
        }
    }
}

impl ZFilterConfig {
    /// Check if a Z value is within the filter range relative to the reference Z
    pub fn is_visible(&self, z: f32, reference_z: f32) -> bool {
        if !self.enabled {
            return true;
        }
        let lower_bound = if self.below.is_infinite() {
            f32::NEG_INFINITY
        } else {
            reference_z - self.below
        };
        let upper_bound = if self.above.is_infinite() {
            f32::INFINITY
        } else {
            reference_z + self.above
        };
        z >= lower_bound && z <= upper_bound
    }
    
    /// Returns true if filter shows all Z levels (disabled or infinite range)
    pub fn shows_all(&self) -> bool {
        !self.enabled || (self.below.is_infinite() && self.above.is_infinite())
    }
}

/// Visualization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualizationConfig {
    /// Show coordinate axes
    pub show_axes: bool,
    
    /// Show grid
    pub show_grid: bool,
    
    /// Grid size
    pub grid_size: f32,
    
    /// Grid divisions
    pub grid_divisions: u32,
    
    /// Show work volume bounds
    pub show_bounds: bool,
    
    /// Background color
    pub background_color: [f32; 4],
    
    /// Ambient light intensity
    pub ambient_light: f32,
    
    /// Enable shadows
    pub shadows_enabled: bool,
    
    /// Anti-aliasing samples
    pub msaa_samples: u32,
    
    /// Z-level filter for trajectory display
    pub z_filter: ZFilterConfig,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            show_axes: true,
            show_grid: true,
            grid_size: 220.0,
            grid_divisions: 22,
            show_bounds: true,
            background_color: [0.1, 0.1, 0.12, 1.0],
            ambient_light: 0.3,
            shadows_enabled: true,
            msaa_samples: 4,
            z_filter: ZFilterConfig::default(),
        }
    }
}

/// Camera settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Projection mode
    pub projection: ProjectionMode,
    
    /// Camera position
    pub position: [f32; 3],
    
    /// Look-at target
    pub target: [f32; 3],
    
    /// Field of view (perspective mode)
    pub fov: f32,
    
    /// Orthographic scale
    pub ortho_scale: f32,
    
    /// Near clip plane
    pub near: f32,
    
    /// Far clip plane
    pub far: f32,
    
    /// Orbit speed
    pub orbit_speed: f32,
    
    /// Pan speed
    pub pan_speed: f32,
    
    /// Zoom speed
    pub zoom_speed: f32,
    
    /// Tracking mode
    pub tracking_mode: TrackingMode,
    
    /// Tracking axis lock
    pub track_axis_lock: AxisLock,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            projection: ProjectionMode::Perspective,
            position: [200.0, 200.0, 200.0],
            target: [110.0, 110.0, 50.0],
            fov: 45.0,
            ortho_scale: 200.0,
            near: 0.1,
            far: 10000.0,
            orbit_speed: 0.5,
            pan_speed: 1.0,
            zoom_speed: 0.1,
            tracking_mode: TrackingMode::None,
            track_axis_lock: AxisLock::None,
        }
    }
}

/// Camera projection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectionMode {
    #[default]
    Perspective,
    Orthographic,
    /// 2D projection onto XY plane (top view)
    View2D_XY,
    /// 2D projection onto XZ plane (front view)
    View2D_XZ,
    /// 2D projection onto YZ plane (side view)
    View2D_YZ,
}

/// Camera tracking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TrackingMode {
    #[default]
    None,
    ToolEndpoint,
    DesiredPath,
    ActualPath,
    SimulatedPath,
}

/// Axis lock for camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AxisLock {
    #[default]
    None,
    X,
    Y,
    Z,
    ToolAxis,
    PathTangent,
}

/// Editor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    /// Font size
    pub font_size: f32,
    
    /// Font family
    pub font_family: String,
    
    /// Show line numbers
    pub show_line_numbers: bool,
    
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    
    /// Tab size
    pub tab_size: u32,
    
    /// Insert spaces instead of tabs
    pub use_spaces: bool,
    
    /// Auto-indent
    pub auto_indent: bool,
    
    /// Word wrap
    pub word_wrap: bool,
    
    /// Highlight current line
    pub highlight_current_line: bool,
    
    /// Syntax colors
    pub colors: SyntaxColors,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_family: "monospace".to_string(),
            show_line_numbers: true,
            syntax_highlighting: true,
            tab_size: 4,
            use_spaces: true,
            auto_indent: true,
            word_wrap: false,
            highlight_current_line: true,
            colors: SyntaxColors::default(),
        }
    }
}

/// Syntax highlighting colors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SyntaxColors {
    pub gcode: [f32; 4],
    pub mcode: [f32; 4],
    pub axis: [f32; 4],
    pub parameter: [f32; 4],
    pub number: [f32; 4],
    pub comment: [f32; 4],
    pub ocode: [f32; 4],
    pub operator: [f32; 4],
    pub variable: [f32; 4],
    pub error: [f32; 4],
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            gcode: [0.4, 0.8, 1.0, 1.0],     // Cyan
            mcode: [1.0, 0.6, 0.2, 1.0],     // Orange
            axis: [0.4, 1.0, 0.4, 1.0],      // Green
            parameter: [1.0, 1.0, 0.4, 1.0], // Yellow
            number: [0.8, 0.8, 0.8, 1.0],    // Light gray
            comment: [0.5, 0.5, 0.5, 1.0],   // Gray
            ocode: [0.8, 0.4, 1.0, 1.0],     // Purple
            operator: [1.0, 1.0, 1.0, 1.0],  // White
            variable: [0.4, 0.6, 1.0, 1.0],  // Blue
            error: [1.0, 0.3, 0.3, 1.0],     // Red
        }
    }
}

/// Simulation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationConfig {
    /// Default playback speed (1.0 = realtime)
    pub playback_speed: f64,
    
    /// Show simulated trajectory
    pub show_simulated: bool,
    
    /// Simulated trajectory color
    pub simulated_color: [f32; 4],
    
    /// Time step buttons
    pub time_steps: Vec<f64>,
    
    /// Loop playback
    pub loop_playback: bool,
    
    /// Pause at M-codes
    pub pause_at_mcodes: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
            show_simulated: true,
            simulated_color: [0.0, 1.0, 1.0, 1.0],  // Cyan
            time_steps: vec![0.01, 0.1, 1.0, 10.0, 100.0, 1000.0],
            loop_playback: false,
            pause_at_mcodes: true,
        }
    }
}

/// Trace color mode for advanced visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TraceColorMode {
    #[default]
    ByMoveType,
    BySpeed,
    ByZHeight,
    ByTime,
    ByAcceleration,
    ByAccuracy,
}

/// Trace display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TraceConfig {
    /// Show desired trajectory
    pub show_desired: bool,
    
    /// Desired trajectory color (rapids)
    pub rapid_color: [f32; 4],
    
    /// Alias for rapid_color for compatibility
    #[serde(alias = "desired_rapid_color")]
    pub desired_rapid_color: [f32; 4],
    
    /// Desired trajectory color (feeds)
    pub feed_color: [f32; 4],
    
    /// Alias for feed_color for compatibility
    #[serde(alias = "desired_feed_color")]
    pub desired_feed_color: [f32; 4],
    
    /// Desired trajectory color (arcs)
    pub arc_color: [f32; 4],
    
    /// Line width
    pub line_width: f32,
    
    /// Show points on trajectory
    pub show_points: bool,
    
    /// Point size
    pub point_size: f32,
    
    /// Show actual trajectory (hidden by default)
    pub show_actual: bool,
    
    /// Actual trajectory color
    pub actual_color: [f32; 4],
    
    /// Simulated trajectory color
    pub simulated_color: [f32; 4],
    
    /// Show simulated trajectory
    pub show_simulated: bool,
    
    /// Show rapid moves
    pub show_rapids: bool,
    
    /// Show retracts
    pub show_retracts: bool,
    
    /// Use 3D tubes instead of lines
    pub use_tubes: bool,
    
    /// Tube radius
    pub tube_radius: f32,
    
    /// Color by speed (legacy, use color_mode)
    pub color_by_speed: bool,
    
    /// Color by time (legacy, use color_mode)
    pub color_by_time: bool,
    
    /// Low speed color
    pub low_speed_color: [f32; 4],
    
    /// High speed color
    pub high_speed_color: [f32; 4],
    
    /// Color mode for trace visualization
    pub color_mode: TraceColorMode,
    
    /// Low Z height color (for ByZHeight mode)
    pub low_z_color: [f32; 4],
    
    /// High Z height color (for ByZHeight mode)
    pub high_z_color: [f32; 4],
    
    /// Start time color (for ByTime mode)
    pub start_time_color: [f32; 4],
    
    /// End time color (for ByTime mode)
    pub end_time_color: [f32; 4],
    
    /// Low acceleration color
    pub low_accel_color: [f32; 4],
    
    /// High acceleration color
    pub high_accel_color: [f32; 4],
    
    /// High accuracy color (green)
    pub high_accuracy_color: [f32; 4],
    
    /// Low accuracy color (red)
    pub low_accuracy_color: [f32; 4],
    
    /// Trace render settings (controls 3D view point density)
    pub render: TraceRenderConfig,
}

/// Trace render settings - controls how the trajectory is rendered in 3D view
/// This is separate from trajectory computation settings to allow independent control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TraceRenderConfig {
    /// Use adaptive step size based on view/pixel density (default: on)
    pub adaptive_step_size: bool,
    
    /// Fraction of pixel to use as max deviation for adaptive mode (default: 1/3)
    /// Lower values = more points = smoother curves
    pub pixel_fraction: f32,
    
    /// Manual max deviation in mm when adaptive is off
    pub max_deviation: f32,
    
    /// Minimum points per segment (ensures curves don't become too coarse)
    pub min_points_per_segment: u32,
    
    /// Maximum points per segment (prevents excessive memory use)
    pub max_points_per_segment: u32,
    
    /// Cache the computed render deviation (updated each frame)
    #[serde(skip)]
    pub computed_deviation: f32,
}

impl Default for TraceRenderConfig {
    fn default() -> Self {
        Self {
            adaptive_step_size: true,
            pixel_fraction: 0.33, // 1/3 of a pixel
            max_deviation: 0.1,   // 0.1mm fallback
            min_points_per_segment: 2,
            max_points_per_segment: 10000,
            computed_deviation: 0.1,
        }
    }
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            show_desired: true,
            rapid_color: [1.0, 0.2, 0.2, 0.5],  // Red, semi-transparent
            desired_rapid_color: [1.0, 0.2, 0.2, 0.5],  // Red, semi-transparent
            feed_color: [0.2, 0.8, 0.2, 1.0],   // Green
            desired_feed_color: [0.2, 0.8, 0.2, 1.0],   // Green
            arc_color: [0.2, 0.2, 1.0, 1.0],    // Blue
            line_width: 2.0,
            show_points: false,
            point_size: 3.0,
            show_actual: false,
            actual_color: [1.0, 0.5, 0.0, 1.0], // Orange
            simulated_color: [0.0, 1.0, 1.0, 1.0], // Cyan
            show_simulated: true,
            show_rapids: true,
            show_retracts: true,
            use_tubes: false,
            tube_radius: 0.5,
            color_by_speed: false,
            color_by_time: false,
            low_speed_color: [0.0, 0.0, 1.0, 1.0],  // Blue
            high_speed_color: [1.0, 0.0, 0.0, 1.0], // Red
            color_mode: TraceColorMode::ByMoveType,
            low_z_color: [0.0, 0.0, 0.8, 1.0],      // Blue (low Z)
            high_z_color: [1.0, 1.0, 0.0, 1.0],     // Yellow (high Z)
            start_time_color: [0.0, 0.5, 1.0, 1.0], // Light blue (start)
            end_time_color: [1.0, 0.5, 0.0, 1.0],   // Orange (end)
            low_accel_color: [0.3, 0.8, 0.3, 1.0],  // Green (gentle)
            high_accel_color: [1.0, 0.3, 0.3, 1.0], // Red (aggressive)
            high_accuracy_color: [0.0, 1.0, 0.0, 1.0], // Green (accurate)
            low_accuracy_color: [1.0, 0.0, 0.0, 1.0],  // Red (deviation)
            render: TraceRenderConfig::default(),
        }
    }
}

/// Tool display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolConfig {
    /// Show tool
    pub show_tool: bool,
    
    /// Tool model type
    pub tool_type: ToolType,
    
    /// Custom tool model path (STL/STEP)
    pub custom_model_path: Option<String>,
    
    /// Tool color
    pub tool_color: [f32; 4],
    
    /// Tool scale
    pub scale: f32,
    
    /// Tool offset (from tool tip to model origin)
    pub offset: [f32; 3],
    
    /// Tool rotation offset (euler angles in degrees)
    pub rotation: [f32; 3],
    
    /// Track tool to trajectory
    pub track_trajectory: TrackingMode,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            show_tool: true,
            tool_type: ToolType::EndMill,
            custom_model_path: None,
            tool_color: [0.7, 0.7, 0.8, 1.0],  // Silver
            scale: 1.0,
            offset: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            track_trajectory: TrackingMode::DesiredPath,
        }
    }
}

/// Tool type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolType {
    #[default]
    EndMill,
    BallNose,
    VBit,
    Drill,
    Nozzle3DPrinter,
    Custom,
}

/// Trajectory computation settings (for 3D visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrajectoryConfig {
    /// Time resolution for trajectory sampling (seconds)
    pub time_resolution: f32,
    
    /// Use max deviation mode instead of time resolution
    pub use_max_deviation: bool,
    
    /// Maximum spatial deviation (mm)
    pub max_deviation: f32,
}

impl Default for TrajectoryConfig {
    fn default() -> Self {
        Self {
            time_resolution: 0.01, // 10ms
            use_max_deviation: false,
            max_deviation: 0.1, // 0.1mm
        }
    }
}

/// Machine output settings (for generating machine commands)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MachineOutputConfig {
    /// Output time resolution (seconds) - how often to output points
    pub time_resolution: f32,
    
    /// Use max deviation mode instead of time resolution
    pub use_max_deviation: bool,
    
    /// Maximum spatial deviation (mm)
    pub max_deviation: f32,
    
    /// Convert arcs (G2/G3) to linear segments
    pub linearize_arcs: bool,
    
    /// Arc linearization: segments per full circle (360°)
    pub arc_segments_per_circle: u32,
    
    /// Arc linearization: maximum chord deviation (mm)
    pub arc_max_deviation: f32,
    
    /// Use arc deviation mode instead of segments
    pub arc_use_deviation: bool,
}

impl Default for MachineOutputConfig {
    fn default() -> Self {
        Self {
            time_resolution: 0.001, // 1ms for precise machine control
            use_max_deviation: true,
            max_deviation: 0.01, // 0.01mm for machine precision
            linearize_arcs: false, // Keep arcs as G2/G3 by default
            arc_segments_per_circle: 64,
            arc_max_deviation: 0.01, // 0.01mm chord deviation
            arc_use_deviation: true,
        }
    }
}

/// Camera preset angles
pub struct CameraPreset;

impl CameraPreset {
    pub fn top() -> ([f32; 3], [f32; 3]) {
        ([110.0, 110.0, 500.0], [110.0, 110.0, 0.0])
    }

    pub fn front() -> ([f32; 3], [f32; 3]) {
        ([110.0, -300.0, 125.0], [110.0, 110.0, 125.0])
    }

    pub fn side() -> ([f32; 3], [f32; 3]) {
        ([-300.0, 110.0, 125.0], [110.0, 110.0, 125.0])
    }

    pub fn iso() -> ([f32; 3], [f32; 3]) {
        ([300.0, -200.0, 250.0], [110.0, 110.0, 50.0])
    }

    pub fn from_name(name: &str) -> Option<([f32; 3], [f32; 3])> {
        match name.to_lowercase().as_str() {
            "top" => Some(Self::top()),
            "front" => Some(Self::front()),
            "side" | "right" => Some(Self::side()),
            "iso" | "isometric" => Some(Self::iso()),
            _ => None,
        }
    }
}
