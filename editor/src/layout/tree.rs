//! Small functions that rebuild a dock tree. They do not touch any model,
//! the LayoutController calls them and stores the result.

use super::interface::{Axis, DockNode, DropZone, PanelKind};

/// Every panel in the tree, left to right / top to bottom.
pub fn all_panels(node: &DockNode) -> Vec<PanelKind> {
    match node {
        DockNode::Tabs { panels, .. } => panels.clone(),
        DockNode::Split { first, second, .. } => {
            let mut list = all_panels(first);
            list.extend(all_panels(second));
            list
        }
    }
}

/// The tree without `panel`. Empty groups disappear and their split collapses.
/// Returns None if nothing is left.
pub fn remove(node: DockNode, panel: PanelKind) -> Option<DockNode> {
    match node {
        DockNode::Tabs { mut panels, active } => {
            panels.retain(|candidate| *candidate != panel);

            if panels.is_empty() {
                None
            } else {
                let active = active.min(panels.len() - 1);
                Some(DockNode::Tabs { panels, active })
            }
        }
        DockNode::Split { axis, ratio, first, second } => {
            match (remove(*first, panel), remove(*second, panel)) {
                (Some(first), Some(second)) => Some(DockNode::split(axis, ratio, first, second)),
                (Some(only), None) | (None, Some(only)) => Some(only),
                (None, None) => None,
            }
        }
    }
}

/// For the four edges: which way to split, and whether the new panel goes first.
fn edge(zone: DropZone) -> Option<(Axis, bool)> {
    match zone {
        DropZone::Left => Some((Axis::Horizontal, true)),
        DropZone::Right => Some((Axis::Horizontal, false)),
        DropZone::Top => Some((Axis::Vertical, true)),
        DropZone::Bottom => Some((Axis::Vertical, false)),
        DropZone::Center => None,
    }
}

/// Puts `panel` next to the group that holds `target`.
pub fn insert(node: DockNode, target: PanelKind, panel: PanelKind, zone: DropZone) -> DockNode {
    match node {
        DockNode::Tabs { mut panels, active } if panels.contains(&target) => match edge(zone) {
            None => {
                panels.push(panel);
                let active = panels.len() - 1;
                DockNode::Tabs { panels, active }
            }
            Some((axis, new_goes_first)) => {
                let existing = DockNode::Tabs { panels, active };
                let added = DockNode::tabs(vec![panel]);

                if new_goes_first {
                    DockNode::split(axis, 0.3, added, existing)
                } else {
                    DockNode::split(axis, 0.7, existing, added)
                }
            }
        },
        // A group that does not hold the target stays as it is.
        node @ DockNode::Tabs { .. } => node,
        DockNode::Split { axis, ratio, first, second } => DockNode::split(
            axis,
            ratio,
            insert(*first, target, panel, zone),
            insert(*second, target, panel, zone),
        ),
    }
}

/// Makes the tab with `panel` the visible one. False if the panel is not in the tree.
pub fn activate(node: &mut DockNode, panel: PanelKind) -> bool {
    match node {
        DockNode::Tabs { panels, active } => match panels.iter().position(|p| *p == panel) {
            Some(index) => {
                *active = index;
                true
            }
            None => false,
        },
        DockNode::Split { first, second, .. } => activate(first, panel) || activate(second, panel),
    }
}

/// Adds the panel as a tab in the first group (the left-most one) and shows it.
pub fn add_to_first_group(node: &mut DockNode, panel: PanelKind) {
    match node {
        DockNode::Tabs { panels, active } => {
            panels.push(panel);
            *active = panels.len() - 1;
        }
        DockNode::Split { first, .. } => add_to_first_group(first, panel),
    }
}

/// Finds a split by its path from the root: false = first child, true = second child.
pub fn split_ratio_at<'a>(node: &'a mut DockNode, path: &[bool]) -> Option<&'a mut f32> {
    match node {
        DockNode::Tabs { .. } => None,
        DockNode::Split { ratio, first, second, .. } => match path.split_first() {
            None => Some(ratio),
            Some((go_second, rest)) => {
                let child = if *go_second { second } else { first };
                split_ratio_at(child, rest)
            }
        },
    }
}
