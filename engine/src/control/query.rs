use super::interface::{ControlData, PrefabChange, PrefabHistoryData, Revision};
use crate::identity::{AssetUuid, ComponentId};

pub fn find_history<'a>(data: &'a ControlData, prefab: &AssetUuid) -> Option<&'a PrefabHistoryData> {
    data.prefabs.iter().find(|history| history.prefab == *prefab)
}

/// How a revision looks in a list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RevisionSummary {
    pub objects_added: usize,
    pub objects_removed: usize,
    /// Renames, switches, moves and every component change.
    pub edits: usize,
}

impl RevisionSummary {
    /// Like "+2 objects, -1 object, 3 edits".
    pub fn to_text(&self) -> String {
        let mut parts = Vec::new();
        let plural = |count: usize| if count == 1 { "object" } else { "objects" };

        if self.objects_added > 0 {
            parts.push(format!("+{} {}", self.objects_added, plural(self.objects_added)));
        }

        if self.objects_removed > 0 {
            parts.push(format!("-{} {}", self.objects_removed, plural(self.objects_removed)));
        }

        if self.edits > 0 {
            parts.push(format!("{} {}", self.edits, if self.edits == 1 { "edit" } else { "edits" }));
        }

        parts.join(", ")
    }
}

pub fn summarize(revision: &Revision) -> RevisionSummary {
    let mut summary = RevisionSummary::default();

    for change in &revision.changes {
        match change {
            PrefabChange::ObjectAdded { .. } => summary.objects_added += 1,
            PrefabChange::ObjectRemoved { .. } => summary.objects_removed += 1,
            _ => summary.edits += 1,
        }
    }

    summary
}

/// Every object uuid and component id that appears anywhere in a history, even in objects that
/// were removed later. They are all marked as used, so an id is never handed out twice.
pub fn used_ids(history: &PrefabHistoryData) -> (Vec<AssetUuid>, Vec<ComponentId>) {
    let mut uuids = vec![history.prefab];
    let mut component_ids = Vec::new();

    for revision in &history.revisions {
        for change in &revision.changes {
            match change {
                PrefabChange::ObjectAdded { object } | PrefabChange::ObjectRemoved { object } => {
                    uuids.push(object.uuid);
                    component_ids.extend(object.components.iter().map(|component| component.id));
                }
                PrefabChange::ComponentAdded { component, .. } | PrefabChange::ComponentRemoved { component, .. } => {
                    component_ids.push(component.id);
                }
                _ => {}
            }
        }
    }

    (uuids, component_ids)
}
