//! Kinematics configuration and computation
//!
//! Provides forward and inverse kinematics for various
//! machine configurations (Cartesian, CoreXY, Delta, SCARA, etc.)

use bevy::prelude::*;

use crate::config::{KinematicsType, MachineSettings};

pub struct KinematicsPlugin;

impl Plugin for KinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KinematicsState::default());
    }
}

/// Runtime kinematics state
#[derive(Resource, Default)]
pub struct KinematicsState {
    /// Current kinematics model
    pub model: KinematicsModel,
    
    /// Last computed joint positions
    pub joint_positions: Vec<f32>,
    
    /// Kinematics error (if any)
    pub error: Option<String>,
}

/// Kinematics model enum
#[derive(Clone, Default)]
pub enum KinematicsModel {
    #[default]
    Cartesian,
    CoreXY(CoreXYKinematics),
    Delta(DeltaKinematics),
    Scara(ScaraKinematics),
    Polar(PolarKinematics),
    FiveAxis(FiveAxisKinematics),
    Custom(CustomKinematics),
}

/// CoreXY kinematics parameters
#[derive(Clone, Default)]
pub struct CoreXYKinematics {
    /// Steps per mm for motor A
    pub steps_per_mm_a: f32,
    /// Steps per mm for motor B
    pub steps_per_mm_b: f32,
}

impl CoreXYKinematics {
    pub fn new() -> Self {
        Self {
            steps_per_mm_a: 80.0,
            steps_per_mm_b: 80.0,
        }
    }
    
    /// Convert XY position to motor positions
    pub fn inverse(&self, x: f32, y: f32) -> (f32, f32) {
        // CoreXY: A = X + Y, B = X - Y
        let a = x + y;
        let b = x - y;
        (a, b)
    }
    
    /// Convert motor positions to XY position
    pub fn forward(&self, a: f32, b: f32) -> (f32, f32) {
        // CoreXY: X = (A + B) / 2, Y = (A - B) / 2
        let x = (a + b) / 2.0;
        let y = (a - b) / 2.0;
        (x, y)
    }
}

/// Delta kinematics parameters
#[derive(Clone)]
pub struct DeltaKinematics {
    /// Length of the diagonal rod
    pub rod_length: f32,
    /// Radius from center to tower
    pub tower_radius: f32,
    /// Tower positions (120° apart)
    pub tower_angles: [f32; 3],
    /// Endstop positions
    pub endstop_offsets: [f32; 3],
    /// Print radius
    pub print_radius: f32,
}

impl Default for DeltaKinematics {
    fn default() -> Self {
        Self::new(215.0, 100.0)
    }
}

impl DeltaKinematics {
    pub fn new(rod_length: f32, tower_radius: f32) -> Self {
        Self {
            rod_length,
            tower_radius,
            tower_angles: [210.0, 330.0, 90.0], // Standard 120° apart
            endstop_offsets: [0.0, 0.0, 0.0],
            print_radius: tower_radius * 0.6,
        }
    }
    
    /// Get tower position
    fn tower_position(&self, tower: usize) -> Vec2 {
        let angle = self.tower_angles[tower].to_radians();
        Vec2::new(
            self.tower_radius * angle.cos(),
            self.tower_radius * angle.sin(),
        )
    }
    
    /// Convert XYZ position to carriage heights (inverse kinematics)
    pub fn inverse(&self, x: f32, y: f32, z: f32) -> Option<[f32; 3]> {
        let mut heights = [0.0_f32; 3];
        let pos = Vec2::new(x, y);
        
        for i in 0..3 {
            let tower = self.tower_position(i);
            let distance = (pos - tower).length();
            
            // Check if position is reachable
            if distance > self.rod_length {
                return None;
            }
            
            // Calculate carriage height using Pythagorean theorem
            let height_delta = (self.rod_length * self.rod_length - distance * distance).sqrt();
            heights[i] = z + height_delta + self.endstop_offsets[i];
        }
        
        Some(heights)
    }
    
    /// Convert carriage heights to XYZ position (forward kinematics)
    pub fn forward(&self, heights: [f32; 3]) -> Option<Vec3> {
        // This uses trilateration to find the effector position
        // Given three sphere centers (at tower positions) and radii (rod lengths),
        // find the intersection point
        
        let p1 = Vec3::new(
            self.tower_position(0).x,
            self.tower_position(0).y,
            heights[0] - self.endstop_offsets[0],
        );
        let p2 = Vec3::new(
            self.tower_position(1).x,
            self.tower_position(1).y,
            heights[1] - self.endstop_offsets[1],
        );
        let p3 = Vec3::new(
            self.tower_position(2).x,
            self.tower_position(2).y,
            heights[2] - self.endstop_offsets[2],
        );
        
        let r = self.rod_length;
        
        // Trilateration algorithm
        let ex = (p2 - p1).normalize();
        let i = ex.dot(p3 - p1);
        let ey = ((p3 - p1) - ex * i).normalize();
        let d = (p2 - p1).length();
        let j = ey.dot(p3 - p1);
        
        let x = (r * r - r * r + d * d) / (2.0 * d);
        let y = (r * r - r * r + i * i + j * j - 2.0 * i * x) / (2.0 * j);
        
        let z_sq = r * r - x * x - y * y;
        if z_sq < 0.0 {
            return None;
        }
        
        let z = -z_sq.sqrt(); // Negative because effector is below carriages
        
        Some(p1 + ex * x + ey * y + Vec3::Z * z)
    }
}

/// SCARA kinematics parameters
#[derive(Clone)]
pub struct ScaraKinematics {
    /// Length of first arm segment
    pub arm1_length: f32,
    /// Length of second arm segment
    pub arm2_length: f32,
    /// Elbow preference (true = elbow up)
    pub elbow_up: bool,
}

impl Default for ScaraKinematics {
    fn default() -> Self {
        Self::new(150.0, 150.0)
    }
}

impl ScaraKinematics {
    pub fn new(arm1: f32, arm2: f32) -> Self {
        Self {
            arm1_length: arm1,
            arm2_length: arm2,
            elbow_up: true,
        }
    }
    
    /// Convert XY position to joint angles (inverse kinematics)
    pub fn inverse(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let l1 = self.arm1_length;
        let l2 = self.arm2_length;
        
        let distance = (x * x + y * y).sqrt();
        
        // Check reachability
        if distance > l1 + l2 || distance < (l1 - l2).abs() {
            return None;
        }
        
        // Calculate elbow angle using law of cosines
        let cos_theta2 = (distance * distance - l1 * l1 - l2 * l2) / (2.0 * l1 * l2);
        
        // Clamp to handle numerical errors
        let cos_theta2 = cos_theta2.clamp(-1.0, 1.0);
        
        let theta2 = if self.elbow_up {
            -cos_theta2.acos()
        } else {
            cos_theta2.acos()
        };
        
        // Calculate shoulder angle
        let k1 = l1 + l2 * theta2.cos();
        let k2 = l2 * theta2.sin();
        let theta1 = y.atan2(x) - k2.atan2(k1);
        
        Some((theta1.to_degrees(), theta2.to_degrees()))
    }
    
    /// Convert joint angles to XY position (forward kinematics)
    pub fn forward(&self, theta1_deg: f32, theta2_deg: f32) -> Vec2 {
        let theta1 = theta1_deg.to_radians();
        let theta2 = theta2_deg.to_radians();
        
        let x = self.arm1_length * theta1.cos() + self.arm2_length * (theta1 + theta2).cos();
        let y = self.arm1_length * theta1.sin() + self.arm2_length * (theta1 + theta2).sin();
        
        Vec2::new(x, y)
    }
}

/// Polar kinematics parameters
#[derive(Clone, Default)]
pub struct PolarKinematics {
    /// Maximum radius
    pub max_radius: f32,
}

impl PolarKinematics {
    pub fn new(max_radius: f32) -> Self {
        Self { max_radius }
    }
    
    /// Convert XY to polar coordinates
    pub fn inverse(&self, x: f32, y: f32) -> (f32, f32) {
        let r = (x * x + y * y).sqrt();
        let theta = y.atan2(x).to_degrees();
        (r, theta)
    }
    
    /// Convert polar to XY coordinates
    pub fn forward(&self, r: f32, theta_deg: f32) -> Vec2 {
        let theta = theta_deg.to_radians();
        Vec2::new(r * theta.cos(), r * theta.sin())
    }
}

/// 5-axis gantry kinematics
#[derive(Clone, Default)]
pub struct FiveAxisKinematics {
    /// A axis rotation offset
    pub a_offset: f32,
    /// B axis rotation offset
    pub b_offset: f32,
    /// Tool length compensation
    pub tool_length: f32,
}

impl FiveAxisKinematics {
    pub fn new() -> Self {
        Self {
            a_offset: 0.0,
            b_offset: 0.0,
            tool_length: 0.0,
        }
    }
    
    /// Apply 5-axis transformation
    pub fn transform(&self, x: f32, y: f32, z: f32, a_deg: f32, b_deg: f32) -> Vec3 {
        let a = (a_deg + self.a_offset).to_radians();
        let b = (b_deg + self.b_offset).to_radians();
        
        // Tool length compensation
        let tool_offset = Vec3::new(
            self.tool_length * b.sin(),
            -self.tool_length * a.sin() * b.cos(),
            -self.tool_length * a.cos() * b.cos(),
        );
        
        Vec3::new(x, y, z) + tool_offset
    }
}

/// Custom kinematics (for user-defined machines)
#[derive(Clone, Default)]
pub struct CustomKinematics {
    /// Number of axes
    pub num_axes: usize,
    /// Transformation matrix (if linear)
    pub transform_matrix: Option<[[f32; 3]; 3]>,
    /// Description
    pub description: String,
}

/// Create kinematics model from settings
pub fn create_kinematics_model(settings: &MachineSettings) -> KinematicsModel {
    match settings.kinematics_type {
        KinematicsType::Cartesian => KinematicsModel::Cartesian,
        
        KinematicsType::CoreXY => {
            KinematicsModel::CoreXY(CoreXYKinematics::new())
        }
        
        KinematicsType::CoreXZ => {
            // CoreXZ is similar to CoreXY but in XZ plane - use CoreXY model
            KinematicsModel::CoreXY(CoreXYKinematics::new())
        }
        
        KinematicsType::Delta => {
            // Use default delta parameters from work volume
            let rod_length = 300.0;  // Typical delta rod length
            let radius = 150.0;      // Typical delta radius
            KinematicsModel::Delta(DeltaKinematics::new(rod_length, radius))
        }
        
        KinematicsType::Scara => {
            // Use default SCARA arm lengths
            let arm1 = 200.0;
            let arm2 = 200.0;
            KinematicsModel::Scara(ScaraKinematics::new(arm1, arm2))
        }
        
        KinematicsType::FiveAxisBC | KinematicsType::FiveAxisAC | KinematicsType::FiveAxisAB => {
            KinematicsModel::FiveAxis(FiveAxisKinematics::new())
        }
        
        KinematicsType::Custom => {
            KinematicsModel::Custom(CustomKinematics::default())
        }
    }
}

/// Convert world position to joint positions
pub fn inverse_kinematics(model: &KinematicsModel, pos: Vec3) -> Option<Vec<f32>> {
    match model {
        KinematicsModel::Cartesian => {
            Some(vec![pos.x, pos.y, pos.z])
        }
        
        KinematicsModel::CoreXY(k) => {
            let (a, b) = k.inverse(pos.x, pos.y);
            Some(vec![a, b, pos.z])
        }
        
        KinematicsModel::Delta(k) => {
            k.inverse(pos.x, pos.y, pos.z)
                .map(|h| h.to_vec())
        }
        
        KinematicsModel::Scara(k) => {
            k.inverse(pos.x, pos.y)
                .map(|(t1, t2)| vec![t1, t2, pos.z])
        }
        
        KinematicsModel::Polar(k) => {
            let (r, theta) = k.inverse(pos.x, pos.y);
            Some(vec![r, theta, pos.z])
        }
        
        KinematicsModel::FiveAxis(_) => {
            // 5-axis needs additional rotation info
            Some(vec![pos.x, pos.y, pos.z, 0.0, 0.0])
        }
        
        KinematicsModel::Custom(_) => {
            // Custom needs user-defined implementation
            Some(vec![pos.x, pos.y, pos.z])
        }
    }
}

/// Convert joint positions to world position
pub fn forward_kinematics(model: &KinematicsModel, joints: &[f32]) -> Option<Vec3> {
    match model {
        KinematicsModel::Cartesian => {
            if joints.len() >= 3 {
                Some(Vec3::new(joints[0], joints[1], joints[2]))
            } else {
                None
            }
        }
        
        KinematicsModel::CoreXY(k) => {
            if joints.len() >= 3 {
                let (x, y) = k.forward(joints[0], joints[1]);
                Some(Vec3::new(x, y, joints[2]))
            } else {
                None
            }
        }
        
        KinematicsModel::Delta(k) => {
            if joints.len() >= 3 {
                k.forward([joints[0], joints[1], joints[2]])
            } else {
                None
            }
        }
        
        KinematicsModel::Scara(k) => {
            if joints.len() >= 3 {
                let pos = k.forward(joints[0], joints[1]);
                Some(Vec3::new(pos.x, pos.y, joints[2]))
            } else {
                None
            }
        }
        
        KinematicsModel::Polar(k) => {
            if joints.len() >= 3 {
                let pos = k.forward(joints[0], joints[1]);
                Some(Vec3::new(pos.x, pos.y, joints[2]))
            } else {
                None
            }
        }
        
        KinematicsModel::FiveAxis(k) => {
            if joints.len() >= 5 {
                Some(k.transform(joints[0], joints[1], joints[2], joints[3], joints[4]))
            } else if joints.len() >= 3 {
                Some(Vec3::new(joints[0], joints[1], joints[2]))
            } else {
                None
            }
        }
        
        KinematicsModel::Custom(_) => {
            if joints.len() >= 3 {
                Some(Vec3::new(joints[0], joints[1], joints[2]))
            } else {
                None
            }
        }
    }
}
