//! The list of engine releases found on GitHub. It is cached in a YAML file,
//! so the launcher still shows versions when offline.

pub mod controller;
pub mod interface;
pub mod model;
pub mod parse;
pub mod query;

pub use controller::ReleasesController;
pub use interface::{AssetInfo, ReleaseCacheData, ReleaseInfo};
pub use model::ReleasesModel;
