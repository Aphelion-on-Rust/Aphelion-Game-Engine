use crate::identity::AssetUuid;
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Texture,
    Mesh,
    Material,
    Shader,
    Audio,
    Animation,
    Other,
}

/// A safe reference to a resource. If the resource is collected and its slot is reused,
/// the generation changes, so old handles are recognized as stale instead of pointing
/// at the wrong resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceHandle {
    pub slot: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GcEntry {
    pub uuid: AssetUuid,
    pub kind: ResourceKind,
    pub generation: u32,
    /// How many users hold this resource right now.
    pub ref_count: u32,
    /// Frames in a row with nobody using it.
    pub idle_frames: u32,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GcData {
    pub entries: Vec<GcEntry>,
    /// Slots of collected resources, ready to be reused.
    pub free_slots: Vec<u32>,
    pub frame: u64,
    /// How many unused frames before a resource is collected. 120 = about 2 seconds at 60 fps.
    pub grace_frames: u32,
}

impl Default for GcData {
    fn default() -> Self {
        Self { entries: Vec::new(), free_slots: Vec::new(), frame: 0, grace_frames: 120 }
    }
}

/// Handed back by `collect` so the owner can free the real memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectedResource {
    pub uuid: AssetUuid,
    pub kind: ResourceKind,
}

impl Interface for GcData {
    const FORMAT: &'static str = "gc_state";
    const VERSION: u32 = 1;
}
