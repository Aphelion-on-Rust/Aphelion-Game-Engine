//! The list of prefab files in the open project (name, uuid, file). Read when the project is
//! opened and kept up to date when prefabs are saved.

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::PrefabLibraryController;
pub use interface::{PrefabLibraryData, PrefabListing};
pub use model::PrefabLibraryModel;
