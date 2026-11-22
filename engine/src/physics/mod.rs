//! Physics. Step 8a is the geometry: where the colliders are in the world, and what a ray hits.
//! Moving bodies (gravity, collisions) comes next.
//!
//! - world.rs    every collider of a scene with its world position, rotation and scaled size
//! - raycast.rs  shooting a ray at the colliders

pub mod raycast;
pub mod world;

pub use raycast::{raycast, RayHit};
pub use world::{extract_colliders, WorldCollider};
