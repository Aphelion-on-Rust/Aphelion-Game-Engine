use flux_engine::identity::AssetUuid;
use flux_engine::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabListing {
    /// The file name without `.prefab.yaml`.
    pub name: String,
    pub uuid: AssetUuid,
    /// Inside the project, like `Assets/Prefabs/Tree.prefab.yaml`.
    pub file: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PrefabLibraryData {
    /// Sorted by name.
    pub prefabs: Vec<PrefabListing>,
}

impl Interface for PrefabLibraryData {
    const FORMAT: &'static str = "prefab_library";
    const VERSION: u32 = 1;
}
