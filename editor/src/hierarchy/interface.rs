use flux_engine::identity::AssetUuid;
use flux_engine::mivc::Interface;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// UI state of the hierarchy panel (not the scene itself).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HierarchyUiData {
    pub expanded: BTreeSet<AssetUuid>,
    pub selected: Vec<AssetUuid>,
}

impl Interface for HierarchyUiData {
    const FORMAT: &'static str = "hierarchy_ui";
    const VERSION: u32 = 1;
}

/// One line in the panel, ready to draw.
#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyRow {
    pub uuid: AssetUuid,
    pub name: String,
    /// 0 for top-level objects. Used for indenting.
    pub depth: u32,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
    /// The object's own active switch.
    pub active: bool,
    /// False if the object or any parent is off. Draw these greyed out.
    pub active_in_hierarchy: bool,
}
