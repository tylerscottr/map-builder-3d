extern crate ncollide3d as nc3;

pub mod collision;
pub mod map;
pub mod player;

use bevy::prelude::*;
use collision::{BarrierObject, Collide, MoveableObject};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Obstical;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Obstical, Name("Elaina Proctor".to_string()), BarrierObject));
    commands.spawn((
        Person,
        Name("Renzo Hume".to_string()),
        MoveableObject::new(),
    ));
    commands.spawn((
        Person,
        Name("Zayna Nieves".to_string()),
        MoveableObject::new(),
    ));
    commands.spawn((
        Person,
        Name("Brock Harrison".to_string()),
        MoveableObject::new(),
    ));
}

fn greet_people(
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    mut query_collision: Query<(&Name, &mut MoveableObject), With<MoveableObject>>,
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
                obj1.1.collide_with(&mut obj2.1);
                println!("After: {:?} {:?}", obj1.1.pos(), obj2.1.pos());
            }
        }

        // Print people's names
        for tuple in query_person.iter() {
            println!("person: Hello {}!", tuple.0,);
        }
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(HelloPlugin)
        .run();
}
