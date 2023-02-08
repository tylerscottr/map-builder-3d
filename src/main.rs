//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - Collision detection: Uses ncollide3d in a Bevy-friendly way so as to allow objects with
//! ncollide3d shapes to be assets.
//! - Map: A collection of 3D tiles, obsticals, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

extern crate ncollide3d as nc3;

/// A module that determines which objects collide with each other
pub mod collision;

use std::sync::Arc;

use bevy::prelude::*;
use collision::{Collide, ObsticalObject, ShapeType, WalkingObject};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Obstical;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Obstical, Name("Elaina Proctor".to_string()), ObsticalObject));
    commands.spawn((
        Person,
        Name("Renzo Hume".to_string()),
        WalkingObject::new(Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.)))),
    ));
    commands.spawn((
        Person,
        Name("Zayna Nieves".to_string()),
        WalkingObject::new(Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.)))),
    ));
    commands.spawn((
        Person,
        Name("Brock Harrison".to_string()),
        WalkingObject::new(Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.)))),
    ));
}

fn greet_people(
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    mut query_collision: Query<(&Name, &mut WalkingObject), With<WalkingObject>>,
    query_person: Query<&Name, With<Person>>,
) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        // Determine which objects collide with each other
        let mut collision_objs = query_collision.iter_mut().collect::<Vec<_>>();
        for index1 in 0..collision_objs.len() {
            for index2 in (index1 + 1)..collision_objs.len() {
                let (left, right) = collision_objs.split_at_mut(index2);
                let obj1 = left.get_mut(index1).unwrap();
                let obj2 = right.get_mut(0).unwrap();
                println!("Before: {:?} {:?}", obj1.1.pos(), obj2.1.pos());
                if let Some(collision) = obj1.1.get_collision_with(&obj2.1) {
                    obj1.1.collide_with(&mut obj2.1, collision);
                }
                println!("After: {:?} {:?}", obj1.1.pos(), obj2.1.pos());
            }
        }

        // Print people's names
        for tuple in query_person.iter() {
            println!("person: Hello {}!", tuple.0,);
        }
    }
}

/// The default plugin for Bevy to get full functionality of this crate
pub struct MapBuilderDefaultPlugin;

impl Plugin for MapBuilderDefaultPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MapBuilderDefaultPlugin)
        .run();
}
