use super::interface::GameObjectData;
use super::query;
use crate::error::{EngineError, EngineResult};
use crate::identity::{AssetUuid, IdController, IdRegistryModel};
use std::collections::HashMap;

/// Copies an object and everything under it. Every copy gets a brand new uuid
/// and every component a brand new component id, so nothing is ever duplicated.
///
/// The returned list has the new root first. Its parent is `new_parent`.
/// Parent/child links inside the copy are rewritten to point at the copies.
pub(crate) fn duplicate_subtree(
    source: &[GameObjectData],
    root: &AssetUuid,
    ids: &mut IdRegistryModel,
    new_parent: Option<AssetUuid>,
) -> EngineResult<Vec<GameObjectData>> {
    let old_uuids = query::subtree_uuids(source, root);

    if old_uuids.is_empty() {
        return Err(EngineError::ObjectNotFound(root.to_string()));
    }

    // Step 1: give every object a new uuid and remember old -> new.
    let mut new_uuid_of: HashMap<AssetUuid, AssetUuid> = HashMap::new();

    for old in &old_uuids {
        new_uuid_of.insert(*old, IdController.new_uuid(ids));
    }

    // Step 2: copy each object and rewrite its links.
    let mut copies = Vec::with_capacity(old_uuids.len());

    for old in &old_uuids {
        let original = query::find_object(source, old)
            .ok_or_else(|| EngineError::ObjectNotFound(old.to_string()))?;
        let mut copy = original.clone();

        copy.uuid = new_uuid_of[old];
        copy.parent = if old == root {
            new_parent
        } else {
            original.parent.and_then(|parent| new_uuid_of.get(&parent).copied())
        };
        copy.children = original
            .children
            .iter()
            .filter_map(|child| new_uuid_of.get(child).copied())
            .collect();

        for component in &mut copy.components {
            component.id = IdController.new_component_id(ids);
        }

        copies.push(copy);
    }

    Ok(copies)
}
