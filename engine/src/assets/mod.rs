//! Asset files and their fixed uuids.
//!
//! Every asset file (a picture, for now) has a small `<file>.meta.yaml` next to it with the asset's
//! uuid. The uuid is made once and never changes, even if the file is edited, moved or renamed together
//! with its meta file. Other things (a sprite, a mesh) refer to the asset by that uuid.
//!
//! The engine side here is the data and the rules. Scanning the folder and decoding pictures is done
//! by the editor (`assets_io`).

pub mod controller;
pub mod interface;
pub mod model;
pub mod query;

pub use controller::AssetLibraryController;
pub use interface::{AssetEntry, AssetKind, AssetLibraryData, AssetMetaData, TextureFilter};
pub use model::AssetLibraryModel;
