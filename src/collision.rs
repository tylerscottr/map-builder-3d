use std::sync::Arc;

use bevy::prelude::*;
use nc3::{
    na::Isometry3,
    shape::{
        Ball, Capsule, ConvexHull, Cuboid, HeightField, Plane, Segment, ShapeHandle, TriMesh,
        Triangle,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ShapeType {
    Ball(Ball<f32>),
    Capsule(Capsule<f32>),
    ConvexHull(ConvexHull<f32>),
    Cuboid(Cuboid<f32>),
    HeightField(HeightField<f32>),
    Plane(Plane<f32>),
    Segment(Segment<f32>),
    TriMesh(TriMesh<f32>),
    Triangle(Triangle<f32>),
    Compound(Vec<(Isometry3<f32>, ShapeType)>),
}

impl std::fmt::Debug for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeType::Ball(ball) => f
                .debug_struct("Ball")
                .field("radius", &ball.radius)
                .finish(),
            ShapeType::Capsule(capsule) => f
                .debug_struct("Capsule")
                .field("half_height", &capsule.half_height)
                .field("radius", &capsule.radius)
                .finish(),
            ShapeType::ConvexHull(hull) => f
                .debug_struct("ConvexHull")
                .field("points", &hull.points().len())
                .finish(),
            ShapeType::Cuboid(cuboid) => f
                .debug_struct("Cuboid")
                .field("half_extents", &cuboid.half_extents)
                .finish(),
            ShapeType::HeightField(height_field) => f
                .debug_struct("HeightField")
                .field("heights", &height_field.heights().len())
                .finish(),
            ShapeType::Plane(plane) => f
                .debug_struct("Plane")
                .field("normal", &plane.normal)
                .finish(),
            ShapeType::Segment(segment) => f
                .debug_struct("Segment")
                .field("a", &segment.a)
                .field("b", &segment.b)
                .finish(),
            ShapeType::TriMesh(mesh) => f
                .debug_struct("TriMesh")
                .field("points", &mesh.points().len())
                .finish(),
            ShapeType::Triangle(triangle) => f
                .debug_struct("Triangle")
                .field("a", &triangle.a)
                .field("b", &triangle.b)
                .field("c", &triangle.c)
                .finish(),
            ShapeType::Compound(compound) => f.debug_list().entries(compound.iter()).finish(),
        }
    }
}

fn nc3_shape_to_shape(shape: &ShapeType) -> Arc<dyn nc3::shape::Shape<f32>> {
    match shape {
        ShapeType::Ball(ball) => Arc::new(ball.clone()),
        ShapeType::Capsule(capsule) => Arc::new(capsule.clone()),
        ShapeType::ConvexHull(hull) => Arc::new(hull.clone()),
        ShapeType::Cuboid(cuboid) => Arc::new(cuboid.clone()),
        ShapeType::HeightField(height_field) => Arc::new(height_field.clone()),
        ShapeType::Plane(plane) => Arc::new(plane.clone()),
        ShapeType::Segment(segment) => Arc::new(segment.clone()),
        ShapeType::TriMesh(mesh) => Arc::new(mesh.clone()),
        ShapeType::Triangle(triangle) => Arc::new(triangle.clone()),
        ShapeType::Compound(compound) => Arc::new(nc3::shape::Compound::new(
            compound
                .iter()
                .map(|(iso, shape)| (*iso, ShapeHandle::from_arc(nc3_shape_to_shape(shape))))
                .collect::<Vec<_>>(),
        )),
    }
}

pub trait CollisionObject {
    fn shape(&self) -> Arc<ShapeType>;

    fn nc3_shape_handle(&self) -> Arc<nc3::shape::ShapeHandle<f32>> {
        Arc::new(nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(
            &self.shape(),
        )))
    }

    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        nc3::na::Isometry3::<f32>::identity()
    }

    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        nc3::na::Vector3::<f32>::zeros()
    }
}

#[derive(Clone, Component)]
pub struct MoveableObject {
    shape: Arc<ShapeType>,
    nc3_shape_handle: Arc<nc3::shape::ShapeHandle<f32>>,
    nc3_position: nc3::na::Isometry3<f32>,
    nc3_velocity: nc3::na::Vector3<f32>,
}

impl std::fmt::Debug for MoveableObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MoveableObject")
            .field("shape", &self.shape)
            .finish()
    }
}

impl MoveableObject {
    pub fn new() -> Self {
        let new_shape = Arc::new(ShapeType::Ball(Ball::<f32>::new(1.)));
        MoveableObject {
            shape: new_shape.clone(),
            nc3_shape_handle: Arc::new(nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(
                &new_shape,
            ))),
            nc3_position: nc3::na::Isometry3::<f32>::new(
                nc3::na::Vector3::<f32>::new(0., 0., 0.),
                nc3::na::zero(),
            ),
            nc3_velocity: nc3::na::Vector3::<f32>::new(0., 0., 0.),
        }
    }

    pub fn pos(&self) -> nc3::na::Translation<f32, 3> {
        self.nc3_position.translation
    }
}

impl CollisionObject for MoveableObject {
    fn shape(&self) -> Arc<ShapeType> {
        self.shape.clone()
    }

    fn nc3_shape_handle(&self) -> Arc<nc3::shape::ShapeHandle<f32>> {
        // Optimize to reduce calls to nc3_shape_to_shape
        self.nc3_shape_handle.clone()
    }

    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        self.nc3_position
    }

    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        self.nc3_velocity
    }
}

#[derive(Debug, Component)]
pub struct BarrierObject;

impl CollisionObject for BarrierObject {
    fn shape(&self) -> Arc<ShapeType> {
        return Arc::new(ShapeType::Capsule(Capsule::<f32>::new(1., 1.)));
    }
}

// TODO StationaryObject/Barrier
// TODO EventSpace

pub trait Collide<A: CollisionObject>: CollisionObject {
    fn collide_with(&mut self, other: &mut A);

    fn get_collition_with(&self, other: &A) -> Option<nc3::query::TOI<f32>> {
        nc3::query::time_of_impact(
            &nc3::query::DefaultTOIDispatcher,
            &self.nc3_position(),
            &self.nc3_velocity(),
            self.nc3_shape_handle().as_arc().as_ref(),
            &other.nc3_position(),
            &other.nc3_velocity(),
            other.nc3_shape_handle().as_arc().as_ref(),
            std::f32::MAX,
            0.0,
        )
        .unwrap()
    }
}

impl Collide<MoveableObject> for MoveableObject {
    fn collide_with(&mut self, other: &mut MoveableObject) {
        // self.pos -= nc3::na::Vector3::<f32>::new(0.1, 0., 0.);
        // other.pos += nc3::na::Vector3::<f32>::new(0.1, 0., 0.);
        let Some(_collision) = self.get_collition_with(other) else { return; };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_collide_with_traits() {
        let o1 = MoveableObject::new();
        let o2 = MoveableObject::new();
        assert!(o1.get_collition_with(&o2).is_none());
    }
}
