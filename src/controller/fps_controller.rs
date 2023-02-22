//! A person-person style controller.
//!
//! Note: Much of the following code has been copied from `smooth_bevy_cameras` and heavily
//! modified rotate the camera while translating the collider and mesh.

use super::*;

use bevy::{
    app::prelude::*,
    ecs::prelude::*,
    input::{mouse::*, prelude::*},
    math::prelude::*,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

/// Types of events that can be triggered for kinematic controllers.
pub enum FpsControlEvent {
    /// Rotate the camera view.
    RotateCamera(Vec2),
    /// Translate the character.
    Translate(Vec3),
    /// Have the character start a jump.
    Jump(Vec3),
}

/// A struct that contains the necessary body components to implement the [`FpsCameraPlugin`].
#[derive(Bundle, Clone)]
pub struct FpsControllerBodyBundle {
    /// The Rapier rigid body type.
    rigid_body: RigidBody,
    /// The Rapier kinematic character controller
    character_controller: KinematicCharacterController,
    /// The additional velocity enacted on the character.
    ///
    /// Used to simulate gravity.
    additional_velocity: CustomVelocity,
}

impl Default for FpsControllerBodyBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::KinematicPositionBased,
            character_controller: KinematicCharacterController {
                translation: Some(Vec3::ZERO), // Allow gravity to be applied from the start
                apply_impulse_to_dynamic_bodies: true,
                ..default()
            },
            additional_velocity: CustomVelocity::default(),
        }
    }
}

impl FpsControllerBodyBundle {
    /// Creates a new [`FpsControllerBodyBundle`]
    pub fn new() -> Self {
        FpsControllerBodyBundle::default()
    }
}

/// A plugin that allows for custom character control in a first-person shooter style.
pub struct FpsCameraPlugin {}

impl FpsCameraPlugin {
    /// Creates a new [`FpsCameraPlugin`]
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, apply_gravity)
            .add_system(custom_input_map)
            .add_system(fps_control_system)
            .add_event::<FpsControlEvent>();
    }
}

/// Handles mouse and keyboard events.
pub fn custom_input_map(
    mut events: EventWriter<FpsControlEvent>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let translate_velocity = 2.0;
    let mouse_rotate_sensitivity = Vec2::splat(0.1);
    let jump_initial_velocity = 5.0 * Vec3::Y;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    events.send(FpsControlEvent::RotateCamera(
        mouse_rotate_sensitivity * cursor_delta,
    ));

    let translation_dir_option = [
        (KeyCode::W, Vec3::Z),
        (KeyCode::A, Vec3::X),
        (KeyCode::S, -Vec3::Z),
        (KeyCode::D, -Vec3::X),
    ]
    .iter()
    .fold(None, |dir_acc, &(key, dir)| {
        if keyboard.pressed(key) {
            return Some(dir_acc.map_or(dir, |acc| acc + dir));
        } else {
            return dir_acc;
        }
    });

    if let Some(translation_dir) = translation_dir_option {
        events.send(FpsControlEvent::Translate(
            translate_velocity * translation_dir.normalize(),
        ));
    }

    if keyboard.pressed(KeyCode::Space) {
        events.send(FpsControlEvent::Jump(jump_initial_velocity));
    }
}

/// Implements the control system for [`FpsCameraPlugin`].
pub fn fps_control_system(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut events: EventReader<FpsControlEvent>,
    mut cameras: Query<(&Parent, &mut LookTransform, &mut Transform)>,
    mut controllers: Query<(
        &mut KinematicCharacterController,
        &mut CustomVelocity,
        &KinematicCharacterControllerOutput,
    )>,
) {
    for (parent, mut look_transform, mut transform) in &mut cameras {
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, look_transform.yaw);
        let rot_x = yaw_rot * Vec3::X;
        let rot_y = yaw_rot * Vec3::Y;
        let rot_z = yaw_rot * Vec3::Z;

        let dt = time.delta_seconds();
        for event in events.iter() {
            match event {
                FpsControlEvent::RotateCamera(delta) => {
                    // Rotates with pitch and yaw.
                    look_transform.pitch += dt * -delta.y;
                    look_transform.yaw += dt * -delta.x;
                    (*transform).clone_from(&look_transform.as_ref().into());
                }
                FpsControlEvent::Translate(delta) => {
                    // Translates the parent up/down (Y) left/right (X) and forward/back (Z).
                    if let Ok((mut parent_controller, _, _)) = controllers.get_mut(parent.get()) {
                        let translation = dt
                            * (delta.x * rot_x + delta.y * rot_y + delta.z * rot_z)
                            * rapier_context.physics_scale();
                        parent_controller.translation = Some(
                            parent_controller
                                .translation
                                .map(|t| t + translation)
                                .unwrap_or(translation),
                        );
                    }
                }
                FpsControlEvent::Jump(jump_velocity) => {
                    // Start a jump
                    if let Ok((_, mut velocity, parent_controller_output)) =
                        controllers.get_mut(parent.get())
                    {
                        if parent_controller_output.grounded {
                            velocity.0 = *jump_velocity * rapier_context.physics_scale();
                        }
                    }
                }
            }
        }
    }
}
