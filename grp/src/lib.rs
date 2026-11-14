//! Graphics Render Pipeline (GRP).
//!
//! Step 7a (this): `frame` turns a scene into a list of things to draw (`RenderFrame`): the camera
//! matrices, meshes, sprites and lights with their world matrices. It is plain maths, no GPU.
//! Later steps draw a `RenderFrame` (a software renderer first, then a GPU one) and add the two
//! pipelines, SGP (Simple) and HGP (Heavy).

pub mod frame;
