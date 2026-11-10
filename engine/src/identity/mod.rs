//! Ids for everything in a project.
//!
//! - AssetUuid:   never changes, never repeats (scripts, prefabs, scenes, instances).
//! - ComponentId: whole number that starts at 10000 and counts up.
//!
//! Folder pattern used across the engine: interface.rs, model.rs, controller.rs.

pub mod controller;
pub mod interface;
pub mod model;

pub use controller::IdController;
pub use interface::{AssetUuid, ComponentId, IdRegistryData, FIRST_COMPONENT_ID};
pub use model::IdRegistryModel;
