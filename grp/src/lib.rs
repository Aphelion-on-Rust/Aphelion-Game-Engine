//! Graphics Render Pipeline (GRP).
//!
//! - `frame`     turns a scene into a list of things to draw (`RenderFrame`), plain maths (step 7a)
//! - `software`  a renderer that draws a `RenderFrame` on the CPU into an image (step 7b)
//! - `backend`   the trait every renderer implements, so the GPU renderer (step 7d) can replace it
//!
//! The software renderer comes first because it works on every computer and its pictures can be
//! checked exactly in tests. Later the two pipelines SGP (Simple) and HGP (Heavy) build on this.

pub mod backend;
pub mod frame;
pub mod software;
