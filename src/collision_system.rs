use crate::collision::{Collide, MoveableObject};
use crate::collision_obstacle::ObstacleObject;
use crate::collision_walking::WalkingObject;

use bevy::prelude::*;

/// Finds all collisions between walking objects and obstacles.
///
/// This function should be called within the system that handles walking objects and obstaicals.
pub fn process_collisions_walking_obstaicals(
    time: &Res<Time>,
    query_walking: &mut Query<&mut WalkingObject, With<WalkingObject>>,
    query_obstacle: &mut Query<&mut ObstacleObject, With<ObstacleObject>>,
) {
    // Determine which walking objects collide with obstacles.
    query_walking.for_each_mut(|mut obj1| {
        query_obstacle.for_each_mut(|mut obj2| {
            // Determine if they will collide within this frame
            if let Some(collision) = obj1
                .as_ref()
                .get_collision_with(obj2.as_ref(), time.delta_seconds())
            {
                // Prevent the collision by stopping both objects just as they touch
                <_>::collide_with(obj1.as_mut(), obj2.as_mut(), collision);
            }
        })
    });
}

/// Finds all collisions between walking objects and updates the internal
/// time-of-impacts.
///
/// This function should be called within the system that handles walking objects.
pub fn process_collisions_walking(
    time: &Res<Time>,
    query_walking: &mut Query<&mut WalkingObject, With<WalkingObject>>,
) {
    // Determine which walking objects collide with each other.
    let mut combinations_walking = query_walking.iter_combinations_mut();
    while let Some([mut obj1, mut obj2]) = combinations_walking.fetch_next() {
        // Determine if they will collide within this frame
        if let Some(collision) = obj1
            .as_ref()
            .get_collision_with(obj2.as_ref(), time.delta_seconds())
        {
            // Prevent the collision by stopping both objects just as they touch
            <_>::collide_with(obj1.as_mut(), obj2.as_mut(), collision);
        }
    }
}

/// Updates the positions of all walking objects based on their time of impact.
///
/// This function should be called within system that handles walking objects.
pub fn update_positions_walking(
    time: &Res<Time>,
    query_walking: &mut Query<&mut WalkingObject, With<WalkingObject>>,
) {
    // Update positions for each walking object
    query_walking.for_each_mut(|mut obj| {
        obj.update_position_for_frame(time.delta());
    });
}

/// The default Bevy system for operating walking objects.
pub fn system_walking_default(
    time: Res<Time>,
    mut query_walking: Query<&mut WalkingObject, With<WalkingObject>>,
    mut query_obstacle: Query<&mut ObstacleObject, With<ObstacleObject>>,
) {
    process_collisions_walking_obstaicals(&time, &mut query_walking, &mut query_obstacle);
    process_collisions_walking(&time, &mut query_walking);
    update_positions_walking(&time, &mut query_walking);
}
