use crate::collision::{Collide, CollisionObject, MoveableObject, ShapeType};

use bevy::prelude::*;
use std::sync::Arc;

/// An object that can walk along the terrain of the map.
#[derive(Clone, Component)]
pub struct WalkingObject {
    pub(crate) shape: Arc<ShapeType>,
    pub(crate) nc3_shape_handle: Arc<nc3::shape::ShapeHandle<f32>>,
    pub(crate) nc3_position: nc3::na::Isometry3<f32>,
    pub(crate) nc3_velocity: nc3::na::Vector3<f32>,
    pub(crate) nc3_toi: Option<f32>,
}

impl std::fmt::Debug for WalkingObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalkingObject")
            .field("shape", &self.shape)
            .field("pos", &self.nc3_position)
            .field("vel", &self.nc3_velocity)
            .field("toi", &self.nc3_toi)
            .finish()
    }
}

impl WalkingObject {
    /// Creates a new WalkingObject.
    pub fn new(
        shape: Arc<ShapeType>,
        nc3_position: nc3::na::Isometry3<f32>,
        nc3_velocity: nc3::na::Vector3<f32>,
    ) -> Self {
        WalkingObject {
            shape: shape.clone(),
            nc3_shape_handle: Arc::new(nc3::shape::ShapeHandle::from_arc(
                crate::collision::nc3_shape_to_shape(&shape),
            )),
            nc3_position,
            nc3_velocity,
            nc3_toi: None,
        }
    }

    /// The current position.
    pub fn pos(&self) -> nc3::na::Translation<f32, 3> {
        self.nc3_position.translation
    }
}

impl MoveableObject for WalkingObject {
    fn combine_toi(&mut self, toi_other: f32) {
        match self.nc3_toi {
            None => self.nc3_toi = Some(toi_other),
            Some(toi_current) => self.nc3_toi = Some(toi_current.min(toi_other)),
        };
    }

    fn time_of_impact(&self) -> Option<f32> {
        self.nc3_toi
    }

    fn position(&self) -> nc3::na::Isometry3<f32> {
        self.nc3_position
    }

    fn velocity(&self) -> nc3::na::Vector3<f32> {
        self.nc3_velocity
    }

    fn set_position(&mut self, position: nc3::na::Isometry3<f32>) {
        self.nc3_position = position;
    }
}

impl CollisionObject for WalkingObject {
    fn shape(&self) -> Arc<ShapeType> {
        self.shape.clone()
    }

    fn nc3_shape_handle(&self) -> Arc<nc3::shape::ShapeHandle<f32>> {
        // Optimize to reduce calls to nc3_shape_to_shape.
        self.nc3_shape_handle.clone()
    }

    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        self.nc3_position
    }

    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        self.nc3_velocity
    }
}

impl Collide<WalkingObject> for WalkingObject {
    fn collide_with(this: &mut Self, other: &mut WalkingObject, collision: nc3::query::TOI<f32>) {
        this.combine_toi(collision.toi);
        other.combine_toi(collision.toi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_no_collide() {
        let o1 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(0., 0., 0.),
        );
        let o2 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(0., 0., 0.),
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!("collision_walking::test_simple_no_collide: {:?}", collision);
        assert!(collision.is_none());
    }

    #[test]
    fn test_simple_collide() {
        let o1 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(1., 0., 0.),
        );
        let o2 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(-1., 0., 0.),
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!("collision_walking::test_simple_collide: {:?}", collision);
        assert!(collision.is_some());
        if let Some(collision) = collision {
            assert!((collision.toi - 4.).abs() <= 1e-6);
        }
    }

    #[test]
    fn test_no_collide_exceeds_max_toi() {
        let o1 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(1., 0., 0.),
        );
        let o2 = WalkingObject::new(
            Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            nc3::na::Vector3::<f32>::new(0., 0., 0.),
        );
        let collision = o1.get_collision_with(&o2, 1.);
        println!(
            "collision_walking::test_no_collide_exceeds_max_toi: {:?}",
            collision
        );
        assert!(collision.is_none());
    }
}
