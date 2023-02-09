//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - [collision]: Uses ncollide3d in a Bevy-friendly way so as to allow objects with
//! ncollide3d shapes to be assets.
//! - map: A collection of 3D tiles, obsticals, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

extern crate ncollide3d as nc3;

/// A module that determines which objects collide with each other
pub mod collision;

/// A module that handles object collisions in the event loop
pub mod collision_system;

use std::sync::Arc;

use bevy::prelude::*;
use collision::{ObsticalObject, ShapeType, WalkingObject};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Obstical;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct DebugMessageTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Obstical, Name("Elaina Proctor".to_string()), ObsticalObject));
    commands.spawn((
        Person,
        Name("Renzo Hume".to_string()),
        WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(0.5, 0., 0.),
        ),
    ));
    commands.spawn((
        Person,
        Name("Zayna Nieves".to_string()),
        WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(-0.5, 0., 0.),
        ),
    ));
    commands.spawn((
        Person,
        Name("Brock Harrison".to_string()),
        WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 10., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(0., 0., 0.),
        ),
    ));
}

fn print_debug_messages(
    time: Res<Time>,
    mut timer: ResMut<DebugMessageTimer>,
    query_walking_named: Query<(&Name, &WalkingObject), (With<Name>, With<WalkingObject>)>,
) {
    // Update our timer with the time elapsed since the last update. If that caused the timer to
    // finish, we print some debug messages.
    if timer.0.tick(time.delta()).just_finished() {
        // Print walking object locations
        println!("Walkable objects:");
        for tuple in query_walking_named.iter() {
            println!("  > {}: {:?}", tuple.0 .0, tuple.1.pos());
        }
        println!("");
    }
}

/// The default plugin for Bevy to get full functionality of this crate
pub struct MapBuilderDefaultPlugin;

impl Plugin for MapBuilderDefaultPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugMessageTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .add_startup_system(add_people)
        .add_system(collision_system::system_walking_default)
        .add_system(print_debug_messages);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MapBuilderDefaultPlugin)
        .run();
}
