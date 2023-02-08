//! A crate for creating a 3d world in Bevy.
//!
//! The crate is composed of the following modules:
//! - Collision detection: Uses ncollide3d in a Bevy-friendly way so as to allow objects with
//! ncollide3d shapes to be assets.
//! - Map: A collection of 3D tiles, obsticals, players, event spaces, and other objects.

#![deny(missing_docs)]
// #![forbid(missing_docs_in_private_items)]

extern crate ncollide3d as nc3;

/// A module that determines which objects collide with each other
pub mod collision;
