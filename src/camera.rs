//! Camera controls and navigation
//!
//! Provides CAD-like camera navigation with orbit, pan, zoom,
//! and preset view angles.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use std::f32::consts::PI;

use crate::app::AppState;
use crate::config::{ProjectionMode, TrackingMode};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, (
               camera_orbit_system,
               camera_pan_system,
               camera_zoom_system,
               camera_tracking_system,
               update_camera_from_config,
           ));
    }
}

/// Camera controller component
#[derive(Component)]
pub struct CameraController {
    pub focus: Vec3,
    pub radius: f32,
    pub pitch: f32,  // Vertical angle
    pub yaw: f32,    // Horizontal angle
    pub projection_mode: ProjectionMode,
    pub ortho_scale: f32,
    pub tracking_mode: TrackingMode,
    pub track_position: Option<Vec3>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            focus: Vec3::new(110.0, 110.0, 50.0),
            radius: 350.0,
            pitch: 0.5,
            yaw: 0.8,
            projection_mode: ProjectionMode::Perspective,
            ortho_scale: 200.0,
            tracking_mode: TrackingMode::None,
            track_position: None,
        }
    }
}

impl CameraController {
    /// Calculate camera position from spherical coordinates
    pub fn position(&self) -> Vec3 {
        let x = self.radius * self.pitch.cos() * self.yaw.cos();
        let y = self.radius * self.pitch.cos() * self.yaw.sin();
        let z = self.radius * self.pitch.sin();
        self.focus + Vec3::new(x, y, z)
    }

    /// Set camera to a preset view
    pub fn set_preset(&mut self, preset: ViewPreset) {
        match preset {
            ViewPreset::Top => {
                self.pitch = PI / 2.0 - 0.001;
                self.yaw = 0.0;
            }
            ViewPreset::Bottom => {
                self.pitch = -PI / 2.0 + 0.001;
                self.yaw = 0.0;
            }
            ViewPreset::Front => {
                self.pitch = 0.0;
                self.yaw = -PI / 2.0;
            }
            ViewPreset::Back => {
                self.pitch = 0.0;
                self.yaw = PI / 2.0;
            }
            ViewPreset::Right => {
                self.pitch = 0.0;
                self.yaw = 0.0;
            }
            ViewPreset::Left => {
                self.pitch = 0.0;
                self.yaw = PI;
            }
            ViewPreset::Isometric => {
                self.pitch = 0.5;
                self.yaw = 0.8;
            }
        }
    }

    /// Fit view to bounding box
    pub fn fit_to_bounds(&mut self, min: Vec3, max: Vec3) {
        let center = (min + max) / 2.0;
        let size = max - min;
        let max_dim = size.x.max(size.y).max(size.z);
        
        self.focus = center;
        self.radius = max_dim * 1.5;
        self.ortho_scale = max_dim;
    }
}

/// Preset view angles
#[derive(Clone, Copy, Debug)]
pub enum ViewPreset {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left,
    Isometric,
}

fn setup_camera(
    mut commands: Commands,
    state: Res<AppState>,
) {
    let config = &state.config.camera;
    
    // Calculate initial controller state from config position/target
    let pos = Vec3::from_array(config.position);
    let target = Vec3::from_array(config.target);
    let offset = pos - target;
    
    let radius = offset.length();
    let pitch = (offset.z / radius).asin();
    let yaw = offset.y.atan2(offset.x);

    let controller = CameraController {
        focus: target,
        radius,
        pitch,
        yaw,
        projection_mode: config.projection,
        ortho_scale: config.ortho_scale,
        tracking_mode: config.tracking_mode,
        track_position: None,
    };

    // Spawn camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(controller.position())
                .looking_at(controller.focus, Vec3::Z),
            projection: match controller.projection_mode {
                ProjectionMode::Perspective => Projection::Perspective(PerspectiveProjection {
                    fov: config.fov.to_radians(),
                    near: config.near,
                    far: config.far,
                    ..default()
                }),
                ProjectionMode::Orthographic 
                | ProjectionMode::View2D_XY 
                | ProjectionMode::View2D_XZ 
                | ProjectionMode::View2D_YZ => Projection::Orthographic(OrthographicProjection {
                    scale: controller.ortho_scale / 100.0,
                    near: config.near,
                    far: config.far,
                    ..default()
                }),
            },
            ..default()
        },
        controller,
    ));

    // Add lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: state.config.visualization.shadows_enabled,
            ..default()
        },
        transform: Transform::from_xyz(100.0, 100.0, 200.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: state.config.visualization.ambient_light * 1000.0,
    });
}

fn camera_orbit_system(
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut CameraController, &mut Transform)>,
    state: Res<AppState>,
) {
    let orbit_button = MouseButton::Right;
    
    if !mouse_button.pressed(orbit_button) {
        mouse_motion.clear();
        return;
    }

    // Don't orbit if Alt is held (that's for pan)
    if keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta += ev.delta;
    }

    if delta.length_squared() == 0.0 {
        return;
    }

    for (mut controller, mut transform) in query.iter_mut() {
        // Don't allow orbit in 2D modes
        if matches!(controller.projection_mode, 
                   ProjectionMode::View2D_XY | ProjectionMode::View2D_XZ | ProjectionMode::View2D_YZ) {
            continue;
        }
        
        let speed = state.config.camera.orbit_speed * 0.01;
        
        controller.yaw -= delta.x * speed;
        controller.pitch += delta.y * speed;
        
        // Clamp pitch to avoid gimbal lock
        controller.pitch = controller.pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
        
        // Update transform
        let pos = controller.position();
        transform.translation = pos;
        *transform = transform.looking_at(controller.focus, Vec3::Z);
    }
}

fn camera_pan_system(
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut CameraController, &mut Transform)>,
    state: Res<AppState>,
) {
    let pan_button = MouseButton::Middle;
    let alt_pan = mouse_button.pressed(MouseButton::Right) && 
                  (keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight));
    
    if !mouse_button.pressed(pan_button) && !alt_pan {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta += ev.delta;
    }

    if delta.length_squared() == 0.0 {
        return;
    }

    for (mut controller, mut transform) in query.iter_mut() {
        let speed = state.config.camera.pan_speed * controller.radius * 0.001;
        
        // Calculate right and up vectors in world space
        let forward = (controller.focus - controller.position()).normalize();
        let right = forward.cross(Vec3::Z).normalize();
        let up = right.cross(forward).normalize();
        
        // Pan
        controller.focus -= right * delta.x * speed;
        controller.focus += up * delta.y * speed;
        
        // Update transform
        let pos = controller.position();
        transform.translation = pos;
        *transform = transform.looking_at(controller.focus, Vec3::Z);
    }
}

fn camera_zoom_system(
    mut scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut CameraController, &mut Transform, &mut Projection)>,
    state: Res<AppState>,
) {
    let mut scroll_delta = 0.0;
    for ev in scroll.read() {
        scroll_delta += ev.y;
    }

    if scroll_delta == 0.0 {
        return;
    }

    for (mut controller, mut transform, mut projection) in query.iter_mut() {
        let speed = state.config.camera.zoom_speed;
        let factor = 1.0 - scroll_delta * speed;
        
        match controller.projection_mode {
            ProjectionMode::Perspective => {
                controller.radius = (controller.radius * factor).max(1.0).min(10000.0);
            }
            ProjectionMode::Orthographic 
            | ProjectionMode::View2D_XY 
            | ProjectionMode::View2D_XZ 
            | ProjectionMode::View2D_YZ => {
                controller.ortho_scale = (controller.ortho_scale * factor).max(1.0).min(10000.0);
                if let Projection::Orthographic(ref mut ortho) = *projection {
                    ortho.scale = controller.ortho_scale / 100.0;
                }
            }
        }
        
        // Update transform
        let pos = controller.position();
        transform.translation = pos;
        *transform = transform.looking_at(controller.focus, Vec3::Z);
    }
}

fn camera_tracking_system(
    mut query: Query<(&mut CameraController, &mut Transform)>,
) {
    for (mut controller, mut transform) in query.iter_mut() {
        if controller.tracking_mode == TrackingMode::None {
            continue;
        }

        if let Some(track_pos) = controller.track_position {
            controller.focus = track_pos;
            
            let pos = controller.position();
            transform.translation = pos;
            *transform = transform.looking_at(controller.focus, Vec3::Z);
        }
    }
}

fn update_camera_from_config(
    state: Res<AppState>,
    mut query: Query<(&mut CameraController, &mut Transform, &mut Projection)>,
) {
    if !state.is_changed() {
        return;
    }

    for (mut controller, mut transform, mut projection) in query.iter_mut() {
        let old_mode = controller.projection_mode;
        controller.projection_mode = state.config.camera.projection;
        controller.tracking_mode = state.config.camera.tracking_mode;

        // Handle 2D view mode transitions
        let mode_changed = old_mode != controller.projection_mode;
        if mode_changed {
            match controller.projection_mode {
                ProjectionMode::View2D_XY => {
                    // Top view - looking down Z axis
                    controller.pitch = PI / 2.0 - 0.001;
                    controller.yaw = 0.0;
                }
                ProjectionMode::View2D_XZ => {
                    // Front view - looking down Y axis
                    controller.pitch = 0.0;
                    controller.yaw = -PI / 2.0;
                }
                ProjectionMode::View2D_YZ => {
                    // Side view - looking down X axis
                    controller.pitch = 0.0;
                    controller.yaw = PI;
                }
                _ => {}
            }
        }

        // Update transform for 2D modes
        if matches!(controller.projection_mode, 
                   ProjectionMode::View2D_XY | ProjectionMode::View2D_XZ | ProjectionMode::View2D_YZ) {
            let pos = controller.position();
            transform.translation = pos;
            *transform = transform.looking_at(controller.focus, Vec3::Z);
        }

        // Update projection
        match controller.projection_mode {
            ProjectionMode::Perspective => {
                *projection = Projection::Perspective(PerspectiveProjection {
                    fov: state.config.camera.fov.to_radians(),
                    near: state.config.camera.near,
                    far: state.config.camera.far,
                    ..default()
                });
            }
            ProjectionMode::Orthographic 
            | ProjectionMode::View2D_XY 
            | ProjectionMode::View2D_XZ 
            | ProjectionMode::View2D_YZ => {
                *projection = Projection::Orthographic(OrthographicProjection {
                    scale: controller.ortho_scale / 100.0,
                    near: state.config.camera.near,
                    far: state.config.camera.far,
                    ..default()
                });
            }
        }
    }
}
