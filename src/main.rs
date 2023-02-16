//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - [collision]: Uses ncollide3d in a Bevy-friendly way so as to allow objects with
//! ncollide3d shapes to be assets.
//! - map: A collection of 3D tiles, obstacles, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

pub extern crate ncollide3d as nc3;

/// A module that determines which objects collide with each other.
pub mod collision;

/// A module for creating and interacting with walking objects.
pub mod collision_walking;

/// A module for creating and interacting with obstacles.
pub mod collision_obstacle;

/// A module that handles object collisions in the event loop.
pub mod collision_system;

use std::sync::Arc;

use bevy::prelude::*;
use collision::{PositionOffset, ShapeType};
use collision_obstacle::ObstacleObject;
use collision_walking::WalkingObject;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct DebugMessageTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((
        Name("Elaina Proctor".to_string()),
        ObstacleObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 10., 0.),
                nc3::na::zero(),
            ),
            &PositionOffset::Default,
        ),
    ));
    commands.spawn((
        Person,
        Name("Renzo Hume".to_string()),
        WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(0.5, 0., 0.),
            &PositionOffset::Default,
        ),
    ));
    commands.spawn((
        Person,
        Name("Zayna Nieves".to_string()),
        WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(-0.5, 0., 0.),
            &PositionOffset::Default,
        ),
    ));
    commands.spawn((
        Person,
        Name("Brock Harrison".to_string()),
        WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 10., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(1., 0., 0.),
            &PositionOffset::Default,
        ),
    ));
}

type QueryWalkingNamed<'ref1, 'ref2, 'world, 'state> =
    Query<'world, 'state, (&'ref1 Name, &'ref2 WalkingObject), (With<Name>, With<WalkingObject>)>;
fn print_debug_messages(
    time: Res<Time>,
    mut timer: ResMut<DebugMessageTimer>,
    query_walking_named: QueryWalkingNamed,
) {
    // Update our timer with the time elapsed since the last update. If that caused the timer to
    // finish, we print some debug messages.
    if timer.0.tick(time.delta()).just_finished() {
        // Print walking object locations
        println!("Walkable objects:");
        for tuple in query_walking_named.iter() {
            println!("  > {}: {:?}", tuple.0 .0, tuple.1.pos());
        }
        println!();
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
