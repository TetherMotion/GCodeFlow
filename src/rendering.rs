//! 3D rendering of GCode trajectories
//!
//! Handles rendering of motion paths, tool visualization,
//! and trajectory display with various modes.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::app::AppState;
use crate::camera::CameraController;
use crate::config::TraceColorMode;
use crate::trajectory::{TrajectoryData, TrajectoryPoint};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderTrajectoryData::default())
           .add_systems(Startup, setup_rendering)
           .add_systems(Update, (
               compute_render_deviation.before(update_render_trajectory),
               update_render_trajectory.before(update_trajectory_mesh),
               update_trajectory_mesh,
               update_tool_mesh.before(update_tool_position),
               update_tool_position,
               update_grid,
               render_selected_point,
               update_axes.after(update_tool_position).after(update_grid).after(render_selected_point),
               update_axis_labels.after(render_selected_point).after(update_tool_position),
           ));
    }
}

/// Render-specific trajectory data with adaptive interpolation
#[derive(Resource, Default)]
pub struct RenderTrajectoryData {
    /// Points interpolated specifically for rendering
    pub points: Vec<TrajectoryPoint>,
    /// The deviation used for this interpolation
    pub used_deviation: f32,
    /// Whether we need to re-interpolate
    pub needs_update: bool,
}

/// Marker for the desired trajectory mesh
#[derive(Component)]
pub struct DesiredTrajectory;

/// Marker for the actual trajectory mesh
#[derive(Component)]
pub struct ActualTrajectory;

/// Marker for the simulated trajectory mesh
#[derive(Component)]
pub struct SimulatedTrajectory;

/// Marker for the tool visualization
#[derive(Component)]
pub struct ToolMarker;

/// Marker for the grid plane
#[derive(Component)]
pub struct GridPlane;

/// Marker for axes visualization
#[derive(Component)]
pub struct AxisMarker {
    pub axis: Axis,
}

/// Label for axis (text at axis end)
#[derive(Component)]
pub struct AxisLabel {
    pub axis: Axis,
}

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Marker for selected point visualization
#[derive(Component)]
pub struct SelectedPointMarker;

/// Materials for rendering
#[derive(Resource, Clone)]
pub struct RenderMaterials {
    pub desired_rapid: Handle<StandardMaterial>,
    pub desired_feed: Handle<StandardMaterial>,
    pub actual: Handle<StandardMaterial>,
    pub simulated: Handle<StandardMaterial>,
    pub tool: Handle<StandardMaterial>,
    pub grid: Handle<StandardMaterial>,
    pub axis_x: Handle<StandardMaterial>,
    pub axis_y: Handle<StandardMaterial>,
    pub axis_z: Handle<StandardMaterial>,
    pub selected_point: Handle<StandardMaterial>,
}

/// Tool geometry helper (tip offset above tool origin so the tip aligns with tool position)
#[derive(Resource, Clone, Copy)]
pub struct ToolGeometry {
    pub tip_offset: f32,
}
/// Track the desired trajectory mesh handle
#[derive(Resource)]
pub struct DesiredTrajectoryMesh(pub Handle<Mesh>);

fn setup_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    state: Res<AppState>,
) {
    let trace_config = &state.config.trace;
    let tool_config = &state.config.tool;
    
    // Create materials
    let render_materials = RenderMaterials {
        desired_rapid: materials.add(StandardMaterial {
            base_color: array_to_color(&trace_config.desired_rapid_color),
            unlit: true,
            ..default()
        }),
        desired_feed: materials.add(StandardMaterial {
            base_color: array_to_color(&trace_config.desired_feed_color),
            unlit: true,
            ..default()
        }),
        actual: materials.add(StandardMaterial {
            base_color: array_to_color(&trace_config.actual_color),
            unlit: true,
            ..default()
        }),
        simulated: materials.add(StandardMaterial {
            base_color: array_to_color(&trace_config.simulated_color),
            unlit: true,
            ..default()
        }),
        tool: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.2),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
        grid: materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        axis_x: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            unlit: true,
            ..default()
        }),
        axis_y: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 0.2),
            unlit: true,
            ..default()
        }),
        axis_z: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 1.0),
            unlit: true,
            ..default()
        }),
        selected_point: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            emissive: LinearRgba::new(1.0, 1.0, 0.0, 1.0),
            ..default()
        }),
    };
    
    commands.insert_resource(render_materials.clone());
    
    // Create grid
    let grid_mesh = create_grid_mesh(500.0, 50);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(grid_mesh),
            material: render_materials.grid.clone(),
            ..default()
        },
        GridPlane,
    ));
    
    // Create axes (GCode -> Bevy): X -> X, Y -> Y, Z -> Z
    // In the default camera perspective the Z axis will point upwards (world Z is up)
    let axis_length = 100.0;
    let axis_radius = 0.5;

    // X axis (red) - along Bevy X
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(axis_radius, axis_length)),
            material: render_materials.axis_x.clone(),
            transform: Transform::from_translation(Vec3::new(axis_length / 2.0, 0.0, 0.0))
                .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        AxisMarker { axis: Axis::X },
    ));

    // Y axis (green) - along Bevy Y
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(axis_radius, axis_length)),
            material: render_materials.axis_y.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, axis_length / 2.0, 0.0)),
            ..default()
        },
        AxisMarker { axis: Axis::Y },
    ));

    // Z axis (blue) - along Bevy Z
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(axis_radius, axis_length)),
            material: render_materials.axis_z.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, axis_length / 2.0))
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        AxisMarker { axis: Axis::Z },
    ));

    // Load font for axis labels (place `fonts/FiraSans-Bold.ttf` in your `assets/` folder)
    let axis_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // Put labels noticeably beyond the axis end so they sit at the far end
    let label_offset = 12.0;

    // Axis labels (X,Y,Z) — larger, color-matched, and placed at the far end of each axis
    // Font size is large and we apply a modest world scale so labels appear big in world space
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "X",
                TextStyle { font: axis_font.clone(), font_size: 180.0, color: Color::srgb(1.0, 0.2, 0.2) }
            ),
            transform: Transform::from_translation(Vec3::new(axis_length + label_offset, 0.0, 0.0))
                .with_scale(Vec3::splat(0.12)),
            ..default()
        },
        AxisLabel { axis: Axis::X },
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Y",
                TextStyle { font: axis_font.clone(), font_size: 180.0, color: Color::srgb(0.2, 1.0, 0.2) }
            ),
            transform: Transform::from_translation(Vec3::new(0.0, axis_length + label_offset, 0.0))
                .with_scale(Vec3::splat(0.12)),
            ..default()
        },
        AxisLabel { axis: Axis::Y },
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Z",
                TextStyle { font: axis_font.clone(), font_size: 180.0, color: Color::srgb(0.2, 0.2, 1.0) }
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, axis_length + label_offset))
                .with_scale(Vec3::splat(0.12)),
            ..default()
        },
        AxisLabel { axis: Axis::Z },
    ));
    
    // Create tool placeholder (pointed end mill)
    let tool_mesh = create_tool_mesh(tool_config);
    // Compute tip offset matching create_tool_mesh logic so the tip sits at z=0 when positioned
    let scale = tool_config.scale;
    let diameter = 6.0 * scale;
    let radius = diameter / 2.0;
    let tip_angle = 60.0_f32.to_radians();
    let tip_height = radius / (tip_angle / 2.0).tan();

    commands.insert_resource(ToolGeometry { tip_offset: tip_height });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(tool_mesh),
            material: render_materials.tool.clone(),
            ..default()
        },
        ToolMarker,
    ));
    
    // Create selected point marker (sphere)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(2.0)),
            material: render_materials.selected_point.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -1000.0, 0.0)), // Hidden initially
            visibility: Visibility::Hidden,
            ..default()
        },
        SelectedPointMarker,
    ));
    
    // Create empty trajectory mesh and track its handle
    let trajectory_mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    let trajectory_mesh_handle = meshes.add(trajectory_mesh);
    commands.insert_resource(DesiredTrajectoryMesh(trajectory_mesh_handle.clone()));
    
    commands.spawn((
        PbrBundle {
            mesh: trajectory_mesh_handle,
            material: render_materials.desired_feed.clone(),
            ..default()
        },
        DesiredTrajectory,
    ));
    
    // Actual trajectory (hidden by default)
    let actual_mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(actual_mesh),
            material: render_materials.actual.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.1, 0.0)), // Slight offset
            visibility: Visibility::Hidden,
            ..default()
        },
        ActualTrajectory,
    ));
    
    // Simulated trajectory (hidden by default)
    let simulated_mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(simulated_mesh),
            material: render_materials.simulated.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.05, 0.0)),
            visibility: Visibility::Hidden,
            ..default()
        },
        SimulatedTrajectory,
    ));
    
    // Add lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
    
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 500000.0,
            range: 500.0,
            ..default()
        },
        transform: Transform::from_xyz(-50.0, 100.0, -50.0),
        ..default()
    });
}

fn create_grid_mesh(size: f32, divisions: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    
    let half = size / 2.0;
    let step = size / divisions as f32;
    
    let mut idx = 0u32;
    
    for i in 0..=divisions {
        let offset = -half + i as f32 * step;
        
        // Line along X (varying Y) — grid now lies on the XY plane (Z = 0)
        positions.push([offset, -half, 0.0]);
        positions.push([offset, half, 0.0]);
        indices.push(idx);
        indices.push(idx + 1);
        idx += 2;
        
        // Line along Y (varying X)
        positions.push([-half, offset, 0.0]);
        positions.push([half, offset, 0.0]);
        indices.push(idx);
        indices.push(idx + 1);
        idx += 2;
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn create_tool_mesh(config: &crate::config::ToolConfig) -> Mesh {
    use crate::config::ToolType;
    
    match config.tool_type {
        ToolType::EndMill => create_endmill_mesh(config.scale),
        ToolType::BallNose => create_ballnose_mesh(config.scale),
        ToolType::VBit => create_vbit_mesh(config.scale),
        ToolType::Drill => create_drill_mesh(config.scale),
        ToolType::Nozzle3DPrinter => create_nozzle_mesh(config.scale),
        ToolType::Custom => create_endmill_mesh(config.scale), // Fallback to endmill
    }
}

/// Create a pointed end mill mesh (cone tip + cylinder body)
fn create_endmill_mesh(scale: f32) -> Mesh {
    let diameter = 6.0 * scale;
    let length = 40.0 * scale;
    let tip_angle = 60.0_f32.to_radians();
    
    let radius = diameter / 2.0;
    let tip_height = radius / (tip_angle / 2.0).tan();
    
    let segments = 16u32;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    // Tip vertex
    positions.push([0.0, 0.0, -tip_height]);
    normals.push([0.0, 0.0, -1.0]);
    
    // Bottom circle of cone
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, 0.0]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Top circle of cylinder
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, length]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Top cap center
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, length]);
    normals.push([0.0, 0.0, 1.0]);
    
    // Cone triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push(1 + i);
        indices.push(1 + next);
    }
    
    // Cylinder side triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        let bottom1 = 1 + i;
        let bottom2 = 1 + next;
        let top1 = 1 + segments + i;
        let top2 = 1 + segments + next;
        
        indices.push(bottom1);
        indices.push(top1);
        indices.push(bottom2);
        
        indices.push(bottom2);
        indices.push(top1);
        indices.push(top2);
    }
    
    // Top cap triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(top_center_idx);
        indices.push(1 + segments + next);
        indices.push(1 + segments + i);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Create a ball nose end mill mesh (hemisphere tip + cylinder body)
fn create_ballnose_mesh(scale: f32) -> Mesh {
    let diameter = 6.0 * scale;
    let length = 40.0 * scale;
    let radius = diameter / 2.0;
    
    let segments = 16u32;
    let hemisphere_rings = 8u32;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    // Hemisphere tip (pointing down, -Z)
    // Bottom pole
    positions.push([0.0, 0.0, -radius]);
    normals.push([0.0, 0.0, -1.0]);
    
    // Hemisphere rings
    for ring in 1..hemisphere_rings {
        let phi = (ring as f32 / hemisphere_rings as f32) * std::f32::consts::FRAC_PI_2;
        let ring_radius = radius * phi.sin();
        let z = -radius * phi.cos();
        
        for seg in 0..segments {
            let theta = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let x = ring_radius * theta.cos();
            let y = ring_radius * theta.sin();
            positions.push([x, y, z]);
            
            // Normal points outward from center
            let nx = phi.sin() * theta.cos();
            let ny = phi.sin() * theta.sin();
            let nz = -phi.cos();
            normals.push([nx, ny, nz]);
        }
    }
    
    // Equator ring (z = 0)
    let equator_start = positions.len() as u32;
    for seg in 0..segments {
        let theta = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        positions.push([x, y, 0.0]);
        normals.push([theta.cos(), theta.sin(), 0.0]);
    }
    
    // Cylinder body - top ring
    let top_start = positions.len() as u32;
    for seg in 0..segments {
        let theta = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        positions.push([x, y, length]);
        normals.push([theta.cos(), theta.sin(), 0.0]);
    }
    
    // Top cap center
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, length]);
    normals.push([0.0, 0.0, 1.0]);
    
    // Hemisphere triangles - bottom pole fan
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        indices.push(0);
        indices.push(1 + seg);
        indices.push(1 + next);
    }
    
    // Hemisphere rings
    for ring in 0..(hemisphere_rings - 2) {
        let ring_start = 1 + ring * segments;
        let next_ring_start = 1 + (ring + 1) * segments;
        
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            
            indices.push(ring_start + seg);
            indices.push(next_ring_start + seg);
            indices.push(ring_start + next);
            
            indices.push(ring_start + next);
            indices.push(next_ring_start + seg);
            indices.push(next_ring_start + next);
        }
    }
    
    // Connect last hemisphere ring to equator
    let last_ring_start = 1 + (hemisphere_rings - 2) * segments;
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        
        indices.push(last_ring_start + seg);
        indices.push(equator_start + seg);
        indices.push(last_ring_start + next);
        
        indices.push(last_ring_start + next);
        indices.push(equator_start + seg);
        indices.push(equator_start + next);
    }
    
    // Cylinder body triangles
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        
        indices.push(equator_start + seg);
        indices.push(top_start + seg);
        indices.push(equator_start + next);
        
        indices.push(equator_start + next);
        indices.push(top_start + seg);
        indices.push(top_start + next);
    }
    
    // Top cap triangles
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        indices.push(top_center_idx);
        indices.push(top_start + next);
        indices.push(top_start + seg);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Create a V-bit mesh (sharp cone tip + cylinder body)
fn create_vbit_mesh(scale: f32) -> Mesh {
    let diameter = 6.0 * scale;
    let length = 30.0 * scale;
    let tip_angle = 90.0_f32.to_radians(); // 90 degree V-bit
    
    let radius = diameter / 2.0;
    let tip_height = radius / (tip_angle / 2.0).tan();
    
    let segments = 16u32;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    // Sharp tip vertex
    positions.push([0.0, 0.0, -tip_height]);
    normals.push([0.0, 0.0, -1.0]);
    
    // Cone base circle
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, 0.0]);
        
        // Cone normal
        let cone_angle = (tip_angle / 2.0).cos();
        let nx = angle.cos() * cone_angle;
        let ny = angle.sin() * cone_angle;
        let nz = -(tip_angle / 2.0).sin();
        normals.push([nx, ny, nz]);
    }
    
    // Cylinder top
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, length]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Top cap center
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, length]);
    normals.push([0.0, 0.0, 1.0]);
    
    // Cone triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push(1 + i);
        indices.push(1 + next);
    }
    
    // Cylinder side
    for i in 0..segments {
        let next = (i + 1) % segments;
        let bottom = 1 + i;
        let bottom_next = 1 + next;
        let top = 1 + segments + i;
        let top_next = 1 + segments + next;
        
        indices.push(bottom);
        indices.push(top);
        indices.push(bottom_next);
        
        indices.push(bottom_next);
        indices.push(top);
        indices.push(top_next);
    }
    
    // Top cap
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(top_center_idx);
        indices.push(1 + segments + next);
        indices.push(1 + segments + i);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Create a drill bit mesh (pointed tip with spiral flutes indication + cylinder)
fn create_drill_mesh(scale: f32) -> Mesh {
    let diameter = 6.0 * scale;
    let length = 50.0 * scale;
    let tip_angle = 118.0_f32.to_radians(); // Standard 118 degree drill point
    
    let radius = diameter / 2.0;
    let tip_height = radius / (tip_angle / 2.0).tan();
    
    let segments = 16u32;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    // Drill tip (pointed)
    positions.push([0.0, 0.0, -tip_height]);
    normals.push([0.0, 0.0, -1.0]);
    
    // Flute indication - slightly narrower section near tip
    let flute_start = 0.0;
    let flute_end = length * 0.7;
    let flute_rings = 4u32;
    
    // Point cone base
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, flute_start]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Flute section with slight taper
    for ring in 1..=flute_rings {
        let z = flute_start + (flute_end - flute_start) * (ring as f32 / flute_rings as f32);
        let taper = 1.0 - 0.05 * (1.0 - ring as f32 / flute_rings as f32); // Slight taper
        
        for i in 0..segments {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let x = radius * taper * angle.cos();
            let y = radius * taper * angle.sin();
            positions.push([x, y, z]);
            normals.push([angle.cos(), angle.sin(), 0.0]);
        }
    }
    
    // Shank (full diameter)
    let shank_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        positions.push([x, y, length]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Top cap
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, length]);
    normals.push([0.0, 0.0, 1.0]);
    
    // Point cone triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push(1 + i);
        indices.push(1 + next);
    }
    
    // Flute section
    for ring in 0..flute_rings {
        let ring_start = 1 + ring * segments;
        let next_ring = 1 + (ring + 1) * segments;
        
        for i in 0..segments {
            let next = (i + 1) % segments;
            
            indices.push(ring_start + i);
            indices.push(next_ring + i);
            indices.push(ring_start + next);
            
            indices.push(ring_start + next);
            indices.push(next_ring + i);
            indices.push(next_ring + next);
        }
    }
    
    // Connect flute to shank
    let flute_end_ring = 1 + flute_rings * segments;
    for i in 0..segments {
        let next = (i + 1) % segments;
        
        indices.push(flute_end_ring + i);
        indices.push(shank_start + i);
        indices.push(flute_end_ring + next);
        
        indices.push(flute_end_ring + next);
        indices.push(shank_start + i);
        indices.push(shank_start + next);
    }
    
    // Top cap
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(top_center_idx);
        indices.push(shank_start + next);
        indices.push(shank_start + i);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Create a 3D printer nozzle mesh (tapered body with small outlet)
fn create_nozzle_mesh(scale: f32) -> Mesh {
    let body_diameter = 8.0 * scale;
    let nozzle_diameter = 0.4 * scale;  // 0.4mm nozzle
    let body_length = 12.0 * scale;
    let taper_length = 4.0 * scale;
    let tip_length = 1.0 * scale;
    
    let body_radius = body_diameter / 2.0;
    let nozzle_radius = nozzle_diameter / 2.0;
    
    let segments = 16u32;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    // Nozzle tip (small opening at bottom)
    let tip_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, -tip_length]);
    normals.push([0.0, 0.0, -1.0]);
    
    // Nozzle opening ring
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        positions.push([nozzle_radius * angle.cos(), nozzle_radius * angle.sin(), -tip_length]);
        normals.push([0.0, 0.0, -1.0]);
    }
    
    // Nozzle tip outer ring
    let tip_outer_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        positions.push([nozzle_radius * angle.cos(), nozzle_radius * angle.sin(), 0.0]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Taper section (nozzle to body)
    let taper_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        positions.push([body_radius * angle.cos(), body_radius * angle.sin(), taper_length]);
        
        // Taper normal
        let taper_angle = ((body_radius - nozzle_radius) / taper_length).atan();
        let nx = angle.cos() * taper_angle.cos();
        let ny = angle.sin() * taper_angle.cos();
        let nz = -taper_angle.sin();
        normals.push([nx, ny, nz]);
    }
    
    // Body top
    let body_top_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        positions.push([body_radius * angle.cos(), body_radius * angle.sin(), body_length]);
        normals.push([angle.cos(), angle.sin(), 0.0]);
    }
    
    // Top cap center
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, body_length]);
    normals.push([0.0, 0.0, 1.0]);
    
    // Nozzle opening (bottom cap with hole - just outer ring)
    // We'll skip the center hole for simplicity, just make a flat bottom
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(tip_center_idx);
        indices.push(1 + next);
        indices.push(1 + i);
    }
    
    // Nozzle tip cylinder
    for i in 0..segments {
        let next = (i + 1) % segments;
        let bottom = 1 + i;
        let bottom_next = 1 + next;
        let top = tip_outer_start + i;
        let top_next = tip_outer_start + next;
        
        indices.push(bottom);
        indices.push(top);
        indices.push(bottom_next);
        
        indices.push(bottom_next);
        indices.push(top);
        indices.push(top_next);
    }
    
    // Taper section
    for i in 0..segments {
        let next = (i + 1) % segments;
        let bottom = tip_outer_start + i;
        let bottom_next = tip_outer_start + next;
        let top = taper_start + i;
        let top_next = taper_start + next;
        
        indices.push(bottom);
        indices.push(top);
        indices.push(bottom_next);
        
        indices.push(bottom_next);
        indices.push(top);
        indices.push(top_next);
    }
    
    // Body cylinder
    for i in 0..segments {
        let next = (i + 1) % segments;
        let bottom = taper_start + i;
        let bottom_next = taper_start + next;
        let top = body_top_start + i;
        let top_next = body_top_start + next;
        
        indices.push(bottom);
        indices.push(top);
        indices.push(bottom_next);
        
        indices.push(bottom_next);
        indices.push(top);
        indices.push(top_next);
    }
    
    // Top cap
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(top_center_idx);
        indices.push(body_top_start + next);
        indices.push(body_top_start + i);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Compute the render deviation based on camera perspective
/// This calculates what mm corresponds to a fraction of a pixel at the trajectory center
fn compute_render_deviation(
    mut state: ResMut<AppState>,
    trajectory: Res<TrajectoryData>,
    camera_query: Query<(&CameraController, &GlobalTransform, &Projection, &Camera)>,
    windows: Query<&Window>,
) {
    let render_config = &mut state.config.trace.render;
    
    if !render_config.adaptive_step_size {
        render_config.computed_deviation = render_config.max_deviation;
        return;
    }
    
    // Get window dimensions
    let window = match windows.iter().next() {
        Some(w) => w,
        None => {
            render_config.computed_deviation = render_config.max_deviation;
            return;
        }
    };
    
    let _window_width = window.width();
    let window_height = window.height();
    
    // Get trajectory center and size for reference
    let trajectory_center = trajectory.center();
    let _trajectory_size = trajectory.size().length().max(1.0);
    
    // Calculate the world-space size of a pixel at the trajectory center
    for (controller, camera_transform, projection, _camera) in camera_query.iter() {
        // Distance from camera to trajectory center
        let camera_pos = camera_transform.translation();
        let distance_to_center = (camera_pos - trajectory_center).length().max(1.0);
        
        // Calculate mm per pixel based on projection type
        let mm_per_pixel = match projection {
            Projection::Perspective(persp) => {
                // For perspective: tan(fov/2) * 2 * distance / height
                let fov_rad = persp.fov;
                let half_height_at_distance = distance_to_center * (fov_rad / 2.0).tan();
                let world_height = half_height_at_distance * 2.0;
                world_height / window_height
            }
            Projection::Orthographic(ortho) => {
                // For orthographic: scale directly gives world units per normalized device unit
                // ortho.scale is world-units-per-100-pixels approximately
                let world_height = controller.ortho_scale * 2.0;
                world_height / window_height
            }
        };
        
        // Apply pixel fraction setting
        let deviation = mm_per_pixel * render_config.pixel_fraction;
        
        // Clamp to reasonable bounds
        render_config.computed_deviation = deviation.clamp(0.0001, 10.0);
        
        // Only use first camera
        break;
    }
}

/// Update render-specific trajectory with adaptive interpolation
fn update_render_trajectory(
    trajectory: Res<TrajectoryData>,
    state: Res<AppState>,
    mut render_trajectory: ResMut<RenderTrajectoryData>,
) {
    let render_config = &state.config.trace.render;
    let target_deviation = render_config.computed_deviation;
    
    // Check if we need to update
    let deviation_changed = (render_trajectory.used_deviation - target_deviation).abs() > 0.0001;
    let trajectory_changed = trajectory.is_changed();
    
    if !deviation_changed && !trajectory_changed && !render_trajectory.points.is_empty() {
        return;
    }
    
    // Re-interpolate trajectory for rendering with fixed deviation
    render_trajectory.points.clear();
    render_trajectory.used_deviation = target_deviation;
    
    if trajectory.desired.len() < 2 {
        render_trajectory.points = trajectory.desired.clone();
        return;
    }
    
    let min_points = render_config.min_points_per_segment as usize;
    let max_points = render_config.max_points_per_segment as usize;
    
    // Walk through trajectory segments and add points based on deviation
    render_trajectory.points.push(trajectory.desired[0].clone());
    
    for i in 1..trajectory.desired.len() {
        let p0 = &trajectory.desired[i - 1];
        let p1 = &trajectory.desired[i];
        
        let segment_length = (p1.position - p0.position).length();
        
        // For very short segments, just add the endpoint
        if segment_length < target_deviation * 0.5 {
            render_trajectory.points.push(p1.clone());
            continue;
        }
        
        // Calculate number of intermediate points needed
        // For a straight line, we need enough points that max chord deviation is within target
        // For a chord of length L, the max deviation for n segments is approximately L^2 / (8 * R)
        // where R is the radius of curvature. For straight lines, this is 0.
        // We use a heuristic: points_needed = ceil(segment_length / (2 * sqrt(2 * target_deviation * segment_length)))
        // Simplified: we just ensure points are close enough that linear interpolation error is bounded
        
        let points_for_segment = if segment_length > 0.0 {
            // Simple heuristic: at least one point per 'target_deviation' of distance
            // but also considering curvature by looking at velocity changes
            let base_points = (segment_length / target_deviation).ceil() as usize;
            base_points.clamp(min_points, max_points)
        } else {
            min_points
        };
        
        // Add intermediate points
        for j in 1..=points_for_segment {
            let t = j as f32 / points_for_segment as f32;
            
            // Linear interpolation
            let pos = p0.position.lerp(p1.position, t);
            let time = p0.time + (p1.time - p0.time) * t;
            let feed_rate = p0.feed_rate + (p1.feed_rate - p0.feed_rate) * t;
            
            render_trajectory.points.push(TrajectoryPoint {
                position: pos,
                time,
                feed_rate,
                line_number: p0.line_number,
                is_rapid: p0.is_rapid,
                is_motion: p0.is_motion,
            });
        }
    }
    
    render_trajectory.needs_update = false;
}

fn update_trajectory_mesh(
    trajectory: Res<TrajectoryData>,
    render_trajectory: Res<RenderTrajectoryData>,
    state: Res<AppState>,
    mut meshes: ResMut<Assets<Mesh>>,
    trajectory_mesh: Res<DesiredTrajectoryMesh>,
) {
    // Update when render trajectory changes or when config changes
    if !render_trajectory.is_changed() && !state.is_changed() {
        return;
    }
    
    let trace_config = &state.config.trace;
    let z_filter = &state.config.visualization.z_filter;
    let reference_z = state.simulation.current_position.z;
    
    // Use render trajectory points (adaptively interpolated) for the mesh
    let points = &render_trajectory.points;
    
    // Build mesh from trajectory points, filtering by Z level
    let mut positions = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut current_strip_start: Option<u32> = None;
    
    // Calculate ranges for color mapping from render points
    let (min_z, max_z) = points.iter().fold(
        (f32::MAX, f32::MIN),
        |(min, max), p| (min.min(p.position.z), max.max(p.position.z))
    );
    let z_range = (max_z - min_z).max(0.001);
    
    let (min_speed, max_speed) = points.iter().fold(
        (f32::MAX, f32::MIN),
        |(min, max), p| (min.min(p.feed_rate), max.max(p.feed_rate))
    );
    let speed_range = (max_speed - min_speed).max(0.001);
    
    let total_time = trajectory.total_duration.max(0.001);
    
    // Pre-compute accelerations if needed
    let accelerations: Vec<f32> = if matches!(trace_config.color_mode, TraceColorMode::ByAcceleration) {
        compute_accelerations(points)
    } else {
        vec![]
    };
    let (min_accel, max_accel) = if !accelerations.is_empty() {
        accelerations.iter().fold(
            (0.0_f32, 0.001_f32),
            |(min, max), &a| (min.min(a.abs()), max.max(a.abs()))
        )
    } else {
        (0.0, 0.001)
    };
    let accel_range = (max_accel - min_accel).max(0.001);
    
    for (i, point) in points.iter().enumerate() {
        // Apply Z filter
        let is_visible = z_filter.is_visible(point.position.z, reference_z);
        
        if is_visible {
            let vertex_idx = positions.len() as u32;
            
            // Use direct mapping: GCode (x,y,z) -> Bevy (x,y,z)
            positions.push([point.position.x, point.position.y, point.position.z]);
            
            let color = match trace_config.color_mode {
                TraceColorMode::ByMoveType => {
                    if point.is_rapid {
                        trace_config.desired_rapid_color
                    } else {
                        trace_config.desired_feed_color
                    }
                }
                TraceColorMode::BySpeed => {
                    let t = (point.feed_rate - min_speed) / speed_range;
                    lerp_color(&trace_config.low_speed_color, &trace_config.high_speed_color, t)
                }
                TraceColorMode::ByZHeight => {
                    let t = (point.position.z - min_z) / z_range;
                    lerp_color(&trace_config.low_z_color, &trace_config.high_z_color, t)
                }
                TraceColorMode::ByTime => {
                    let t = point.time / total_time;
                    lerp_color(&trace_config.start_time_color, &trace_config.end_time_color, t)
                }
                TraceColorMode::ByAcceleration => {
                    let accel = if i < accelerations.len() { accelerations[i].abs() } else { 0.0 };
                    let t = (accel - min_accel) / accel_range;
                    lerp_color(&trace_config.low_accel_color, &trace_config.high_accel_color, t)
                }
                TraceColorMode::ByAccuracy => {
                    // For accuracy, we'd need actual vs desired comparison
                    // For now, use feed_color as a placeholder
                    trace_config.desired_feed_color
                }
            };
            colors.push(color);
            
            // Build line indices - connect to previous visible point if in same strip
            if let Some(_start) = current_strip_start {
                if vertex_idx > 0 {
                    // Add line segment from previous vertex to current
                    indices.push(vertex_idx - 1);
                    indices.push(vertex_idx);
                }
            } else {
                // Start a new strip
                current_strip_start = Some(vertex_idx);
            }
        } else {
            // Point filtered out - end current strip
            current_strip_start = None;
        }
    }
    
    if let Some(mesh) = meshes.get_mut(&trajectory_mesh.0) {
        // Use LineList topology for filtered segments (handles gaps properly)
        *mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineList,
            bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        
        if !indices.is_empty() {
            mesh.insert_indices(Indices::U32(indices));
        }
    }
}

/// Linearly interpolate between two colors
fn lerp_color(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Compute accelerations from trajectory points
fn compute_accelerations(points: &[TrajectoryPoint]) -> Vec<f32> {
    if points.len() < 3 {
        return vec![0.0; points.len()];
    }
    
    let mut accelerations = vec![0.0; points.len()];
    
    for i in 1..points.len() - 1 {
        let dt1 = (points[i].time - points[i - 1].time).max(0.001);
        let dt2 = (points[i + 1].time - points[i].time).max(0.001);
        
        let v1 = (points[i].position - points[i - 1].position).length() / dt1;
        let v2 = (points[i + 1].position - points[i].position).length() / dt2;
        
        let dt_avg = (dt1 + dt2) / 2.0;
        accelerations[i] = (v2 - v1) / dt_avg;
    }
    
    accelerations
}

fn update_tool_position(
    state: Res<AppState>,
    trajectory: Res<TrajectoryData>,
    tool_geo: Res<ToolGeometry>,
    mut query: Query<(&mut Transform, &mut Visibility), With<ToolMarker>>,
) {
    let sim_state = &state.simulation;
    
    for (mut transform, mut visibility) in query.iter_mut() {
        if !state.config.tool.show_tool {
            *visibility = Visibility::Hidden;
            continue;
        }
        
        *visibility = Visibility::Visible;
        
        // Get position from simulation or current time
        if let Some(point) = trajectory.point_at_time(sim_state.current_time) {
            // Direct mapping: GCode -> Bevy. Offset by tool tip height so the tip sits at the point
            transform.translation = Vec3::new(
                point.position.x,
                point.position.y,
                point.position.z + tool_geo.tip_offset,
            );
        }
    }
}

/// Rebuild the tool mesh and update tip offset when tool scale or config changes
fn update_tool_mesh(
    state: Res<AppState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Handle<Mesh>, With<ToolMarker>>,
    mut tool_geo: ResMut<ToolGeometry>,
    mut last_scale: Local<f32>,
) {
    let scale = state.config.tool.scale;

    // Run only when scale changes
    if (*last_scale - scale).abs() < f32::EPSILON {
        return;
    }

    *last_scale = scale;

    // Recompute tip offset (matches create_tool_mesh logic)
    let tip_angle = 60.0_f32.to_radians();
    let diameter = 6.0 * scale;
    let radius = diameter / 2.0;
    let tip_height = radius / (tip_angle / 2.0).tan();
    tool_geo.tip_offset = tip_height;

    // Replace mesh on all ToolMarker entities
    for mut mesh_handle in query.iter_mut() {
        *mesh_handle = meshes.add(create_tool_mesh(&state.config.tool));
    }
}

fn update_grid(
    state: Res<AppState>,
    mut query: Query<&mut Visibility, With<GridPlane>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = if state.config.visualization.show_grid {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_axes(
    state: Res<AppState>,
    mut query: Query<(&mut Visibility, Option<&AxisMarker>, Option<&AxisLabel>)>,
) {
    let vis = if state.config.visualization.show_axes {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for (mut visibility, axis_marker, axis_label) in query.iter_mut() {
        if axis_marker.is_some() || axis_label.is_some() {
            *visibility = vis;
        }
    }
}

/// Orient axis labels to face the active camera
fn update_axis_labels(
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut query: Query<(&AxisLabel, &mut Transform), With<Text>>,
) {
    // Use the first camera we find
    let cam_tf = match camera_query.iter().next() {
        Some(tf) => tf.translation(),
        None => return,
    };

    for (_label, mut transform) in query.iter_mut() {
        transform.look_at(cam_tf, Vec3::Z);
    }
}

fn render_selected_point(
    trajectory: Res<TrajectoryData>,
    mut query: Query<(&mut Transform, &mut Visibility), With<SelectedPointMarker>>,
) {
    for (mut transform, mut visibility) in query.iter_mut() {
        if let Some(idx) = trajectory.selected_point {
            if let Some(point) = trajectory.desired.get(idx) {
                *visibility = Visibility::Visible;
                transform.translation = Vec3::new(
                    point.position.x,
                    point.position.y,
                    point.position.z,
                );
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn array_to_color(arr: &[f32; 4]) -> Color {
    Color::srgba(arr[0], arr[1], arr[2], arr[3])
}

/// Create a tube mesh along a trajectory for thicker lines
pub fn create_trajectory_tube_mesh(
    points: &[TrajectoryPoint],
    radius: f32,
    segments: u32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    if points.len() < 2 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    }
    
    for (i, point) in points.iter().enumerate() {
        // Calculate tangent direction
        let tangent = if i == 0 {
            (points[1].position - points[0].position).normalize()
        } else if i == points.len() - 1 {
            (points[i].position - points[i - 1].position).normalize()
        } else {
            ((points[i + 1].position - points[i].position).normalize() +
             (points[i].position - points[i - 1].position).normalize()).normalize()
        };
        
        // Calculate perpendicular vectors
        let up = if tangent.y.abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };
        let right = tangent.cross(up).normalize();
        let actual_up = right.cross(tangent).normalize();
        
        // Create ring of vertices
        let base_idx = positions.len() as u32;
        
        for j in 0..segments {
            let angle = j as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let offset = right * angle.cos() * radius + actual_up * angle.sin() * radius;
            let pos = point.position + offset;
            let normal = offset.normalize();
            
            // Use direct mapping (GCode -> Bevy): x,y,z
            positions.push([pos.x, pos.y, pos.z]);
            normals.push([normal.x, normal.y, normal.z]);
        }
        
        // Connect to previous ring
        if i > 0 {
            let prev_base = base_idx - segments;
            for j in 0..segments {
                let next = (j + 1) % segments;
                
                indices.push(prev_base + j);
                indices.push(base_idx + j);
                indices.push(prev_base + next);
                
                indices.push(prev_base + next);
                indices.push(base_idx + j);
                indices.push(base_idx + next);
            }
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
