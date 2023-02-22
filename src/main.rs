//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - map: A collection of 3D tiles, obstacles, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

/// A module that integrates the adds some useful functions to the Rapier physics engine.
pub mod rapier_mesh_bundles;

/// A module that adds mouse/keyboard control to the camera.
pub mod controller;

use controller::{fps_controller::*, *};
use rapier_mesh_bundles::*;

use bevy::{core_pipeline::clear_color::*, pbr::*, prelude::*, render::camera::*, window::*};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

#[derive(Component)]
struct Name(String);

const PHYSICAL_SCALE: f32 = 1.0;

fn main() {
    App::new()
        // .insert_resource(DirectionalLightShadowMap { size: 2048 }) // Higher values cause lag!
        .insert_resource(RapierConfiguration {
            gravity: RapierConfiguration::default().gravity * PHYSICAL_SCALE,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Map Builder 3D".to_string(),
                width: 1280.0,
                height: 720.0,
                position: WindowPosition::Centered,
                resizable: true,
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(PHYSICAL_SCALE))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::new())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        // .add_system(print_ball_altitude)
        .add_system(set_camera_viewports)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    const CAM_DISTANCE: f32 = 30.;
    let initial_cam_pos = CAM_DISTANCE * Vec3::new(-3.0, 3.0, 10.0).normalize() * PHYSICAL_SCALE;
    commands
        .spawn(LeftCamera)
        .insert(LookTransformCameraBundle {
            look_transform: LookTransform::from_pos_target(initial_cam_pos, Vec3::ZERO),
            ..default()
        });

    // Light bulb
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0) * PHYSICAL_SCALE,
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the ground.
    commands.spawn(RapierColliderPbrBundle {
        shape: RapierShapeBundle::cuboid(Vec3::new(15.0, 5.0, 15.0) * PHYSICAL_SCALE, &mut meshes),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_translation(Vec3::new(0.0, -4.5, 0.0) * PHYSICAL_SCALE),
        ..default()
    });
    // commands.spawn(RapierColliderPbrBundle {
    //     shape: RapierShapeBundle::plane(Vec2::new(5.0, 5.0), &mut meshes),
    //     material: materials.add(Color::rgb(0.2, 0.4, 0.2).into()),
    //     transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
    //     ..default()
    // });
    commands.spawn(RapierColliderPbrBundle {
        shape: RapierShapeBundle::cuboid(Vec3::new(4.0, 2.5, 4.0) * PHYSICAL_SCALE, &mut meshes),
        material: materials.add(Color::rgb(0.2, 0.2, 0.4).into()),
        transform: Transform::from_translation(Vec3::new(0.0, -0.5, 0.0) * PHYSICAL_SCALE),
        ..default()
    });

    // Create the bouncing ball.
    commands
        .spawn(Name("Kong Ball".into()))
        .insert(RigidBody::Dynamic)
        .insert(Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        })
        .insert(Velocity {
            linvel: Vec3::new(1.0, 2.0, 3.0) * PHYSICAL_SCALE,
            // angvel: Vec3::ZERO,
            angvel: Vec3::new(0.2, -1.0, 0.0),
        })
        .with_children(|children| {
            children
                .spawn(RapierColliderPbrBundle {
                    shape: RapierShapeBundle::sphere(0.5 * PHYSICAL_SCALE, &mut meshes),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.3).into()),
                    transform: Transform::from_translation(
                        Vec3::new(0.0, -0.25, 0.0) * PHYSICAL_SCALE,
                    ),
                    ..default()
                })
                .insert(Restitution {
                    coefficient: 0.7,
                    combine_rule: CoefficientCombineRule::Max,
                });
            children
                .spawn(RapierColliderPbrBundle {
                    shape: RapierShapeBundle::sphere(0.5 * PHYSICAL_SCALE, &mut meshes),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.3).into()),
                    transform: Transform::from_translation(
                        Vec3::new(0.0, 0.25, 0.0) * PHYSICAL_SCALE,
                    ),
                    ..default()
                })
                .insert(Restitution {
                    coefficient: 0.7,
                    combine_rule: CoefficientCombineRule::Max,
                });
        })
        .insert(TransformBundle::from(Transform::from_translation(
            Vec3::new(0.0, 4.0, 0.0) * PHYSICAL_SCALE,
        )))
        .insert(VisibilityBundle::default());

    // Create the bouncing capsule.
    let capsule_pos = Vec3::new(-1.0, 5.0, -1.0) * PHYSICAL_SCALE;
    commands
        .spawn(Name("Capsule".into()))
        .insert(RapierColliderPbrBundle {
            shape: RapierShapeBundle::capsule(
                0.5 * PHYSICAL_SCALE,
                0.5 * PHYSICAL_SCALE,
                &mut meshes,
            ),
            material: materials.add(Color::rgb(0.3, 0.3, 0.7).into()),
            transform: Transform::from_translation(capsule_pos),
            ..default()
        })
        .insert(FpsControllerBodyBundle::new())
        .with_children(|children| {
            children
                .spawn(RightCamera)
                .insert(LookTransformCameraBundle {
                    camera_bundle: Camera3dBundle {
                        camera: Camera {
                            // Renders the right camera after the left camera, which has a default priority of 0
                            priority: 1,
                            ..default()
                        },
                        camera_3d: Camera3d {
                            // don't clear on the second camera because the first camera already cleared the window
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                });
        });
}

// fn print_ball_altitude(positions: Query<(&Name, &Transform), With<RigidBody>>) {
//     for (name, transform) in positions.iter() {
//         println!("Altitude of {}: {}", name.0, transform.translation.y);
//     }
// }

fn set_camera_viewports(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        let window = windows.get(resize_event.id).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });

        let mut right_camera = right_camera.single_mut();
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.physical_width() / 2, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });
    }
}
