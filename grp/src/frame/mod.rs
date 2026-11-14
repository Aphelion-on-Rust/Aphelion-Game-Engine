//! From a scene to a list of things to draw.
//!
//! - interface.rs  the draw list (`RenderFrame`) and its items
//! - transform.rs  local and world matrices of objects
//! - extract.rs    choosing the camera and collecting meshes, sprites and lights
//!
//! The coordinate system is left-handed with Y up and Z forward, and rotations are Euler angles in
//! degrees applied in the order Z, then X, then Y (the same convention as Unity).

pub mod extract;
pub mod interface;
pub mod transform;

pub use extract::{camera_from_world, collect, fallback_camera, find_game_camera, projection_matrix};
pub use interface::{RenderCamera, RenderFrame, RenderLight, RenderMesh, RenderSprite};
pub use transform::{local_matrix, world_matrix};
