use super::interface::PrefabData;
use super::model::PrefabModel;
use crate::error::{EngineError, EngineResult};
use crate::identity::{AssetUuid, IdController, IdRegistryModel};
use crate::mivc::{Controller, Model, Store};
use crate::scene::copy::duplicate_subtree;
use crate::scene::{query, SceneModel};

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
}
