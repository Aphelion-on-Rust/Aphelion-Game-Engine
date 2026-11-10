//! Read-only helpers. They take plain slices, so views and controllers can both use them.

use super::interface::GameObjectData;
use crate::identity::AssetUuid;
use std::collections::HashSet;

pub fn find_object<'a>(objects: &'a [GameObjectData], uuid: &AssetUuid) -> Option<&'a GameObjectData> {
    objects.iter().find(|object| object.uuid == *uuid)
}

/// Objects that have no parent.
pub fn root_objects(objects: &[GameObjectData]) -> Vec<&GameObjectData> {
    objects.iter().filter(|object| object.parent.is_none()).collect()
}

/// The object and everything under it. The root comes first, then children in order.
pub fn subtree_uuids(objects: &[GameObjectData], root: &AssetUuid) -> Vec<AssetUuid> {
    let mut result = Vec::new();
    let mut waiting = vec![*root];

    while let Some(uuid) = waiting.pop() {
        if let Some(object) = find_object(objects, &uuid) {
            result.push(uuid);

            // Pushed in reverse so the first child is handled first.
            waiting.extend(object.children.iter().rev().copied());
        }
    }

    result
}

/// True if `candidate` is `ancestor` itself or sits anywhere below it.
pub fn is_descendant_of(objects: &[GameObjectData], candidate: &AssetUuid, ancestor: &AssetUuid) -> bool {
    subtree_uuids(objects, ancestor).contains(candidate)
}

/// True if the object and all of its parents are active.
/// An object inside a switched-off parent is switched off too.
pub fn is_active_in_hierarchy(objects: &[GameObjectData], uuid: &AssetUuid) -> bool {
    let mut current = find_object(objects, uuid);

    if current.is_none() {
        return false;
    }

    while let Some(object) = current {
        if !object.active {
            return false;
        }

        current = object.parent.as_ref().and_then(|parent| find_object(objects, parent));
    }

    true
}

/// Checks a scene that was read from a file: no uuid and no component id may appear twice.
/// Returns a sentence describing the first problem, or None if all is fine.
pub fn find_duplicate_ids(objects: &[GameObjectData]) -> Option<String> {
    let mut uuids = HashSet::new();
    let mut component_ids = HashSet::new();

    for object in objects {
        if !uuids.insert(object.uuid) {
            return Some(format!("the object uuid {} appears twice", object.uuid));
        }

        for component in &object.components {
            if !component_ids.insert(component.id) {
                return Some(format!("the component id {} appears twice", component.id.value()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::IdRegistryModel;
    use crate::mivc::{Controller, Model};
    use crate::scene::SceneController;

    #[test]
    fn a_copied_object_is_reported_as_a_duplicate() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let object = SceneController.create_object(&mut scene, &mut ids, "A", None).unwrap();

        assert!(find_duplicate_ids(&scene.data().objects).is_none());

        let copy = find_object(&scene.data().objects, &object).unwrap().clone();
        SceneController.write(&mut scene).objects.push(copy);

        assert!(find_duplicate_ids(&scene.data().objects).is_some());
    }
}
