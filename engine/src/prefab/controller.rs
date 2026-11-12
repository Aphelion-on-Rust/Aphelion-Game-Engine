use super::interface::PrefabData;
use super::model::PrefabModel;
use crate::error::{EngineError, EngineResult};
use crate::identity::{AssetUuid, IdController, IdRegistryModel};
use crate::mivc::{Controller, Model, Store};
use crate::scene::copy::duplicate_subtree;
use crate::scene::{query, ComponentEntry, GameObjectData, SceneModel};

pub struct PrefabController;

impl Controller for PrefabController {}

impl PrefabController {
    /// Saves an object (and everything under it) from a scene as a prefab.
    /// The prefab holds its own copies, so changing the scene later does not change it.
    pub fn create_from_object(
        &self,
        scene: &SceneModel,
        ids: &mut IdRegistryModel,
        object: &AssetUuid,
        name: &str,
    ) -> EngineResult<PrefabModel> {
        let objects = duplicate_subtree(&scene.data().objects, object, ids, None)?;

        Ok(Store::new(PrefabData {
            uuid: IdController.new_uuid(ids),
            name: name.to_string(),
            objects,
        }))
    }

    /// Writes the current state of a scene instance back into the prefab ("Apply").
    ///
    /// The instance and the prefab are lined up branch by branch (the first child with the first
    /// child, and so on) and components are lined up by kind. Everything that lines up keeps the
    /// prefab's own uuid and component id. New parts get new ids, parts that are gone are removed.
    pub fn update_from_instance(
        &self,
        prefab: &mut PrefabModel,
        scene: &SceneModel,
        ids: &mut IdRegistryModel,
        instance_root: &AssetUuid,
    ) -> EngineResult<()> {
        let root = query::find_object(&scene.data().objects, instance_root)
            .ok_or_else(|| EngineError::ObjectNotFound(instance_root.to_string()))?;

        if root.prefab_source != Some(prefab.data().uuid) {
            return Err(EngineError::InvalidOperation("that object is not an instance of this prefab".to_string()));
        }

        let prefab_root = prefab.data().objects.first().map(|object| object.uuid);
        let mut objects = Vec::new();

        collect_branch(&scene.data().objects, instance_root, &prefab.data().objects, prefab_root, None, ids, &mut objects)?;
        self.write(prefab).objects = objects;
        Ok(())
    }

    /// Places a copy of the prefab in a scene. Returns the uuid of the new root object.
    pub fn instantiate(
        &self,
        prefab: &PrefabModel,
        scene: &mut SceneModel,
        ids: &mut IdRegistryModel,
        parent: Option<AssetUuid>,
    ) -> EngineResult<AssetUuid> {
        let source = &prefab.data().objects;
        let root = source
            .first()
            .ok_or_else(|| EngineError::InvalidOperation("the prefab is empty".to_string()))?;

        if let Some(parent_uuid) = &parent {
            query::find_object(&scene.data().objects, parent_uuid)
                .ok_or_else(|| EngineError::ObjectNotFound(parent_uuid.to_string()))?;
        }

        let mut copies = duplicate_subtree(source, &root.uuid, ids, parent)?;

        copies[0].prefab_source = Some(prefab.data().uuid);
        let new_root = copies[0].uuid;

        let data = self.write(scene);
        data.objects.extend(copies);

        if let Some(parent_uuid) = &parent {
            if let Some(owner) = data.objects.iter_mut().find(|object| object.uuid == *parent_uuid) {
                owner.children.push(new_root);
            }
        }

        Ok(new_root)
    }
}

/// Copies an instance branch into `out` (parents first). `counterpart` is the prefab object it
/// lines up with, if any: its uuid and component ids are reused.
fn collect_branch(
    instance: &[GameObjectData],
    instance_uuid: &AssetUuid,
    prefab_objects: &[GameObjectData],
    counterpart: Option<AssetUuid>,
    parent: Option<AssetUuid>,
    ids: &mut IdRegistryModel,
    out: &mut Vec<GameObjectData>,
) -> EngineResult<AssetUuid> {
    let source = query::find_object(instance, instance_uuid)
        .ok_or_else(|| EngineError::ObjectNotFound(instance_uuid.to_string()))?;
    let matching = counterpart.and_then(|uuid| query::find_object(prefab_objects, &uuid));
    let uuid = match matching {
        Some(node) => node.uuid,
        None => IdController.new_uuid(ids),
    };

    // Components line up by kind, in order.
    let mut unused: Vec<&ComponentEntry> = matching.map(|node| node.components.iter().collect()).unwrap_or_default();
    let mut components = Vec::new();

    for component in &source.components {
        let kind = std::mem::discriminant(&component.data);
        let id = match unused.iter().position(|old| std::mem::discriminant(&old.data) == kind) {
            Some(position) => unused.remove(position).id,
            None => IdController.new_component_id(ids),
        };

        components.push(ComponentEntry { id, data: component.data.clone() });
    }

    // The object goes in before its children, so parents always come first.
    let index = out.len();

    out.push(GameObjectData {
        uuid,
        name: source.name.clone(),
        active: source.active,
        parent,
        children: Vec::new(),
        components,
        prefab_source: None,
    });

    let mut children = Vec::new();

    for (position, child) in source.children.iter().enumerate() {
        let child_counterpart = matching.and_then(|node| node.children.get(position)).copied();

        children.push(collect_branch(instance, child, prefab_objects, child_counterpart, Some(uuid), ids, out)?);
    }

    out[index].children = children;
    Ok(uuid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::SceneController;
    use std::collections::HashSet;

    #[test]
    fn every_instance_gets_new_ids() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let tree = SceneController.create_object(&mut scene, &mut ids, "Tree", None).unwrap();
        SceneController.create_object(&mut scene, &mut ids, "Leaves", Some(tree)).unwrap();

        let prefab = PrefabController.create_from_object(&scene, &mut ids, &tree, "Tree").unwrap();
        PrefabController.instantiate(&prefab, &mut scene, &mut ids, None).unwrap();
        PrefabController.instantiate(&prefab, &mut scene, &mut ids, None).unwrap();

        // 3 trees x 2 objects each, and nothing may repeat.
        let objects = &scene.data().objects;
        let uuids: HashSet<_> = objects.iter().map(|object| object.uuid).collect();
        let component_ids: HashSet<_> = objects
            .iter()
            .flat_map(|object| object.components.iter().map(|component| component.id))
            .collect();

        assert_eq!(objects.len(), 6);
        assert_eq!(uuids.len(), 6);
        assert_eq!(component_ids.len(), 6);
    }

    #[test]
    fn instance_root_remembers_its_prefab() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let rock = SceneController.create_object(&mut scene, &mut ids, "Rock", None).unwrap();
        let prefab = PrefabController.create_from_object(&scene, &mut ids, &rock, "Rock").unwrap();

        let placed = PrefabController.instantiate(&prefab, &mut scene, &mut ids, None).unwrap();
        let object = query::find_object(&scene.data().objects, &placed).unwrap();

        assert_eq!(object.prefab_source, Some(prefab.data().uuid));
    }

    #[test]
    fn updating_a_prefab_keeps_its_ids_and_gives_new_parts_new_ids() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let tree = SceneController.create_object(&mut scene, &mut ids, "Tree", None).unwrap();
        SceneController.create_object(&mut scene, &mut ids, "Leaves", Some(tree)).unwrap();

        let mut prefab = PrefabController.create_from_object(&scene, &mut ids, &tree, "Tree").unwrap();
        let old_root = prefab.data().objects[0].clone();
        let old_leaves = prefab.data().objects[1].clone();

        let placed = PrefabController.instantiate(&prefab, &mut scene, &mut ids, None).unwrap();
        let placed_leaves = query::find_object(&scene.data().objects, &placed).unwrap().children[0];

        // Edit the instance: rename a child and add a new one.
        SceneController.rename_object(&mut scene, &placed_leaves, "Branches").unwrap();
        SceneController.create_object(&mut scene, &mut ids, "Fruit", Some(placed)).unwrap();

        PrefabController.update_from_instance(&mut prefab, &scene, &mut ids, &placed).unwrap();

        let objects = &prefab.data().objects;

        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0].uuid, old_root.uuid);
        assert_eq!(objects[0].components[0].id, old_root.components[0].id);
        assert_eq!(objects[1].uuid, old_leaves.uuid);
        assert_eq!(objects[1].name, "Branches");
        assert_eq!(objects[2].name, "Fruit");
        assert_ne!(objects[2].uuid, old_root.uuid);
        assert_ne!(objects[2].uuid, old_leaves.uuid);
        assert_eq!(objects[0].children, vec![old_leaves.uuid, objects[2].uuid]);
        assert!(objects.iter().all(|object| object.prefab_source.is_none()));
    }

    #[test]
    fn only_an_instance_of_the_prefab_can_update_it() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let tree = SceneController.create_object(&mut scene, &mut ids, "Tree", None).unwrap();
        let mut prefab = PrefabController.create_from_object(&scene, &mut ids, &tree, "Tree").unwrap();

        // `tree` was never marked as an instance of the prefab.
        assert!(PrefabController.update_from_instance(&mut prefab, &scene, &mut ids, &tree).is_err());
    }
}
