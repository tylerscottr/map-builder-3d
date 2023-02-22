/// A mod that creates a controller that acts like a first-person shooter.
pub mod fps_controller;

use bevy::{ecs::prelude::*, math::prelude::*, prelude::*};
use bevy_rapier3d::prelude::*;

/// A struct used to generate simple transforms for cameras.
#[derive(Component, Clone)]
pub struct LookTransform {
    /// The offset from the parent.
    pub offset: Vec3,
    /// The pitch for the camera transform.
    pub pitch: f32,
    /// The XZ radius that causes a roational offset based on the pitch.
    pub pitch_radius: f32,
    /// The yaw for the camera transform.
    pub yaw: f32,
    /// The radius that causes a roational offset based on the yaw.
    pub yaw_radius: f32,
}

impl Default for LookTransform {
    fn default() -> Self {
        Self {
            offset: Vec3::ZERO,
            pitch: 0.0,
            pitch_radius: 0.0,
            yaw: 0.0,
            yaw_radius: 0.0,
        }
    }
}

impl LookTransform {
    /// Creates a look offset from pitch and yaw.
    pub fn from_pitch_yaw(pitch: f32, yaw: f32) -> Self {
        Self {
            pitch,
            yaw,
            ..default()
        }
    }

    /// Creates a look offset from pitch, yaw, and offset.
    pub fn from_pitch_yaw_offset(pitch: f32, yaw: f32, offset: Vec3) -> Self {
        Self {
            offset,
            pitch,
            yaw,
            ..default()
        }
    }

    /// Creates a look offset from a position and a target.
    pub fn from_pos_target(pos: Vec3, target: Vec3) -> Self {
        let ray = target - pos;
        let pitch = std::f32::consts::FRAC_PI_2 - Vec3::Y.angle_between(ray);
        let yaw = (Mat3::from_rotation_x(pitch) * Vec3::Z).angle_between(ray);

        Self {
            offset: pos,
            pitch,
            yaw,
            ..default()
        }
    }

    fn unit_vector_from_pitch_and_yaw(pitch: f32, yaw: f32) -> Vec3 {
        // Apply the yaw first
        let ray = Mat3::from_rotation_y(yaw) * Vec3::Z;
        let pitch_axis = ray.cross(Vec3::Y);

        // Aplly the pitch last
        Mat3::from_axis_angle(pitch_axis, pitch) * ray
    }

    /// Converts the look transform into a useful Bevy transform.
    pub fn to_transform(&self) -> Transform {
        let pitch_yaw_vector = Self::unit_vector_from_pitch_and_yaw(self.pitch, self.yaw);

        Transform::from_translation(
            self.offset
                + Vec3::new(
                    self.pitch_radius * pitch_yaw_vector.x,
                    self.yaw_radius * pitch_yaw_vector.y,
                    self.pitch_radius * pitch_yaw_vector.z,
                ),
        )
        .looking_at(self.offset + pitch_yaw_vector, Vec3::Y)
    }
}

impl Into<Transform> for LookTransform {
    fn into(self) -> Transform {
        self.to_transform()
    }
}

impl Into<Transform> for &LookTransform {
    fn into(self) -> Transform {
        self.to_transform()
    }
}

/// A struct that contains the necessary camera components for a camera with [`LookTransform`].
#[derive(Bundle)]
pub struct LookTransformCameraBundle {
    /// The camera transform bundle
    pub look_transform: LookTransform,
    /// A 3D camera bundle.
    pub camera_bundle: Camera3dBundle,
}

impl Default for LookTransformCameraBundle {
    fn default() -> Self {
        Self {
            camera_bundle: Camera3dBundle::default(),
            look_transform: LookTransform::default(),
        }
    }
}

impl LookTransformCameraBundle {
    /// Creates a new [`LookTransformCameraBundle`]
    pub fn new() -> Self {
        LookTransformCameraBundle::default()
    }
}

/// A custom velocity that is applied to kinematic controllers.
///
/// This is also used to make emulate gravity since gravity acts as a contstant acceleration.
#[derive(Debug, Clone, Component)]
pub struct CustomVelocity(pub Vec3);

impl Default for CustomVelocity {
    fn default() -> Self {
        CustomVelocity(Vec3::ZERO)
    }
}

fn apply_gravity(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut query: Query<
        (
            &mut CustomVelocity,
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
        ),
        With<KinematicCharacterController>,
    >,
) {
    for (mut velocity, mut controller, controller_output) in &mut query {
        if controller_output.grounded && (velocity.0.y < 0.0) {
            // Stop vertical movement.
            velocity.0.y = 0.0;
        } else {
            // Accelerate due to gravity.
            let new_velocity = velocity.0 + time.delta_seconds() * rapier_config.gravity;
            velocity.0 = new_velocity;
        }

        // Apply velocity.
        let translation = time.delta_seconds() * velocity.0;
        controller.translation = Some(
            controller
                .translation
                .map(|t| t + translation)
                .unwrap_or(translation),
        )
    }
}

/// A plugin that allows synchronization of [`LookTransform`] and camera transforms.
pub struct LookTransformPlugin;

impl LookTransformPlugin {
    /// Creates a new [`LookTransformPlugin`]
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for LookTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, sync_camera_transforms);
    }
}

/// Synchronizes [`LookTransform`] and camera transforms.
pub fn sync_camera_transforms(
    mut cameras: Query<(&LookTransform, &mut Transform), Changed<LookTransform>>,
) {
    for (look_transform, mut scene_transform) in cameras.iter_mut() {
        scene_transform.clone_from(&look_transform.into());
    }
}
