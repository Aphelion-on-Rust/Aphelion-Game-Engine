use crate::mivc::Interface;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use uuid::Uuid;

/// Component ids start here. Anything below is reserved for the engine.
pub const FIRST_COMPONENT_ID: u32 = 10_000;

/// Fixed identity of an asset or object. Written once, never edited.
/// Editing a script keeps its uuid. Copying a prefab makes a brand new uuid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetUuid(Uuid);

impl AssetUuid {
    /// Only the IdController calls this, so duplicates are always checked.
    pub(super) fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(text: &str) -> Option<Self> {
        Uuid::parse_str(text).ok().map(Self)
    }
}

impl fmt::Display for AssetUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The extra whole-number id of a component. Starts at 10000.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(u32);

impl ComponentId {
    pub(super) fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// What gets saved: every id that is in use, and the next free component id.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdRegistryData {
    pub used_uuids: BTreeSet<AssetUuid>,
    pub used_component_ids: BTreeSet<ComponentId>,
    pub next_component_id: u32,
}

impl Default for IdRegistryData {
    fn default() -> Self {
        Self {
            used_uuids: BTreeSet::new(),
            used_component_ids: BTreeSet::new(),
            next_component_id: FIRST_COMPONENT_ID,
        }
    }
}

impl Interface for IdRegistryData {
    const FORMAT: &'static str = "id_registry";
    const VERSION: u32 = 1;
}
