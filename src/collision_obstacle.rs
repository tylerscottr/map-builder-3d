use crate::collision::{Collide, CollisionObject, MoveableObject, ShapeType, ShapeTypeWithHandle};
use crate::collision_walking::WalkingObject;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An object that prevents moving objects from passing through
#[derive(Clone, Deserialize, Serialize, Component)]
pub struct ObstacleObject {
    pub(crate) shape: ShapeTypeWithHandle,
    pub(crate) nc3_position: nc3::na::Isometry3<f32>,
}

impl std::fmt::Debug for ObstacleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObstacleObject")
            .field("shape", &self.shape.shape)
            .field("pos", &self.nc3_position)
            .finish()
    }
}

impl ObstacleObject {
    /// Creates a new ObstacleObject.
    pub fn new(shape: &Arc<ShapeType>, nc3_position: &nc3::na::Isometry3<f32>) -> Self {
        ObstacleObject {
            shape: ShapeTypeWithHandle::new(shape),
            nc3_position: *nc3_position,
        }
    }

    /// The current position.
    pub fn pos(&self) -> nc3::na::Translation<f32, 3> {
        self.nc3_position.translation
    }
}

impl CollisionObject for ObstacleObject {
    fn shape(&self) -> &ShapeTypeWithHandle {
        &self.shape
    }

    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        self.nc3_position
    }

    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        nc3::na::zero()
    }
}

impl Collide<ObstacleObject> for WalkingObject {
    fn collide_with(obj1: &mut Self, _obj2: &mut ObstacleObject, collision: nc3::query::TOI<f32>) {
        obj1.combine_toi(collision.toi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_no_collide() {
        let o1 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(0., 0., 0.),
        );
        let o2 = ObstacleObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!(
            "collision_obstacle::test_simple_no_collide: {:?}",
            collision
        );
        assert!(collision.is_none());
    }

    #[test]
    fn test_simple_collide() {
        let o1 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(1., 0., 0.),
        );
        let o2 = ObstacleObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!("collision_obstacle::test_simple_collide: {:?}", collision);
        assert!(collision.is_some());
        if let Some(collision) = collision {
            assert!((collision.toi - 8.).abs() <= 1e-6);
        }
    }

    #[test]
    fn test_no_collide_exceeds_max_toi() {
        let o1 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(1., 0., 0.),
        );
        let o2 = ObstacleObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
        );
        let collision = o1.get_collision_with(&o2, 1.);
        println!(
            "collision_obstacle::test_no_collide_exceeds_max_toi: {:?}",
            collision
        );
        assert!(collision.is_none());
    }
}
