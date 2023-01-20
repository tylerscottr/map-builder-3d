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

// use bevy::math::{f32::Vec2, Rect};
// use serde::{Deserialize, Serialize};

// const LINE_BOUNDS_EXPAND: f32 = 1e-6;

// #[derive(Debug, Deserialize, Serialize)]
// pub enum CollisionShape {
//     Rect { center: Vec2, size: Vec2 },
//     Circle { center: Vec2, radius: f32 },
//     SightLine { start: Vec2, length: f32 },
// }

// fn line_to_line_intersect_t(
//     start_a: Vec2,
//     end_a: Vec2,
//     start_b: Vec2,
//     end_b: Vec2,
// ) -> Option<(f32, f32)> {
//     // Test for quick fail
//     if (start_a.x.min(end_a.x) > start_b.x.max(end_b.x))
//         || (start_a.y.min(end_a.y) > start_b.y.max(end_b.y))
//         || (start_a.x.max(end_a.x) < start_b.x.min(end_b.x))
//         || (start_a.y.max(end_a.y) < start_b.y.min(end_b.y))
//     {
//         return None;
//     }

//     // Find position along line segments that intersect
//     let t = ((start_a.x - start_b.x) * (start_b.y - end_b.y)
//         - (start_a.y - start_b.y) * (start_b.x - end_b.x))
//         / ((start_a.x - end_a.x) * (start_b.y - end_b.y)
//             - (start_a.y - end_a.y) * (start_b.x - end_b.x));
//     let u = ((start_a.x - start_b.x) * (start_a.y - end_a.y)
//         - (start_a.y - start_b.y) * (start_a.x - end_a.x))
//         / ((start_a.x - end_a.x) * (start_b.y - end_b.y)
//             - (start_a.y - end_a.y) * (start_b.x - end_b.x));

//     // Check that the line segnents are not parallel
//     if t.is_infinite() || t.is_nan() || u.is_infinite() || u.is_nan() {
//         return None;
//     }

//     // // Check that the intersection lies on the line segnent
//     // if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
//     //     return None;
//     // }

//     // Return the location of the intersection
//     Some((t, u))
// }

// fn t_lies_on_line(start_a: Vec2, end_a: Vec2, t: f32) -> bool {
//     (0.0..=1.0).contains(&t)
// }

// fn point_on_line_at_t(start_a: Vec2, end_a: Vec2, t: f32) -> Vec2 {
//     start_a.lerp(end_a, t)
// }

// fn valid_point_on_line_at_t(start_a: Vec2, end_a: Vec2, t: f32) -> Option<Vec2> {
//     if !t_lies_on_line(start_a, end_a, t) {
//         return None;
//     }
//     Some(point_on_line_at_t(start_a, end_a, t))
// }

// fn valid_line_to_line_intersect(
//     start_a: Vec2,
//     end_a: Vec2,
//     start_b: Vec2,
//     end_b: Vec2,
// ) -> Option<Vec2> {
//     let t_u = line_to_line_intersect_t(start_a, end_a, start_b, end_b);
//     if let Some((t, u)) = t_u {
//         // t gets checked in valid_point_on_line_at_t, but u doesn't
//         if !t_lies_on_line(start_a, end_a, t) || !t_lies_on_line(start_b, end_b, u) {
//             return None;
//         }

//         return Some(point_on_line_at_t(start_a, end_a, t));
//     } else {
//         return None;
//     }
// }

// pub fn moving_line_to_stationary_line_collide(
//     start_a: Vec2,
//     end_a: Vec2,
//     movement_a: Vec2,
//     start_b: Vec2,
//     end_b: Vec2,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a = Rect::from_corners(start_a, end_a)
//         .inset(LINE_BOUNDS_EXPAND)
//         .union(
//             Rect::from_corners(start_a + movement_a, end_a + movement_a).inset(LINE_BOUNDS_EXPAND),
//         );
//     let rect_b = Rect::from_corners(start_b, end_b).inset(LINE_BOUNDS_EXPAND);
//     if rect_a.intersect(rect_b).is_empty() {
//         return None; // No collision
//     }

//     // If the two lines will collide, then the movement vector of any endpoint will collide with
//     // the other line. We must also check the reverse movement vector from the stationary line as
//     // well.
//     let intersection_tests = vec![
//         (start_a, start_a + movement_a, start_b, end_b, 1.),
//         (end_a, end_a + movement_a, start_b, end_b, 1.),
//         (start_b, start_b - movement_a, start_a, end_a, -1.),
//         (end_b, end_b - movement_a, start_a, end_a, -1.),
//     ];
//     let best_intersection = intersection_tests
//         .into_iter()
//         .map(|(s1, e1, s2, e2, mul)| {
//             // Find intersections and only keep information that we'll need later
//             valid_line_to_line_intersect(s1, e1, s2, e2)
//                 .map(|i| (i - s1, mul, (i - s1).length_squared()))
//         })
//         .flatten() // Remove invalid intersections
//         .min_by(|x, y| x.2.total_cmp(&y.2)) // Find intersection with shortest travel
//         .map(|(v, mul, _)| v * mul);
//     best_intersection
// }

// fn get_rect_lines(center: Vec2, size: Vec2, angle: f32) -> Vec<(Vec2, Vec2)> {
//     let rect = Rect::from_center_size(center, size);
//     let tl_nominal = rect.min;
//     let tr_nominal = Vec2::new(rect.max.x, rect.min.y);
//     let bl_nominal = Vec2::new(rect.min.x, rect.max.y);
//     let br_nominal = rect.max;

//     // Rotate the rect about the center
//     let tl_rotated = center + Vec2::from_angle(angle).rotate(tl_nominal - center);
//     let tr_rotated = center + Vec2::from_angle(angle).rotate(tr_nominal - center);
//     let bl_rotated = center + Vec2::from_angle(angle).rotate(bl_nominal - center);
//     let br_rotated = center + Vec2::from_angle(angle).rotate(br_nominal - center);

//     // Create and return lines from the corners
//     vec![
//         (tl_rotated, tr_rotated),
//         (tr_rotated, br_rotated),
//         (br_rotated, bl_rotated),
//         (bl_rotated, tl_rotated),
//     ]
// }

// pub fn moving_line_to_stationary_rect_collide(
//     start_a: Vec2,
//     end_a: Vec2,
//     movement_a: Vec2,
//     center_b: Vec2,
//     size_b: Vec2,
//     angle_b: f32,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a = Rect::from_corners(start_a, end_a)
//         .inset(LINE_BOUNDS_EXPAND)
//         .union(
//             Rect::from_corners(start_a + movement_a, end_a + movement_a).inset(LINE_BOUNDS_EXPAND),
//         );
//     let rect_b =
//         Rect::from_center_size(center_b, Vec2::splat(size_b.max_element() * 2.0_f32.sqrt()));

//     // Treat each side of the rect as individual line segments
//     let rect_lines = get_rect_lines(center_b, size_b, angle_b);

//     // Check collision of the line with each of the sides of the rect
//     let collisions = rect_lines
//         .into_iter()
//         .map(|(start_b, end_b)| {
//             moving_line_to_stationary_line_collide(start_a, end_a, movement_a, start_b, end_b)
//         })
//         .flatten();

//     // Only keep the collision with the shortest length
//     let best_collision = collisions
//         .map(|c| (c, c.length_squared()))
//         .min_by(|x, y| x.1.total_cmp(&y.1))
//         .map(|(c, _)| c);
//     best_collision
// }

// fn line_to_circle_intersects(start: Vec2, end: Vec2, center: Vec2, radius: f32) -> Vec<Vec2> {
//     // Based on algorithm from wolfram:
//     // https://mathworld.wolfram.com/Circle-LineIntersection.html
//     let delta = end - start;
//     let delta_flip = Vec2::new(delta.y, delta.x);
//     let delta_sign = Vec2::splat(if delta.y == 0. { 1. } else { delta.y.signum() });
//     let big_d = (start - center).perp_dot(end - center);
//     let base = Vec2::new(1., -1.) * big_d * delta_flip;
//     let discriminant = radius * radius * delta.length_squared() - big_d * big_d;

//     if discriminant < 0. {
//         // No intersections
//         return vec![];
//     }

//     let intersect1 =
//         (base + delta_sign * delta * discriminant.sqrt()) / delta.length_squared() + center;
//     if discriminant == 0. {
//         // Tangent line
//         return vec![intersect1];
//     }

//     // Secant line
//     let intersect2 =
//         (base - delta_sign * delta * discriminant.sqrt()) / delta.length_squared() + center;
//     vec![intersect1, intersect2]
// }

// pub fn moving_line_to_stationary_circle_collide(
//     start_a: Vec2,
//     end_a: Vec2,
//     movement_a: Vec2,
//     center_b: Vec2,
//     radius_b: f32,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a = Rect::from_corners(start_a, end_a)
//         .inset(LINE_BOUNDS_EXPAND)
//         .union(
//             Rect::from_corners(start_a + movement_a, end_a + movement_a).inset(LINE_BOUNDS_EXPAND),
//         );
//     let rect_b = Rect::from_center_half_size(center_b, Vec2::splat(radius_b));
//     if rect_a.intersect(rect_b).is_empty() {
//         return None; // No collision
//     }

//     // These are two scenarios of a line segment intersecting a circle:
//     //   1) Somewhere within the line will collide with the circle. In this case, the point that will
//     //      touch the circle will be where the line is tangent.
//     //   2) An endpoint will touch the circle. In this scenario, the collision happens where the
//     //      movement vector intersects the circle.

//     // Find the two points on the circle that has a tangent parallel to the line
//     let angle = (end_a - start_a).angle_between((1., 0.).into());
//     let tangent_pt1 = center_b + radius_b * Vec2::from_angle(std::f32::consts::FRAC_PI_2 + angle);
//     let tangent_pt2 = center_b
//         + radius_b * Vec2::from_angle(std::f32::consts::PI + std::f32::consts::FRAC_PI_2 + angle);

//     // Find where those tangent points would intersect the line segment if movement backwards
//     let mut intersections = vec![
//         valid_line_to_line_intersect(start_a, end_a, tangent_pt1, tangent_pt1 - movement_a)
//             .map(|i| (i, tangent_pt1)),
//         valid_line_to_line_intersect(start_a, end_a, tangent_pt2, tangent_pt2 - movement_a)
//             .map(|i| (i, tangent_pt2)),
//     ];

//     // Find where the end points will intersect with the circle when moving
//     intersections.extend(
//         line_to_circle_intersects(start_a, start_a + movement_a, center_b, radius_b)
//             .into_iter()
//             .map(|i| Some((start_a, i))),
//     );
//     intersections.extend(
//         line_to_circle_intersects(end_a, end_a + movement_a, center_b, radius_b)
//             .into_iter()
//             .map(|i| Some((end_a, i))),
//     );

//     // Find the first valid intersection
//     let best_intersection = intersections
//         .into_iter()
//         .flatten() // only keep valid intersections
//         .map(|(line_pt, tangent_pt)| (line_pt, tangent_pt, line_pt.distance_squared(tangent_pt)))
//         .min_by(|x, y| x.2.total_cmp(&y.2))
//         .map(|(line_pt, tangent_pt, _)| (line_pt, tangent_pt));

//     // Return the result if valid
//     best_intersection.map(|(line_pt, tangent_pt)| tangent_pt - line_pt)
// }

// pub fn moving_rect_to_stationary_rect_collide(
//     center_a: Vec2,
//     size_a: Vec2,
//     angle_a: f32,
//     movement_a: Vec2,
//     center_b: Vec2,
//     size_b: Vec2,
//     angle_b: f32,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a =
//         Rect::from_center_size(center_a, Vec2::splat(size_a.max_element() * 2.0_f32.sqrt())).union(
//             Rect::from_center_size(
//                 center_a + movement_a,
//                 Vec2::splat(size_a.max_element() * 2.0_f32.sqrt()),
//             ),
//         );
//     let rect_b =
//         Rect::from_center_size(center_b, Vec2::splat(size_b.max_element() * 2.0_f32.sqrt()));
//     if rect_a.intersect(rect_b).is_empty() {
//         return None; // No collision
//     }

//     // Treat each side of the rect as individual line segments
//     let rect_lines = get_rect_lines(center_a, size_a, angle_a);

//     // Check collision of each line with rect b
//     let collisions = rect_lines
//         .into_iter()
//         .map(|(start_a, end_a)| {
//             moving_line_to_stationary_rect_collide(
//                 start_a, end_a, movement_a, center_b, size_b, angle_b,
//             )
//         })
//         .flatten();

//     // Only keep the collision with the shortest length
//     let best_collision = collisions
//         .map(|c| (c, c.length_squared()))
//         .min_by(|x, y| x.1.total_cmp(&y.1))
//         .map(|(c, _)| c);
//     best_collision
// }

// pub fn moving_rect_to_stationary_circle_collide(
//     center_a: Vec2,
//     size_a: Vec2,
//     angle_a: f32,
//     movement_a: Vec2,
//     center_b: Vec2,
//     radius_b: f32,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a =
//         Rect::from_center_size(center_a, Vec2::splat(size_a.max_element() * 2.0_f32.sqrt())).union(
//             Rect::from_center_size(
//                 center_a + movement_a,
//                 Vec2::splat(size_a.max_element() * 2.0_f32.sqrt()),
//             ),
//         );
//     let rect_b = Rect::from_center_half_size(center_b, Vec2::splat(radius_b));
//     if rect_a.intersect(rect_b).is_empty() {
//         return None; // No collision
//     }

//     // Treat each side of the rect as individual line segments
//     let rect_lines = get_rect_lines(center_a, size_a, angle_a);

//     // Check collision of each line with rect b
//     let collisions = rect_lines
//         .into_iter()
//         .map(|(start_a, end_a)| {
//             moving_line_to_stationary_circle_collide(start_a, end_a, movement_a, center_b, radius_b)
//         })
//         .flatten();

//     // Only keep the collision with the shortest length
//     let best_collision = collisions
//         .map(|c| (c, c.length_squared()))
//         .min_by(|x, y| x.1.total_cmp(&y.1))
//         .map(|(c, _)| c);
//     best_collision
// }

// pub fn moving_circle_to_stationary_circle_collide(
//     center_a: Vec2,
//     radius_a: f32,
//     movement_a: Vec2,
//     center_b: Vec2,
//     radius_b: f32,
// ) -> Option<Vec2> {
//     // Test for quick fail if the two objects aren't even close
//     let rect_a = Rect::from_center_half_size(center_a, Vec2::splat(radius_a)).union(
//         Rect::from_center_half_size(center_a + movement_a, Vec2::splat(radius_a)),
//     );
//     let rect_b = Rect::from_center_half_size(center_b, Vec2::splat(radius_b));
//     if rect_a.intersect(rect_b).is_empty() {
//         return None; // No collision
//     }

//     // We'll use trigonometry to determine where along the movement vector circle a might encounter
//     // circle b. Imagine a triangle with the following sides:
//     //   Side 1: from circle a's initial center to circle b's center
//     //   Side 2: from circle a's initial center to its center at the time of collision
//     //   Side 3: from circle b's center to circle a's center at the time of collision
//     //
//     // We know that when the two circles meet (if they even collide) the distance between them will
//     // be radius_a + radius_b, so that gives us the length of one leg. We can also easily determine
//     // the angle between side 1 and side 2 since side 2 is coincident with the movement vector.
//     let a_to_b_initial = center_b - center_a;
//     let a_to_b_initial_length = a_to_b_initial.length();
//     let alpha = movement_a.angle_between(a_to_b_initial);
//     let radius_total = radius_a + radius_b;

//     // No we can use law of sines to determine the other angles
//     let beta1 = (a_to_b_initial_length / radius_total * alpha.sin()).asin();
//     if beta1.is_nan() || beta1.is_infinite() {
//         return None; // No collision is possible
//     }

//     // There is also a second valid value of beta, which is when circle a collide with the other
//     // side of circle b. This also makes sense considering that sin(theta) = sin(pi-theta), so an
//     // arcsine should technically give us two results.
//     let beta2 = std::f32::consts::PI - beta1;
//     let beta = vec![beta1, beta2];

//     // For each beta result, we want to calculate the point of collision. We'll then use nearest
//     // point as the result. Law of sines will give us the vectors for the two possible collisions.
//     let gamma = beta.iter().map(|b| (b, 2. * std::f32::consts::PI - b));
//     let possible_collisions = gamma.map(|(b, g)| a_to_b_initial_length / b.sin() * g.sin());
//     let collision = possible_collisions.min_by(|x, y| x.total_cmp(&y));

//     if let Some(valid_collision) = collision {
//         // Make sure we're actually restricting the movement
//         let movement_length = movement_a.length();
//         if valid_collision > movement_length {
//             return None;
//         }
//         let movement_to_collision = valid_collision * movement_a / movement_length;

//         // Double check we didn't have a near-zero movement length
//         if !movement_to_collision.is_finite() {
//             return None;
//         }

//         // Return the restricted movement
//         Some(movement_to_collision)
//     } else {
//         None
//     }
// }

// pub fn check_for_movement_collision(
//     shape_a: CollisionShape,
//     angle_a: f32,
//     movement_a: Vec2,
//     shape_b: CollisionShape,
//     angle_b: f32,
// ) -> Option<Vec2> {
//     // TODO
//     return None;
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const MAX_ERROR: f32 = 1e-6;

//     fn vec2_is_close(v1: &Vec2, v2: &Vec2, max_error: f32) -> bool {
//         let max_error_sq = max_error * max_error;
//         v1.distance_squared(*v2) <= max_error_sq
//     }

//     fn vec_of_vec2_is_close(vec1: &Vec<Vec2>, vec2: &Vec<Vec2>, max_error: f32) -> bool {
//         if vec1.len() != vec2.len() {
//             return false;
//         }

//         fn check_vec(vec1: &Vec<Vec2>, vec2: &Vec<Vec2>, max_error: f32) -> bool {
//             let max_error_sq = max_error * max_error;
//             vec1.iter().all(|&v1| {
//                 if let Some(nearest) = vec2
//                     .iter()
//                     .map(|v2| v1.distance_squared(*v2))
//                     .min_by(|x, y| x.total_cmp(&y))
//                 {
//                     nearest <= max_error_sq
//                 } else {
//                     false
//                 }
//             })
//         }

//         check_vec(vec1, vec2, max_error) && check_vec(vec2, vec1, max_error)
//     }

//     fn do_test_line_segment_intersect(
//         start_a: Vec2,
//         end_a: Vec2,
//         start_b: Vec2,
//         end_b: Vec2,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let intersect = valid_line_to_line_intersect(start_a, end_a, start_b, end_b);
//         println!(
//             "Test line-line intersection: ({:?}, {:?}) x ({:?}, {:?}) => {:?} vs expected {:?}",
//             start_a, end_a, start_b, end_b, intersect, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (intersect.is_some() == exp.is_some())
//                 && if intersect.is_some() && exp.is_some() {
//                     vec2_is_close(&intersect.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_line_segment_intersect() {
//         do_test_line_segment_intersect(
//             (0., 0.).into(),
//             (10., 10.).into(),
//             (10., 0.).into(),
//             (0., 10.).into(),
//             vec![Some((5., 5.).into())],
//         );

//         do_test_line_segment_intersect(
//             (0., 0.).into(),
//             (10., 0.).into(),
//             (0., 0.1).into(),
//             (10., 0.1).into(),
//             vec![None],
//         );

//         do_test_line_segment_intersect(
//             (0., 0.).into(),
//             (10., 0.).into(),
//             (0., 0.0).into(),
//             (10., 0.0).into(),
//             vec![None],
//         );

//         do_test_line_segment_intersect(
//             (0., 0.).into(),
//             (10., 0.).into(),
//             (0., 0.0).into(),
//             (0., 10.0).into(),
//             vec![None, Some((0., 0.).into())],
//         );
//     }

//     fn do_test_line_to_circle_intersect(
//         start_a: Vec2,
//         end_a: Vec2,
//         center_b: Vec2,
//         radius_b: f32,
//         expected: Vec<Vec2>,
//     ) {
//         let intersects = line_to_circle_intersects(start_a, end_a, center_b, radius_b);
//         println!(
//             "Test line-circle intersection: ({:?}, {:?}) x ({:?}, {:?}) => {:?} vs expected {:?}",
//             start_a, end_a, center_b, radius_b, intersects, expected
//         );
//         assert!(vec_of_vec2_is_close(&intersects, &expected, MAX_ERROR));
//     }

//     #[test]
//     fn test_line_to_circle_intersect() {
//         do_test_line_to_circle_intersect(
//             (-10., 0.).into(),
//             (10., 0.).into(),
//             (0., 0.).into(),
//             1.,
//             vec![(-1., 0.).into(), (1., 0.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (-10., 1.).into(),
//             (10., 1.).into(),
//             (0., 0.).into(),
//             1.,
//             vec![(0., 1.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (-10., 0.5).into(),
//             (10., 0.5).into(),
//             (0., 0.).into(),
//             1.,
//             vec![
//                 (-(std::f32::consts::FRAC_PI_6).cos(), 0.5).into(),
//                 ((std::f32::consts::FRAC_PI_6).cos(), 0.5).into(),
//             ],
//         );

//         do_test_line_to_circle_intersect(
//             (-10., 5.).into(),
//             (10., 5.).into(),
//             (0., 5.).into(),
//             1.,
//             vec![(-1., 5.).into(), (1., 5.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (-10., 6.).into(),
//             (10., 6.).into(),
//             (0., 5.).into(),
//             1.,
//             vec![(0., 6.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (-10., 5.5).into(),
//             (10., 5.5).into(),
//             (0., 5.).into(),
//             1.,
//             vec![
//                 (-(std::f32::consts::FRAC_PI_6).cos(), 5.5).into(),
//                 ((std::f32::consts::FRAC_PI_6).cos(), 5.5).into(),
//             ],
//         );

//         do_test_line_to_circle_intersect(
//             (5., -10.).into(),
//             (5., 10.).into(),
//             (5., 0.).into(),
//             1.,
//             vec![(5., -1.).into(), (5., 1.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (6., -10.).into(),
//             (6., 10.).into(),
//             (5., 0.).into(),
//             1.,
//             vec![(6., 0.).into()],
//         );

//         do_test_line_to_circle_intersect(
//             (5.5, -10.).into(),
//             (5.5, 10.).into(),
//             (5., 0.).into(),
//             1.,
//             vec![
//                 (5.5, -(std::f32::consts::FRAC_PI_6).cos()).into(),
//                 (5.5, (std::f32::consts::FRAC_PI_6).cos()).into(),
//             ],
//         );
//     }

//     fn do_test_line_to_line_collision(
//         start_a: Vec2,
//         end_a: Vec2,
//         movement_a: Vec2,
//         start_b: Vec2,
//         end_b: Vec2,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement =
//             moving_line_to_stationary_line_collide(start_a, end_a, movement_a, start_b, end_b);
//         println!(
//             "Test line-line collision: ({:?}, {:?}) + {:?} x ({:?}, {:?}) => {:?} vs expected {:?}",
//             start_a, end_a, movement_a, start_b, end_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_line_to_line_collision() {
//         do_test_line_to_line_collision(
//             (0., 0.).into(),             // start_a
//             (0., 10.).into(),            // end_a
//             (10., 0.).into(),            // movement
//             (5., 4.).into(),             // start_b
//             (6., 6.).into(),             // end_b
//             vec![Some((5., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 0.).into(),             // start_a
//             (0., 10.).into(),            // end_a
//             (10., 0.).into(),            // movement
//             (6., 4.).into(),             // start_b
//             (5., 6.).into(),             // end_b
//             vec![Some((5., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 0.).into(),                    // start_a
//             (0., 10.).into(),                   // end_a
//             (10., 0.).into(),                   // movement
//             (10., 4.).into(),                   // start_b
//             (10., 6.).into(),                   // end_b
//             vec![None, Some((10., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 0.).into(),  // start_a
//             (0., 10.).into(), // end_a
//             (10., 0.).into(), // movement
//             (20., 4.).into(), // start_b
//             (20., 6.).into(), // end_b
//             vec![None],       // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 4.).into(),             // start_a
//             (1., 6.).into(),             // end_a
//             (10., 0.).into(),            // movement
//             (5., 0.).into(),             // start_b
//             (5., 10.).into(),            // end_b
//             vec![Some((4., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (1., 4.).into(),             // start_a
//             (0., 6.).into(),             // end_a
//             (10., 0.).into(),            // movement
//             (5., 0.).into(),             // start_b
//             (5., 10.).into(),            // end_b
//             vec![Some((4., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 4.).into(),                    // start_a
//             (0., 6.).into(),                    // end_a
//             (10., 0.).into(),                   // movement
//             (10., 0.).into(),                   // start_b
//             (10., 10.).into(),                  // end_b
//             vec![None, Some((10., 0.).into())], // expected result
//         );

//         do_test_line_to_line_collision(
//             (0., 4.).into(),   // start_a
//             (0., 6.).into(),   // end_a
//             (10., 0.).into(),  // movement
//             (20., 0.).into(),  // start_b
//             (20., 10.).into(), // end_b
//             vec![None],        // expected result
//         );
//     }

//     fn do_test_line_to_rect_collision(
//         start_a: Vec2,
//         end_a: Vec2,
//         movement_a: Vec2,
//         center_b: Vec2,
//         size_b: Vec2,
//         angle_b: f32,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement = moving_line_to_stationary_rect_collide(
//             start_a, end_a, movement_a, center_b, size_b, angle_b,
//         );
//         println!(
//             "Test line-rect collision: ({:?}, {:?}) + {:?} x ({:?}, {:?} rot {:?}) => {:?} vs expected {:?}",
//             start_a, end_a, movement_a, center_b, size_b, angle_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_line_to_rect_collision() {
//         do_test_line_to_rect_collision(
//             (0., 0.).into(),             // start_a
//             (0., 10.).into(),            // end_a
//             (10., 0.).into(),            // movement
//             (5., 5.).into(),             // center_b
//             (2., 2.).into(),             // size_b
//             0.,                          // angle_b
//             vec![Some((4., 0.).into())], // expected result
//         );

//         do_test_line_to_rect_collision(
//             (0., 0.).into(),                                   // start_a
//             (0., 10.).into(),                                  // end_a
//             (10., 0.).into(),                                  // movement
//             (5., 5.).into(),                                   // center_b
//             (2., 2.).into(),                                   // size_b
//             std::f32::consts::FRAC_PI_4,                       // angle_b
//             vec![Some((5. - 1. * 2.0_f32.sqrt(), 0.).into())], // expected result
//         );

//         do_test_line_to_rect_collision(
//             (0., 0.).into(),                   // start_a
//             (0., 10.).into(),                  // end_a
//             (4., 0.).into(),                   // movement
//             (5., 5.).into(),                   // center_b
//             (2., 2.).into(),                   // size_b
//             0.,                                // angle_b
//             vec![None, Some((4., 0.).into())], // expected result
//         );

//         do_test_line_to_rect_collision(
//             (0., 0.).into(),                                         // start_a
//             (0., 10.).into(),                                        // end_a
//             (5. - 1. * 2.0_f32.sqrt(), 0.).into(),                   // movement
//             (5., 5.).into(),                                         // center_b
//             (2., 2.).into(),                                         // size_b
//             std::f32::consts::FRAC_PI_4,                             // angle_b
//             vec![None, Some((5. - 1. * 2.0_f32.sqrt(), 0.).into())], // expected result
//         );

//         do_test_line_to_rect_collision(
//             (0., 0.).into(),  // start_a
//             (0., 10.).into(), // end_a
//             (1., 0.).into(),  // movement
//             (5., 5.).into(),  // center_b
//             (2., 2.).into(),  // size_b
//             0.,               // angle_b
//             vec![None],       // expected result
//         );

//         do_test_line_to_rect_collision(
//             (0., 0.).into(),             // start_a
//             (0., 10.).into(),            // end_a
//             (1., 0.).into(),             // movement
//             (5., 5.).into(),             // center_b
//             (2., 2.).into(),             // size_b
//             std::f32::consts::FRAC_PI_4, // angle_b
//             vec![None],                  // expected result
//         );
//     }

//     fn do_test_line_to_circle_collision(
//         start_a: Vec2,
//         end_a: Vec2,
//         movement_a: Vec2,
//         center_b: Vec2,
//         radius_b: f32,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement = moving_line_to_stationary_circle_collide(
//             start_a, end_a, movement_a, center_b, radius_b,
//         );
//         println!(
//             "Test line-circle collision: ({:?}, {:?}) + {:?} x ({:?}, {:?}) => {:?} vs expected {:?}",
//             start_a, end_a, movement_a, center_b, radius_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_line_to_circle_collision() {
//         do_test_line_to_circle_collision(
//             (-10., 1.).into(),           // line start
//             (10., 1.).into(),            // line end
//             (0., -1.).into(),            // movement
//             (0., 0.).into(),             // center
//             1.,                          // radius
//             vec![Some((0., 0.).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (-10., 1.5).into(),            // line start
//             (10., 1.5).into(),             // line end
//             (0., -1.).into(),              // movement
//             (0., 0.).into(),               // center
//             1.,                            // radius
//             vec![Some((0., -0.5).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (-10., 2.).into(),                  // line start
//             (10., 2.).into(),                   // line end
//             (0., -1.).into(),                   // movement
//             (0., 0.).into(),                    // center
//             1.,                                 // radius
//             vec![None, Some((0., -1.).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (-10., 3.).into(), // line start
//             (10., 3.).into(),  // line end
//             (0., -1.).into(),  // movement
//             (0., 0.).into(),   // center
//             1.,                // radius
//             vec![None],        // expected result
//         );

//         do_test_line_to_circle_collision(
//             (1., -10.).into(),           // line start
//             (1., 10.).into(),            // line end
//             (-1., 0.).into(),            // movement
//             (0., 0.).into(),             // center
//             1.,                          // radius
//             vec![Some((0., 0.).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (1.5, -10.).into(),            // line start
//             (1.5, 10.).into(),             // line end
//             (-1., 0.).into(),              // movement
//             (0., 0.).into(),               // center
//             1.,                            // radius
//             vec![Some((-0.5, 0.).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (2., -10.).into(),                  // line start
//             (2., 10.).into(),                   // line end
//             (-1., 0.).into(),                   // movement
//             (0., 0.).into(),                    // center
//             1.,                                 // radius
//             vec![None, Some((-1., 0.).into())], // expected result
//         );

//         do_test_line_to_circle_collision(
//             (3., -10.).into(), // line start
//             (3., 10.).into(),  // line end
//             (-1., 0.).into(),  // movement
//             (0., 0.).into(),   // center
//             1.,                // radius
//             vec![None],        // expected result
//         );
//     }

//     fn do_test_rect_to_rect_collision(
//         center_a: Vec2,
//         size_a: Vec2,
//         angle_a: f32,
//         movement_a: Vec2,
//         center_b: Vec2,
//         size_b: Vec2,
//         angle_b: f32,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement = moving_rect_to_stationary_rect_collide(
//             center_a, size_a, angle_a, movement_a, center_b, size_b, angle_b,
//         );
//         println!(
//             "Test rect-rect collision: ({:?}, {:?} rot {:?}) + {:?} x ({:?}, {:?} rot {:?}) => {:?} vs expected {:?}",
//             center_a, size_a, angle_a, movement_a, center_b, size_b, angle_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_rect_to_rect_collision() {
//         do_test_rect_to_rect_collision(
//             (0., 0.).into(),              // center_a
//             (1., 1.).into(),              // size_a
//             0.,                           // angle_a
//             (10., 0.).into(),             // movement
//             (5., 0.).into(),              // center_b
//             (2., 2.).into(),              // size_b
//             0.,                           // angle_b
//             vec![Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_rect_collision(
//             (0., 0.).into(),                    // center_a
//             (1., 1.).into(),                    // size_a
//             0.,                                 // angle_a
//             (3.5, 0.).into(),                   // movement
//             (5., 0.).into(),                    // center_b
//             (2., 2.).into(),                    // size_b
//             0.,                                 // angle_b
//             vec![None, Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_rect_collision(
//             (0., 0.).into(), // center_a
//             (1., 1.).into(), // size_a
//             0.,              // angle_a
//             (1., 0.).into(), // movement
//             (5., 0.).into(), // center_b
//             (2., 2.).into(), // size_b
//             0.,              // angle_b
//             vec![None],      // expected result
//         );

//         do_test_rect_to_rect_collision(
//             (0., 0.).into(),              // center_a
//             (2., 2.).into(),              // size_a
//             0.,                           // angle_a
//             (10., 0.).into(),             // movement
//             (5., 0.).into(),              // center_b
//             (1., 1.).into(),              // size_b
//             0.,                           // angle_b
//             vec![Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_rect_collision(
//             (0., 0.).into(),                    // center_a
//             (2., 2.).into(),                    // size_a
//             0.,                                 // angle_a
//             (3.5, 0.).into(),                   // movement
//             (5., 0.).into(),                    // center_b
//             (1., 1.).into(),                    // size_b
//             0.,                                 // angle_b
//             vec![None, Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_rect_collision(
//             (0., 0.).into(), // center_a
//             (2., 2.).into(), // size_a
//             0.,              // angle_a
//             (1., 0.).into(), // movement
//             (5., 0.).into(), // center_b
//             (1., 1.).into(), // size_b
//             0.,              // angle_b
//             vec![None],      // expected result
//         );
//     }

//     fn do_test_rect_to_circle_collision(
//         center_a: Vec2,
//         size_a: Vec2,
//         angle_a: f32,
//         movement_a: Vec2,
//         center_b: Vec2,
//         radius_b: f32,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement = moving_rect_to_stationary_circle_collide(
//             center_a, size_a, angle_a, movement_a, center_b, radius_b,
//         );
//         println!(
//             "Test rect-circle collision: ({:?}, {:?} rot {:?}) + {:?} x ({:?}, {:?}) => {:?} vs expected {:?}",
//             center_a, size_a, angle_a, movement_a, center_b, radius_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     #[test]
//     fn test_rect_to_circle_collision() {
//         do_test_rect_to_circle_collision(
//             (0., 0.).into(),              // center_a
//             (1., 1.).into(),              // size_a
//             0.,                           // angle_a
//             (10., 0.).into(),             // movement
//             (5., 0.).into(),              // center_b
//             1.,                           // radius_b
//             vec![Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_circle_collision(
//             (0., 0.).into(),                    // center_a
//             (1., 1.).into(),                    // size_a
//             0.,                                 // angle_a
//             (3.5, 0.).into(),                   // movement
//             (5., 0.).into(),                    // center_b
//             1.,                                 // radius_b
//             vec![None, Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_circle_collision(
//             (0., 0.).into(), // center_a
//             (1., 1.).into(), // size_a
//             0.,              // angle_a
//             (1., 0.).into(), // movement
//             (5., 0.).into(), // center_b
//             1.,              // radius_b
//             vec![None],      // expected result
//         );

//         do_test_rect_to_circle_collision(
//             (0., 0.).into(),              // center_a
//             (2., 2.).into(),              // size_a
//             0.,                           // angle_a
//             (10., 0.).into(),             // movement
//             (5., 0.).into(),              // center_b
//             0.5,                          // radius_b
//             vec![Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_circle_collision(
//             (0., 0.).into(),                    // center_a
//             (2., 2.).into(),                    // size_a
//             0.,                                 // angle_a
//             (3.5, 0.).into(),                   // movement
//             (5., 0.).into(),                    // center_b
//             0.5,                                // radius_b
//             vec![None, Some((3.5, 0.).into())], // expected result
//         );

//         do_test_rect_to_circle_collision(
//             (0., 0.).into(), // center_a
//             (2., 2.).into(), // size_a
//             0.,              // angle_a
//             (1., 0.).into(), // movement
//             (5., 0.).into(), // center_b
//             0.5,             // radius_b
//             vec![None],      // expected result
//         );
//     }

//     fn do_test_circle_to_circle_collision(
//         center_a: Vec2,
//         radius_a: f32,
//         movement_a: Vec2,
//         center_b: Vec2,
//         radius_b: f32,
//         expected: Vec<Option<Vec2>>,
//     ) {
//         let accepted_movement = moving_circle_to_stationary_circle_collide(
//             center_a, radius_a, movement_a, center_b, radius_b,
//         );
//         println!(
//             "Test circle-circle collision: ({:?}, {:?}) + {:?} x ({:?}, {:?}) => {:?} vs expected {:?}",
//             center_a, radius_a, movement_a, center_b, radius_b, accepted_movement, expected
//         );
//         assert!(expected
//             .into_iter()
//             .any(|exp| (accepted_movement.is_some() == exp.is_some())
//                 && if accepted_movement.is_some() && exp.is_some() {
//                     vec2_is_close(&accepted_movement.unwrap(), &exp.unwrap(), MAX_ERROR)
//                 } else {
//                     true
//                 }));
//     }

//     fn bench_test_circle_to_circle_collision(
//         center_a: Vec2,
//         radius_a: f32,
//         movement_a: Vec2,
//         center_b: Vec2,
//         radius_b: f32,
//     ) {
//         let start_time = std::time::Instant::now();
//         let mut count = 0;
//         loop {
//             let _accepted_movement = moving_circle_to_stationary_circle_collide(
//                 center_a, radius_a, movement_a, center_b, radius_b,
//             );

//             count = count + 1;
//             if start_time.elapsed().as_millis() > 1000 {
//                 break;
//             }
//         }
//         println!(
//             "Benchmark circle-circle collision: {:?} iterations in {:?} ms, average {:?} ms per iteration",
//             count,
//             start_time.elapsed().as_millis(),
//             start_time.elapsed().as_millis() as f32 / count as f32,
//         );
//     }

//     #[test]
//     fn test_circle_to_circle_collision() {
//         do_test_circle_to_circle_collision(
//             (-10., 0.).into(), // center_a
//             1.,                // radius_a
//             (1., 0.).into(),   // movement
//             (10., 0.).into(),  // center_b
//             1.,                // radius_b
//             vec![None],        // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (-10., 0.).into(),                  // center_a
//             1.,                                 // radius_a
//             (20., 0.).into(),                   // movement
//             (10., 0.).into(),                   // center_b
//             1.,                                 // radius_b
//             vec![None, Some((20., 0.).into())], // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (-10., 0.).into(),            // center_a
//             1.,                           // radius_a
//             (30., 0.).into(),             // movement
//             (10., 0.).into(),             // center_b
//             1.,                           // radius_b
//             vec![Some((20., 0.).into())], // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (10., 0.).into(),              // center_a
//             1.,                            // radius_a
//             (-30., 0.).into(),             // movement
//             (-10., 0.).into(),             // center_b
//             1.,                            // radius_b
//             vec![Some((-20., 0.).into())], // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (0., -10.).into(), // center_a
//             1.,                // radius_a
//             (0., 1.).into(),   // movement
//             (0., 10.).into(),  // center_b
//             1.,                // radius_b
//             vec![None],        // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (0., -10.).into(),                  // center_a
//             1.,                                 // radius_a
//             (0., 20.).into(),                   // movement
//             (0., 10.).into(),                   // center_b
//             1.,                                 // radius_b
//             vec![None, Some((0., 20.).into())], // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (0., -10.).into(),            // center_a
//             1.,                           // radius_a
//             (0., 30.).into(),             // movement
//             (0., 10.).into(),             // center_b
//             1.,                           // radius_b
//             vec![Some((0., 20.).into())], // expected result
//         );

//         do_test_circle_to_circle_collision(
//             (0., 10.).into(),              // center_a
//             1.,                            // radius_a
//             (0., -30.).into(),             // movement
//             (0., -10.).into(),             // center_b
//             1.,                            // radius_b
//             vec![Some((0., -20.).into())], // expected result
//         );

//         bench_test_circle_to_circle_collision(
//             (0., 10.).into(),  // center_a
//             1.,                // radius_a
//             (0., -30.).into(), // movement
//             (0., -10.).into(), // center_b
//             1.,                // radius_b
//         );
//     }
// }
