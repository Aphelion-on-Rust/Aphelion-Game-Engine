//! Prefabs: a saved branch of objects that can be placed in scenes again and again.
//! Every placed copy gets brand new uuids and component ids.

pub mod controller;
pub mod interface;
pub mod model;

pub use controller::PrefabController;
pub use interface::PrefabData;
pub use model::PrefabModel;
