use crate::collision::{
    Collide, CollisionObject, MoveableObject, PositionOffset, ShapeType, ShapeTypeWithHandle,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An object that can walk along the terrain of the map.
#[derive(Clone, Deserialize, Serialize, Component)]
#[component(storage = "SparseSet")]
pub struct WalkingObject {
    pub(crate) shape: ShapeTypeWithHandle,
    pub(crate) nc3_position: nc3::na::Isometry3<f32>,
    pub(crate) nc3_velocity: nc3::na::Vector3<f32>,
    pub(crate) nc3_toi: Option<f32>,
    pub(crate) shape_offset: PositionOffset,
}

impl std::fmt::Debug for WalkingObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalkingObject")
            .field("shape", &self.shape.shape)
            .field("pos", &self.nc3_position)
            .field("vel", &self.nc3_velocity)
            .field("toi", &self.nc3_toi)
            .finish()
    }
}

impl WalkingObject {
    /// Creates a new WalkingObject.
    pub fn new(
        shape: &Arc<ShapeType>,
        nc3_position: &nc3::na::Isometry3<f32>,
        nc3_velocity: &nc3::na::Vector3<f32>,
        shape_offset: &PositionOffset,
    ) -> Self {
        WalkingObject {
            shape: ShapeTypeWithHandle::new(shape),
            nc3_position: *nc3_position,
            nc3_velocity: *nc3_velocity,
            nc3_toi: None,
            shape_offset: *shape_offset,
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
    fn shape(&self) -> &ShapeTypeWithHandle {
        &self.shape
    }

    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        self.nc3_position
    }

    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        self.nc3_velocity
    }

    fn default_shape_offset_isometry(&self) -> nc3::na::Isometry3<f32> {
        let aabb = self
            .shape
            .nc3_shape_handle
            .aabb(&nc3::na::Isometry3::<f32>::identity());
        nc3::na::Isometry3::<f32>::from_parts(
            nc3::na::Translation3::<f32>::new(aabb.center().x, aabb.center().y, aabb.maxs.z)
                .inverse(),
            nc3::na::UnitQuaternion::<f32>::identity(),
        )
    }

    fn shape_offset(&self) -> PositionOffset {
        self.shape_offset
    }
}

impl Collide<WalkingObject> for WalkingObject {
    fn collide_with(obj1: &mut Self, obj2: &mut WalkingObject, collision: nc3::query::TOI<f32>) {
        obj1.combine_toi(collision.toi);
        obj2.combine_toi(collision.toi);
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
            &PositionOffset::Default,
        );
        let o2 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(0., 0., 0.),
            &PositionOffset::Default,
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!("collision_walking::test_simple_no_collide: {:?}", collision);
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
            &PositionOffset::Default,
        );
        let o2 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(-1., 0., 0.),
            &PositionOffset::Default,
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
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(1., 0., 0.),
            &PositionOffset::Default,
        );
        let o2 = WalkingObject::new(
            &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
            &nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(10., 0., 0.),
                nc3::na::zero(),
            ),
            &nc3::na::Vector3::<f32>::new(0., 0., 0.),
            &PositionOffset::Default,
        );
        let collision = o1.get_collision_with(&o2, 1.);
        println!(
            "collision_walking::test_no_collide_exceeds_max_toi: {:?}",
            collision
        );
        assert!(collision.is_none());
    }
}
