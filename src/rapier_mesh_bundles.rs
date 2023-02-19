use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// A struct that contains a rapier collider and as well as a mesh handle.
///
/// Having them grouped together like this allows us to create both at the same time since we
/// generally would like to have the collider be the same shape as the mesh. However, this will not
/// be the case for complex meshes like characters and buildings.
#[derive(Bundle, Clone)]
pub struct RapierShapeBundle {
    /// A Rapier collider struct
    pub collider: Collider,
    /// A Bevy mesh handle
    pub mesh: Handle<Mesh>,
}

impl Default for RapierShapeBundle {
    fn default() -> Self {
        Self {
            collider: Collider::ball(0.0), // Massless object
            mesh: Default::default(),      // No mesh
        }
    }
}

impl RapierShapeBundle {
    /// Creates a collider and a mesh for a plane in the XZ plane.
    pub fn plane(half_size: Vec2, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        RapierShapeBundle {
            collider: Collider::heightfield(
                vec![0., 0., 0., 0.],
                2,
                2,
                Vec3::new(2. * half_size.x, 1., 2. * half_size.y),
            ),
            mesh: meshes.add(Mesh::from(shape::Box::new(
                2. * half_size.x,
                0.0,
                2. * half_size.y,
            ))),
        }
    }

    /// Creates a collider and a mesh for a box.
    pub fn cuboid(half_size: Vec3, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        RapierShapeBundle {
            collider: Collider::cuboid(half_size.x, half_size.y, half_size.z),
            mesh: meshes.add(Mesh::from(shape::Box::new(
                2. * half_size.x,
                2. * half_size.y,
                2. * half_size.z,
            ))),
        }
    }

    /// Creates a collider and a mesh for a sphere.
    pub fn sphere(radius: f32, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        RapierShapeBundle {
            collider: Collider::ball(radius),
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius,
                ..default()
            })),
        }
    }

    /// Creates a collider and a mesh for a capsule that stands tall in the Y direction.
    ///
    /// Note: half_length describes half the length between the two hemispheres of the capsule.
    pub fn capsule(half_length: f32, radius: f32, meshes: &mut ResMut<Assets<Mesh>>) -> Self {
        RapierShapeBundle {
            collider: Collider::capsule(
                Vec3::new(0., -half_length, 0.),
                Vec3::new(0., half_length, 0.),
                radius,
            ),
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius,
                depth: half_length * 2.,
                ..default()
            })),
        }
    }
}

/// A component bundle for rapier entities with a [`Collider`], [`Mesh`] and a [`StandardMaterial`].
pub type RapierColliderPbrBundle = RapierColliderMaterialMeshBundle<StandardMaterial>;

/// A component bundle for rapier entities with a [`Collider`], [`Mesh`] and a [`Material`].
#[derive(Bundle, Clone)]
pub struct RapierColliderMaterialMeshBundle<M: Material> {
    /// A bundle containing the collider and the mesh handle.
    pub shape: RapierShapeBundle,
    /// The material assigned to the mesh.
    pub material: Handle<M>,
    /// The transform applied to both the collider and the mesh.
    pub transform: Transform,
    /// The global transform (ncessary to make the transform work).
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible.
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering.
    pub computed_visibility: ComputedVisibility,
}

impl<M: Material> Default for RapierColliderMaterialMeshBundle<M> {
    fn default() -> Self {
        Self {
            shape: RapierShapeBundle::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
