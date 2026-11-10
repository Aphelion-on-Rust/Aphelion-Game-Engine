//! Resource garbage collection.
//!
//! Textures, meshes, materials, audio ... are loaded once and shared. Users of a resource
//! `retain` it and later `release` it. A resource nobody uses for a while (the grace period)
//! gets collected, and the owner of the real memory (renderer, audio) is told to free it.
//!
//! This is the engine's own GC. Memory used by C# scripts is cleaned by the .NET GC.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::GcController;
pub use interface::{CollectedResource, GcData, ResourceHandle, ResourceKind};
pub use model::GcModel;
