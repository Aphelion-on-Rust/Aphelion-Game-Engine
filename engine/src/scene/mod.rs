//! Scenes: a list of GameObjects, each with components.
//!
//! - interface.rs  data shapes (GameObject, components, Scene)
//! - model.rs      stores a scene
//! - query.rs      read-only lookups (safe for views to use)
//! - controller.rs create, parent, add components, destroy
//! - presets.rs    ready-made objects (cube, camera, light ...)
//! - copy.rs       duplicating a branch with brand new ids (used by prefabs)
//!
//! There is no view.rs yet, the editor will add the hierarchy view.

pub(crate) mod copy;
pub mod controller;
pub mod interface;
pub mod model;
pub mod presets;
pub mod query;

pub use controller::SceneController;
pub use interface::{
    ComponentData, ComponentEntry, GameObjectData, RectTransformData, SceneData, ScriptData,
    TransformData,
};
pub use model::SceneModel;
pub use presets::ObjectPreset;
