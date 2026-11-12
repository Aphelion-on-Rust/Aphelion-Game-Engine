use crate::identity::AssetUuid;
use crate::mivc::Interface;
use crate::scene::GameObjectData;
use serde::{Deserialize, Serialize};

/// A saved branch of objects. `objects[0]` is always the root.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabData {
    pub uuid: AssetUuid,
    pub name: String,
    pub objects: Vec<GameObjectData>,
}

impl Interface for PrefabData {
    const FORMAT: &'static str = "prefab";
    const VERSION: u32 = 1;
}
