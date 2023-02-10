//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - [collision]: Uses ncollide3d in a Bevy-friendly way so as to allow objects with
//! ncollide3d shapes to be assets.
//! - map: A collection of 3D tiles, obstacles, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

pub extern crate ncollide3d as nc3;

/// A module that determines which objects collide with each other.
pub mod collision;

/// A module for creating and interacting with walking objects.
pub mod collision_walking;

/// A module for creating and interacting with obstacles.
pub mod collision_obstacle;

/// A module that handles object collisions in the event loop.
pub mod collision_system;
