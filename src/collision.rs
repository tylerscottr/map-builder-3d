use std::sync::Arc;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// TODO EventSpace

/// An enumeration describing the various ncollide3d shapes that are available as assets.
#[derive(Serialize, Deserialize, Clone)]
pub enum ShapeType {
    /// A ball shape
    Ball(nc3::shape::Ball<f32>),
    /// A capsule shape
    Capsule(nc3::shape::Capsule<f32>),
    /// A convex hull shape
    ConvexHull(nc3::shape::ConvexHull<f32>),
    /// A cuboid shape
    Cuboid(nc3::shape::Cuboid<f32>),
    /// A height field
    HeightField(nc3::shape::HeightField<f32>),
    /// A plane
    Plane(nc3::shape::Plane<f32>),
    /// A segment
    Segment(nc3::shape::Segment<f32>),
    /// A triangle mesh shape
    TriMesh(nc3::shape::TriMesh<f32>),
    /// A triangle shape
    Triangle(nc3::shape::Triangle<f32>),
    /// A compound shape comprised of a vector of other shapes
    Compound(Vec<(nc3::na::Isometry3<f32>, ShapeType)>),
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
        ShapeType::Ball(ball) => Arc::new(*ball),
        ShapeType::Capsule(capsule) => Arc::new(*capsule),
        ShapeType::ConvexHull(hull) => Arc::new(hull.clone()),
        ShapeType::Cuboid(cuboid) => Arc::new(*cuboid),
        ShapeType::HeightField(height_field) => Arc::new(height_field.clone()),
        ShapeType::Plane(plane) => Arc::new(plane.clone()),
        ShapeType::Segment(segment) => Arc::new(*segment),
        ShapeType::TriMesh(mesh) => Arc::new(mesh.clone()),
        ShapeType::Triangle(triangle) => Arc::new(*triangle),
        ShapeType::Compound(compound) => Arc::new(nc3::shape::Compound::new(
            compound
                .iter()
                .map(|(iso, shape)| {
                    (
                        *iso,
                        nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(shape)),
                    )
                })
                .collect::<Vec<_>>(),
        )),
    }
}

/// Objects that are of type MoveableObject can be moved every frame.
pub trait MoveableObject {
    /// Combines self's time of impact (toi) with another.
    ///
    /// Generally, it's advised to have self's toi be an option so that it can be determine whether
    /// or not a collision actually occurred just by observing the toi:
    /// ```rust
    /// fn combine_toi(&mut self, toi_other: f32) {
    ///     match self.nc3_toi {
    ///         None => self.nc3_toi = Some(toi_other),
    ///         Some(toi_current) => self.nc3_toi = Some(toi_current.min(toi_other)),
    ///     };
    /// }
    /// ```
    fn combine_toi(&mut self, toi_other: f32);

    /// The calculated time of impact or none if an impact has not ocurred
    fn time_of_impact(&self) -> Option<f32>;

    /// The position of the object.
    fn position(&self) -> nc3::na::Isometry3<f32>;

    /// The velocity of the object.
    fn velocity(&self) -> nc3::na::Vector3<f32>;

    /// Sets the position of the object.
    fn set_position(&mut self, position: nc3::na::Isometry3<f32>);

    /// Updates the position of the object based on the delta time and the object's toi
    fn update_position_for_frame(&mut self, time_delta: std::time::Duration) {
        let time_delta = {
            match self.time_of_impact() {
                None => time_delta.as_secs_f32(),
                Some(toi) => toi.min(time_delta.as_secs_f32()),
            }
        };

        let mut new_position = self.position();
        let translate = nc3::na::Translation3::<f32>::from(self.velocity() * time_delta);
        new_position.append_translation_mut(&translate);
        self.set_position(new_position);
    }
}

/// Objects that are of type CollisionObject can implement ways in which they collide with other
/// CollisionObject instances.
pub trait CollisionObject {
    /// Defines the Bevy-friendly shape object for collision detection.
    fn shape(&self) -> Arc<ShapeType>;

    /// Defines a ncollide3d shape handle which contains a ncollide3d shape.
    ///
    /// This is the shape object that is used in collision detection. Implementations of
    /// CollisionObject may wish to store the result of this function in a variable so that it
    /// does not need to be created on each iteration of the event loop.
    ///
    /// ```rust
    /// pub struct NewCollisionObject {
    ///     shape: Arc<ShapeType>,
    ///     nc3_shape_handle: Arc<nc3::shape::ShapeHandle<f32>>,
    ///     ...
    /// }
    ///
    /// impl NewCollisionObject {
    ///     pub fn new() -> Self {
    ///         let new_shape = Arc::new(ShapeType::Ball(Ball::<f32>::new(1.)));
    ///         NewCollisionObject {
    ///             shape: new_shape.clone(),
    ///             nc3_shape_handle: Arc::new(nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(
    ///                 &new_shape,
    ///             ))),
    ///             ...
    ///         }
    ///     }
    /// }
    ///
    /// impl CollisionObject for WalkingObject {
    ///     fn shape(&self) -> Arc<ShapeType> {
    ///         self.shape.clone()
    ///     }
    ///
    ///     fn nc3_shape_handle(&self) -> Arc<nc3::shape::ShapeHandle<f32>> {
    ///         // Optimize to reduce calls to nc3_shape_to_shape.
    ///         self.nc3_shape_handle.clone()
    ///     }
    ///
    ///     ...
    /// }
    /// ```
    fn nc3_shape_handle(&self) -> Arc<nc3::shape::ShapeHandle<f32>> {
        Arc::new(nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(
            &self.shape(),
        )))
    }

    /// The position of the shape.
    ///
    /// The default implimentation returns `nc3::na::Isometry3::<f32>::identity()`.
    fn nc3_position(&self) -> nc3::na::Isometry3<f32> {
        nc3::na::Isometry3::<f32>::identity()
    }

    /// The velocity of the shape.
    ///
    /// The default implimentation returns `nc3::na::Vector3::<f32>::zeros()`.
    fn nc3_velocity(&self) -> nc3::na::Vector3<f32> {
        nc3::na::Vector3::<f32>::zeros()
    }
}

/// An object that can walk along the terrain of the map.
#[derive(Clone, Component)]
pub struct WalkingObject {
    shape: Arc<ShapeType>,
    nc3_shape_handle: Arc<nc3::shape::ShapeHandle<f32>>,
    nc3_position: nc3::na::Isometry3<f32>,
    nc3_velocity: nc3::na::Vector3<f32>,
    nc3_toi: Option<f32>,
}

impl std::fmt::Debug for WalkingObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalkingObject")
            .field("shape", &self.shape)
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
            nc3_shape_handle: Arc::new(nc3::shape::ShapeHandle::from_arc(nc3_shape_to_shape(
                &shape,
            ))),
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

/// An object that moving objects cannot pass through.
#[derive(Debug, Component)]
pub struct ObsticalObject;

impl CollisionObject for ObsticalObject {
    fn shape(&self) -> Arc<ShapeType> {
        Arc::new(ShapeType::Capsule(nc3::shape::Capsule::<f32>::new(1., 1.)))
    }
}

/// A trait that defines how CollisionObject instances can interact with each other.
pub trait Collide<A: CollisionObject>: CollisionObject {
    /// Performs all necessary actions with two objects that collide
    fn collide_with(this: &mut Self, other: &mut A, collision: nc3::query::TOI<f32>);

    /// Determines if two objects collide or will collide.
    ///
    /// If two objects will collide, an estimation of when they will collide is provides.
    fn get_collision_with(&self, other: &A, max_toi: f32) -> Option<nc3::query::TOI<f32>> {
        nc3::query::time_of_impact(
            &nc3::query::DefaultTOIDispatcher,
            &self.nc3_position(),
            &self.nc3_velocity(),
            self.nc3_shape_handle().as_arc().as_ref(),
            &other.nc3_position(),
            &other.nc3_velocity(),
            other.nc3_shape_handle().as_arc().as_ref(),
            max_toi,
            0.0,
        )
        .unwrap_or_default()
    }
}

impl Collide<WalkingObject> for WalkingObject {
    fn collide_with(this: &mut Self, other: &mut WalkingObject, collision: nc3::query::TOI<f32>) {
        let (toi_self, toi_other) = {
            let v_self = this.nc3_velocity.magnitude();
            let v_other = other.nc3_velocity.magnitude();
            (
                collision.toi * v_self / v_other,
                collision.toi * v_other / v_self,
            )
        };
        self.combine_toi(toi_self);
        other.combine_toi(toi_other);
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
        println!("Collision: {:?}", collision);
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
            nc3::na::Vector3::<f32>::new(0., 0., 0.),
        );
        let collision = o1.get_collision_with(&o2, std::f32::MAX);
        println!("Collision: {:?}", collision);
        assert!(collision.is_some());
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
        println!("Collision: {:?}", collision);
        assert!(collision.is_none());
    }
}
