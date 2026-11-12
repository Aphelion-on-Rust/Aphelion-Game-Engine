//! Finding what changed between two states of a prefab, and undoing such changes.

use super::interface::PrefabChange;
use crate::identity::AssetUuid;
use crate::prefab::PrefabData;
use crate::scene::GameObjectData;
use std::collections::HashSet;

/// The prefab after undoing a revision, plus what happened.
pub struct RevertReport {
    pub prefab: PrefabData,
    /// How many changes were undone.
    pub applied: usize,
    /// Changes that were kept because later edits touched the same thing. Each is a sentence.
    pub skipped: Vec<String>,
}

fn find<'a>(objects: &'a [GameObjectData], uuid: &AssetUuid) -> Option<&'a GameObjectData> {
    objects.iter().find(|object| object.uuid == *uuid)
}

fn find_mut<'a>(objects: &'a mut [GameObjectData], uuid: &AssetUuid) -> Option<&'a mut GameObjectData> {
    objects.iter_mut().find(|object| object.uuid == *uuid)
}

fn name_of(objects: &[GameObjectData], uuid: &AssetUuid) -> String {
    find(objects, uuid).map_or_else(|| "an object".to_string(), |object| format!("\"{}\"", object.name))
}

/// Everything that changed going from `old` to `new`.
pub fn diff(old: &PrefabData, new: &PrefabData) -> Vec<PrefabChange> {
    let mut changes = Vec::new();

    for object in &old.objects {
        if find(&new.objects, &object.uuid).is_none() {
            changes.push(PrefabChange::ObjectRemoved { object: object.clone() });
        }
    }

    for object in &new.objects {
        if find(&old.objects, &object.uuid).is_none() {
            changes.push(PrefabChange::ObjectAdded { object: object.clone() });
        }
    }

    for after in &new.objects {
        let Some(before) = find(&old.objects, &after.uuid) else {
            continue;
        };
        let object = after.uuid;

        if before.name != after.name {
            changes.push(PrefabChange::ObjectRenamed { object, from: before.name.clone(), to: after.name.clone() });
        }

        if before.active != after.active {
            changes.push(PrefabChange::ObjectActiveChanged { object, from: before.active, to: after.active });
        }

        if before.parent != after.parent {
            changes.push(PrefabChange::ObjectMoved { object, from: before.parent, to: after.parent });
        }

        for component in &before.components {
            match after.components.iter().find(|other| other.id == component.id) {
                None => changes.push(PrefabChange::ComponentRemoved { object, component: component.clone() }),
                Some(other) if other.data != component.data => changes.push(PrefabChange::ComponentEdited {
                    object,
                    component: component.id,
                    from: component.data.clone(),
                    to: other.data.clone(),
                }),
                Some(_) => {}
            }
        }

        for component in &after.components {
            if !before.components.iter().any(|other| other.id == component.id) {
                changes.push(PrefabChange::ComponentAdded { object, component: component.clone() });
            }
        }
    }

    changes
}

/// Undoes the given changes (one revision) on the current prefab, and nothing else.
/// A change is skipped, not forced, if later edits touched the same thing.
pub fn revert(current: &PrefabData, changes: &[PrefabChange]) -> RevertReport {
    let mut prefab = current.clone();
    let mut applied = 0;
    let mut skipped = Vec::new();

    // 1. Bring back removed objects first, in the order they were removed (parents before children).
    for change in changes {
        if let PrefabChange::ObjectRemoved { object } = change {
            match restore_object(&mut prefab, object) {
                Ok(()) => applied += 1,
                Err(reason) => skipped.push(reason),
            }
        }
    }

    // 2. Undo the rest, last change first.
    for change in changes.iter().rev() {
        if matches!(change, PrefabChange::ObjectRemoved { .. }) {
            continue;
        }

        match undo_change(&mut prefab, change) {
            Ok(()) => applied += 1,
            Err(reason) => skipped.push(reason),
        }
    }

    fix_hierarchy(&mut prefab.objects);
    RevertReport { prefab, applied, skipped }
}

fn restore_object(prefab: &mut PrefabData, object: &GameObjectData) -> Result<(), String> {
    if find(&prefab.objects, &object.uuid).is_some() {
        return Err(format!("\"{}\" is already back", object.name));
    }

    if let Some(parent) = object.parent {
        if find(&prefab.objects, &parent).is_none() {
            return Err(format!("\"{}\" can't come back, its parent is gone", object.name));
        }
    }

    prefab.objects.push(object.clone());
    Ok(())
}

/// True if `candidate` is `ancestor` or sits below it.
fn is_below(objects: &[GameObjectData], candidate: AssetUuid, ancestor: AssetUuid) -> bool {
    let mut current = Some(candidate);

    while let Some(uuid) = current {
        if uuid == ancestor {
            return true;
        }

        current = find(objects, &uuid).and_then(|object| object.parent);
    }

    false
}

fn undo_change(prefab: &mut PrefabData, change: &PrefabChange) -> Result<(), String> {
    match change {
        PrefabChange::ObjectRemoved { .. } => Ok(()), // handled before

        PrefabChange::ObjectAdded { object } => {
            let Some(position) = prefab.objects.iter().position(|other| other.uuid == object.uuid) else {
                return Err(format!("\"{}\" is already gone", object.name));
            };

            if prefab.objects.iter().any(|other| other.parent == Some(object.uuid)) {
                return Err(format!("\"{}\" was kept, it has newer children", object.name));
            }

            prefab.objects.remove(position);
            Ok(())
        }

        PrefabChange::ObjectRenamed { object, from, to } => {
            let label = name_of(&prefab.objects, object);
            let target = find_mut(&mut prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?;

            if target.name == *to {
                target.name = from.clone();
                Ok(())
            } else {
                Err(format!("the name of {label} was kept, it was renamed again"))
            }
        }

        PrefabChange::ObjectActiveChanged { object, from, to } => {
            let label = name_of(&prefab.objects, object);
            let target = find_mut(&mut prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?;

            if target.active == *to {
                target.active = *from;
                Ok(())
            } else {
                Err(format!("the on/off switch of {label} was kept, it was changed again"))
            }
        }

        PrefabChange::ObjectMoved { object, from, to } => {
            let label = name_of(&prefab.objects, object);
            let current_parent = find(&prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?.parent;

            if current_parent != *to {
                return Err(format!("the place of {label} was kept, it was moved again"));
            }

            if let Some(parent) = from {
                if find(&prefab.objects, parent).is_none() {
                    return Err(format!("{label} stays, its old parent is gone"));
                }

                if is_below(&prefab.objects, *parent, *object) {
                    return Err(format!("{label} stays, moving it back would make a loop"));
                }
            }

            if let Some(target) = find_mut(&mut prefab.objects, object) {
                target.parent = *from;
            }

            Ok(())
        }

        PrefabChange::ComponentAdded { object, component } => {
            let label = name_of(&prefab.objects, object);
            let target = find_mut(&mut prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?;
            let Some(position) = target.components.iter().position(|other| other.id == component.id) else {
                return Err(format!("a component of {label} is already gone"));
            };

            if component.data.is_placement() {
                return Err(format!("the Transform of {label} can't be removed"));
            }

            if target.components[position].data != component.data {
                return Err(format!("a component of {label} was kept, it was edited since"));
            }

            target.components.remove(position);
            Ok(())
        }

        PrefabChange::ComponentRemoved { object, component } => {
            let label = name_of(&prefab.objects, object);
            let target = find_mut(&mut prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?;

            if target.components.iter().any(|other| other.id == component.id) {
                return Err(format!("a component of {label} is already back"));
            }

            if component.data.is_placement() && target.components.iter().any(|other| other.data.is_placement()) {
                return Err(format!("{label} already has a Transform"));
            }

            target.components.push(component.clone());
            Ok(())
        }

        PrefabChange::ComponentEdited { object, component, from, to } => {
            let label = name_of(&prefab.objects, object);
            let target = find_mut(&mut prefab.objects, object).ok_or_else(|| format!("{label} is gone"))?;
            let entry = target
                .components
                .iter_mut()
                .find(|other| other.id == *component)
                .ok_or_else(|| format!("a component of {label} is gone"))?;

            if entry.data == *to {
                entry.data = from.clone();
                Ok(())
            } else {
                Err(format!("a component of {label} was kept, it was changed again"))
            }
        }
    }
}

/// Makes the parent / children links agree again after changes: a missing parent becomes the top
/// level, and every object's children are the objects that name it as their parent
/// (existing order kept, new ones last).
fn fix_hierarchy(objects: &mut [GameObjectData]) {
    let existing: HashSet<AssetUuid> = objects.iter().map(|object| object.uuid).collect();

    for object in objects.iter_mut() {
        if let Some(parent) = object.parent {
            if !existing.contains(&parent) {
                object.parent = None;
            }
        }
    }

    let links: Vec<(AssetUuid, Option<AssetUuid>)> = objects.iter().map(|object| (object.uuid, object.parent)).collect();

    for object in objects.iter_mut() {
        let owner = object.uuid;
        let mut children: Vec<AssetUuid> = object
            .children
            .iter()
            .copied()
            .filter(|child| links.iter().any(|(uuid, parent)| uuid == child && *parent == Some(owner)))
            .collect();

        for (uuid, parent) in &links {
            if *parent == Some(owner) && !children.contains(uuid) {
                children.push(*uuid);
            }
        }

        object.children = children;
    }
}
