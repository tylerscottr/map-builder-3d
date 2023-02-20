//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - map: A collection of 3D tiles, obstacles, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

/// A module that integrates the adds some useful functions to the Rapier physics engine.
pub mod rapier_mesh_bundles;

/// A module that adds mouse/keyboard control to the camera.
pub mod controller;
