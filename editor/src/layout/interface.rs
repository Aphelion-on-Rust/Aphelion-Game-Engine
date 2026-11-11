use flux_engine::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PanelKind {
    Hierarchy,
    Scene,
    Game,
    Inspector,
    Project,
    Console,
    Animator,
    Timeline,
    Terrain,
    Profiler,
    History,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// Children sit side by side.
    Horizontal,
    /// Children sit one above the other.
    Vertical,
}

/// Where a dragged panel is dropped on a target panel's group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZone {
    Left,
    Right,
    Top,
    Bottom,
    /// Becomes a tab in the same group.
    Center,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "node")]
pub enum DockNode {
    Split {
        axis: Axis,
        /// Share of the space the `first` child gets (0.1 to 0.9).
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
    Tabs {
        panels: Vec<PanelKind>,
        /// Index in `panels` of the visible tab.
        active: usize,
    },
}

impl DockNode {
    pub fn tabs(panels: Vec<PanelKind>) -> DockNode {
        DockNode::Tabs { panels, active: 0 }
    }

    pub fn split(axis: Axis, ratio: f32, first: DockNode, second: DockNode) -> DockNode {
        DockNode::Split { axis, ratio, first: Box::new(first), second: Box::new(second) }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutData {
    pub root: DockNode,
}

/// The first layout a new user sees:
///
/// ```text
/// | Hierarchy | Scene / Game                | Inspector |
/// |           | Project / Console / History |           |
/// ```
impl Default for LayoutData {
    fn default() -> Self {
        let center = DockNode::split(
            Axis::Vertical,
            0.7,
            DockNode::tabs(vec![PanelKind::Scene, PanelKind::Game]),
            DockNode::tabs(vec![PanelKind::Project, PanelKind::Console, PanelKind::History]),
        );
        let center_and_inspector = DockNode::split(
            Axis::Horizontal,
            0.78,
            center,
            DockNode::tabs(vec![PanelKind::Inspector]),
        );

        Self {
            root: DockNode::split(
                Axis::Horizontal,
                0.18,
                DockNode::tabs(vec![PanelKind::Hierarchy]),
                center_and_inspector,
            ),
        }
    }
}

impl Interface for LayoutData {
    const FORMAT: &'static str = "editor_layout";
    const VERSION: u32 = 1;
}
