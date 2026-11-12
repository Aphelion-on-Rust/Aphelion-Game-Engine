use crate::identity::{AssetUuid, ComponentId};
use crate::mivc::Interface;
use crate::scene::{ComponentData, ComponentEntry, GameObjectData};
use serde::{Deserialize, Serialize};

/// One thing that changed in a prefab. Every change keeps the old and the new value,
/// so it can be undone later without looking at any other revision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change")]
pub enum PrefabChange {
    ObjectAdded { object: GameObjectData },
    /// The whole object is kept, so it can come back.
    ObjectRemoved { object: GameObjectData },
    ObjectRenamed { object: AssetUuid, from: String, to: String },
    ObjectActiveChanged { object: AssetUuid, from: bool, to: bool },
    /// The parent changed.
    ObjectMoved { object: AssetUuid, from: Option<AssetUuid>, to: Option<AssetUuid> },
    ComponentAdded { object: AssetUuid, component: ComponentEntry },
    ComponentRemoved { object: AssetUuid, component: ComponentEntry },
    ComponentEdited { object: AssetUuid, component: ComponentId, from: ComponentData, to: ComponentData },
}

/// One save of a prefab.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Revision {
    /// 1, 2, 3 ... never reused.
    pub number: u32,
    pub saved_on: String,
    pub message: String,
    /// Set when this revision was made by reverting another one.
    pub reverts: Option<u32>,
    pub changes: Vec<PrefabChange>,
}

/// The history of one prefab. Saved as `.aphelion/prefabs/<prefab uuid>.yaml`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefabHistoryData {
    pub prefab: AssetUuid,
    pub name: String,
    /// Where the prefab file is, inside the project (`Assets/Prefabs/Tree.prefab.yaml`).
    pub file: String,
    pub revisions: Vec<Revision>,
}

impl Interface for PrefabHistoryData {
    const FORMAT: &'static str = "aphelion_prefab_history";
    const VERSION: u32 = 1;
}

/// The histories of all prefabs of the project, in memory.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlData {
    pub prefabs: Vec<PrefabHistoryData>,
}

impl Interface for ControlData {
    const FORMAT: &'static str = "aphelion_control";
    const VERSION: u32 = 1;
}
