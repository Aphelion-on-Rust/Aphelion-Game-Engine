//! Graphics Render Pipeline (GRP).
//!
//! - `frame`     turns a scene into a list of things to draw (`RenderFrame`), plain maths (step 7a)
//! - `software`  a renderer that draws a `RenderFrame` on the CPU into an image (step 7b)
//! - `gizmos`    outlines of colliders and the like, as lines
//! - `pick`      finds the object under a point of the picture (click to select)
//! - `backend`   the trait every renderer implements, so the GPU renderer (step 7d) can replace it
//!
//! The software renderer comes first because it works on every computer and its pictures can be
//! checked exactly in tests. Later the two pipelines SGP (Simple) and HGP (Heavy) build on this.

/// The maths types, so other crates use exactly the version this crate was built with.
pub use glam;

pub mod backend;
pub mod frame;
pub mod gizmos;
pub mod pick;
pub mod software;
