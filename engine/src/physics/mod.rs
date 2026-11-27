//! Physics. 8a: where the colliders are in the world, and what a ray hits. 8b: bodies that move and collide.
//!
//! - world.rs    every collider of a scene with its world position, rotation and scaled size
//! - raycast.rs  shooting a ray at the colliders
//! - convex.rs   a collider as a solid that can say which of its points is farthest in a direction
//! - gjk.rs      do two solids overlap, how deep, and which way to push them apart
//! - sim.rs      the physics world: gravity, collisions, friction, bounce, triggers and events

pub mod convex;
pub mod gjk;
pub mod raycast;
pub mod sim;
pub mod world;

pub use raycast::{raycast, RayHit};
pub use sim::{BodyParams, PhysicsEvent, PhysicsEventKind, PhysicsSettings, PhysicsWorld};
pub use world::{extract_colliders, WorldCollider};
