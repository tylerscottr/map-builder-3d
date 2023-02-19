//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - map: A collection of 3D tiles, obstacles, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

/// A module that integrates the adds some useful functions to the Rapier physics engine.
pub mod rapier_mesh_bundles;

use rapier_mesh_bundles::*;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct LocalPlayer(Vec3);

#[derive(Component)]
struct Name(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(print_ball_altitude)
        .add_system_to_stage(CoreStage::PostUpdate, sync_camera)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    const ZOOM: f32 = 3.;
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0 * ZOOM, 3.0 * ZOOM, 10.0 * ZOOM)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Light bulb
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
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
    /* Create the ground. */
    commands.spawn(RapierColliderPbrBundle {
        shape: RapierShapeBundle::cuboid(Vec3::new(15.0, 5.0, 15.0), &mut meshes),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -4.5, 0.0),
        ..default()
    });
    commands.spawn(RapierColliderPbrBundle {
        shape: RapierShapeBundle::plane(Vec2::new(5.0, 5.0), &mut meshes),
        material: materials.add(Color::rgb(0.2, 0.4, 0.2).into()),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });

    /* Create the bouncing ball. */
    commands
        .spawn(Name("Kong Ball".into()))
        .insert(RigidBody::Dynamic)
        .insert(Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        })
        .insert(Velocity {
            linvel: Vec3::new(1.0, 2.0, 3.0),
            // angvel: Vec3::ZERO,
            angvel: Vec3::new(0.2, -1.0, 0.0),
        })
        .with_children(|children| {
            children
                .spawn(RapierColliderPbrBundle {
                    shape: RapierShapeBundle::sphere(0.5, &mut meshes),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.3).into()),
                    transform: Transform::from_xyz(0.0, -0.25, 0.0),
                    ..default()
                })
                .insert(Restitution {
                    coefficient: 0.7,
                    combine_rule: CoefficientCombineRule::Max,
                });
            children
                .spawn(RapierColliderPbrBundle {
                    shape: RapierShapeBundle::sphere(0.5, &mut meshes),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.3).into()),
                    transform: Transform::from_xyz(0.0, 0.25, 0.0),
                    ..default()
                })
                .insert(Restitution {
                    coefficient: 0.7,
                    combine_rule: CoefficientCombineRule::Max,
                });
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
        .insert(VisibilityBundle::default());

    /* Create the bouncing capsule. */
    commands
        .spawn(LocalPlayer(0.9 * Vec3::Y))
        .insert(Name("Capsule".into()))
        .insert(RigidBody::Dynamic)
        .insert(RapierColliderPbrBundle {
            shape: RapierShapeBundle::capsule(0.5, 0.5, &mut meshes),
            material: materials.add(Color::rgb(0.3, 0.3, 0.7).into()),
            transform: Transform::from_xyz(-1.0, 5.0, -1.0),
            ..default()
        });
}

fn print_ball_altitude(positions: Query<(&Name, &Transform), With<RigidBody>>) {
    for (name, transform) in positions.iter() {
        println!("Altitude of {}: {}", name.0, transform.translation.y);
    }
}

fn sync_camera(
    local_players: Query<(&LocalPlayer, &Transform), With<LocalPlayer>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<LocalPlayer>)>,
) {
    let Some((local_player_0, transform_player_0)) = local_players.iter().next() else { return; };
    let Some(mut transform_cam_0) = cameras.iter_mut().next() else { return; };

    let player_pos = transform_player_0.transform_point(Vec3::ZERO);
    let new_cam_pos = player_pos + local_player_0.0;
    transform_cam_0.as_mut().clone_from(
        &Transform::from_translation(new_cam_pos)
            .looking_at(new_cam_pos + 1. * Vec3::Z - 0. * Vec3::Y, Vec3::Y),
    );
}
