use super::interface::{
    ComponentData, ComponentEntry, GameObjectData, SceneData, TransformData,
};
use super::model::SceneModel;
use super::query;
use crate::error::{EngineError, EngineResult};
use crate::identity::{AssetUuid, ComponentId, IdController, IdRegistryModel};
use crate::mivc::{Controller, Model, Store};

/// All the logic for changing a scene.
pub struct SceneController;

impl Controller for SceneController {}

fn not_found(uuid: &AssetUuid) -> EngineError {
    EngineError::ObjectNotFound(uuid.to_string())
}

fn find_mut<'a>(objects: &'a mut [GameObjectData], uuid: &AssetUuid) -> EngineResult<&'a mut GameObjectData> {
    objects.iter_mut().find(|object| object.uuid == *uuid).ok_or_else(|| not_found(uuid))
}

impl SceneController {
    pub fn create_scene(&self, ids: &mut IdRegistryModel, name: &str) -> SceneModel {
        Store::new(SceneData {
            uuid: IdController.new_uuid(ids),
            name: name.to_string(),
            objects: Vec::new(),
        })
    }

    /// Makes a new object with a Transform. Pass a parent to put it in the hierarchy.
    pub fn create_object(
        &self,
        scene: &mut SceneModel,
        ids: &mut IdRegistryModel,
        name: &str,
        parent: Option<AssetUuid>,
    ) -> EngineResult<AssetUuid> {
        if let Some(parent_uuid) = &parent {
            query::find_object(&scene.data().objects, parent_uuid).ok_or_else(|| not_found(parent_uuid))?;
        }

        let uuid = IdController.new_uuid(ids);
        let transform = ComponentEntry {
            id: IdController.new_component_id(ids),
            data: ComponentData::Transform(TransformData::default()),
        };

        let data = self.write(scene);

        data.objects.push(GameObjectData {
            uuid,
            name: name.to_string(),
            active: true,
            parent,
            children: Vec::new(),
            components: vec![transform],
            prefab_source: None,
        });

        if let Some(parent_uuid) = &parent {
            find_mut(&mut data.objects, parent_uuid)?.children.push(uuid);
        }

        Ok(uuid)
    }

    pub fn add_component(
        &self,
        scene: &mut SceneModel,
        ids: &mut IdRegistryModel,
        object: &AssetUuid,
        component: ComponentData,
    ) -> EngineResult<ComponentId> {
        let existing = query::find_object(&scene.data().objects, object).ok_or_else(|| not_found(object))?;

        if component.is_placement() && existing.components.iter().any(|entry| entry.data.is_placement()) {
            return Err(EngineError::InvalidOperation(
                "an object can only have one Transform or RectTransform".to_string(),
            ));
        }

        let id = IdController.new_component_id(ids);

        find_mut(&mut self.write(scene).objects, object)?
            .components
            .push(ComponentEntry { id, data: component });

        Ok(id)
    }

    /// Changes the data of a component (for example the position in a Transform).
    /// The component must stay the same kind, a Transform can't turn into a Script.
    pub fn update_component(
        &self,
        scene: &mut SceneModel,
        object: &AssetUuid,
        component: ComponentId,
        change: impl FnOnce(&mut ComponentData),
    ) -> EngineResult<()> {
        let target = find_mut(&mut self.write(scene).objects, object)?;
        let entry = target
            .components
            .iter_mut()
            .find(|entry| entry.id == component)
            .ok_or(EngineError::ComponentNotFound(component.value()))?;

        let backup = entry.data.clone();
        change(&mut entry.data);

        if std::mem::discriminant(&entry.data) != std::mem::discriminant(&backup) {
            entry.data = backup;
            return Err(EngineError::InvalidOperation("a component can't change into another kind".to_string()));
        }

        Ok(())
    }

    pub fn remove_component(
        &self,
        scene: &mut SceneModel,
        object: &AssetUuid,
        component: ComponentId,
    ) -> EngineResult<()> {
        let target = find_mut(&mut self.write(scene).objects, object)?;
        let position = target
            .components
            .iter()
            .position(|entry| entry.id == component)
            .ok_or(EngineError::ComponentNotFound(component.value()))?;

        if target.components[position].data.is_placement() {
            return Err(EngineError::InvalidOperation(
                "the Transform or RectTransform cannot be removed".to_string(),
            ));
        }

        target.components.remove(position);
        Ok(())
    }

    /// Moves an object under a new parent (or to the top level with None).
    pub fn set_parent(
        &self,
        scene: &mut SceneModel,
        object: &AssetUuid,
        new_parent: Option<AssetUuid>,
    ) -> EngineResult<()> {
        let objects = &scene.data().objects;
        let existing = query::find_object(objects, object).ok_or_else(|| not_found(object))?;
        let old_parent = existing.parent;

        if let Some(parent_uuid) = &new_parent {
            query::find_object(objects, parent_uuid).ok_or_else(|| not_found(parent_uuid))?;

            // An object can't go inside itself or inside one of its own children.
            if query::is_descendant_of(objects, parent_uuid, object) {
                return Err(EngineError::InvalidOperation(
                    "an object cannot be parented under itself or its own child".to_string(),
                ));
            }
        }

        let data = self.write(scene);

        if let Some(old) = &old_parent {
            find_mut(&mut data.objects, old)?.children.retain(|child| child != object);
        }

        find_mut(&mut data.objects, object)?.parent = new_parent;

        if let Some(parent_uuid) = &new_parent {
            find_mut(&mut data.objects, parent_uuid)?.children.push(*object);
        }

        Ok(())
    }

    /// Removes the object and everything under it.
    /// The uuids stay marked as used in the registry, so they are never handed out again.
    pub fn destroy_object(&self, scene: &mut SceneModel, object: &AssetUuid) -> EngineResult<()> {
        let objects = &scene.data().objects;
        let parent = query::find_object(objects, object).ok_or_else(|| not_found(object))?.parent;
        let doomed = query::subtree_uuids(objects, object);

        let data = self.write(scene);

        if let Some(parent_uuid) = &parent {
            find_mut(&mut data.objects, parent_uuid)?.children.retain(|child| child != object);
        }

        data.objects.retain(|entry| !doomed.contains(&entry.uuid));
        Ok(())
    }

    pub fn rename_object(&self, scene: &mut SceneModel, object: &AssetUuid, name: &str) -> EngineResult<()> {
        find_mut(&mut self.write(scene).objects, object)?.name = name.to_string();
        Ok(())
    }

    pub fn set_active(&self, scene: &mut SceneModel, object: &AssetUuid, active: bool) -> EngineResult<()> {
        find_mut(&mut self.write(scene).objects, object)?.active = active;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::yaml::{from_yaml, to_yaml};

    fn setup() -> (SceneController, SceneModel, IdRegistryModel) {
        let mut ids = IdRegistryModel::default();
        let scene = SceneController.create_scene(&mut ids, "Test");

        (SceneController, scene, ids)
    }

    #[test]
    fn new_object_gets_a_transform() {
        let (controller, mut scene, mut ids) = setup();
        let uuid = controller.create_object(&mut scene, &mut ids, "Player", None).unwrap();
        let object = query::find_object(&scene.data().objects, &uuid).unwrap();

        assert_eq!(object.components.len(), 1);
        assert!(object.components[0].data.is_placement());
        assert_eq!(object.components[0].id.value(), 10_000);
    }

    #[test]
    fn child_is_linked_both_ways() {
        let (controller, mut scene, mut ids) = setup();
        let parent = controller.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        let child = controller.create_object(&mut scene, &mut ids, "Child", Some(parent)).unwrap();
        let objects = &scene.data().objects;

        assert_eq!(query::find_object(objects, &child).unwrap().parent, Some(parent));
        assert_eq!(query::find_object(objects, &parent).unwrap().children, vec![child]);
    }

    #[test]
    fn cannot_parent_into_own_child() {
        let (controller, mut scene, mut ids) = setup();
        let parent = controller.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        let child = controller.create_object(&mut scene, &mut ids, "Child", Some(parent)).unwrap();

        assert!(controller.set_parent(&mut scene, &parent, Some(child)).is_err());
        assert!(controller.set_parent(&mut scene, &parent, Some(parent)).is_err());
    }

    #[test]
    fn destroying_removes_the_whole_branch() {
        let (controller, mut scene, mut ids) = setup();
        let parent = controller.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        controller.create_object(&mut scene, &mut ids, "Child", Some(parent)).unwrap();
        let other = controller.create_object(&mut scene, &mut ids, "Other", None).unwrap();

        controller.destroy_object(&mut scene, &parent).unwrap();

        assert_eq!(scene.data().objects.len(), 1);
        assert_eq!(scene.data().objects[0].uuid, other);
    }

    #[test]
    fn second_transform_is_refused() {
        let (controller, mut scene, mut ids) = setup();
        let uuid = controller.create_object(&mut scene, &mut ids, "Thing", None).unwrap();
        let extra = ComponentData::Transform(TransformData::default());

        assert!(controller.add_component(&mut scene, &mut ids, &uuid, extra).is_err());
    }

    #[test]
    fn scene_survives_a_yaml_round_trip() {
        let (controller, mut scene, mut ids) = setup();
        let parent = controller.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        controller.create_object(&mut scene, &mut ids, "Child", Some(parent)).unwrap();

        let text = to_yaml(scene.data()).unwrap();
        let loaded: SceneData = from_yaml(&text).unwrap();

        assert_eq!(loaded.objects, scene.data().objects);
    }

    #[test]
    fn components_can_be_edited_but_not_turned_into_another_kind() {
        let (controller, mut scene, mut ids) = setup();
        let uuid = controller.create_object(&mut scene, &mut ids, "Thing", None).unwrap();
        let component = scene.data().objects[0].components[0].id;

        controller
            .update_component(&mut scene, &uuid, component, |data| {
                if let ComponentData::Transform(transform) = data {
                    transform.position.x = 5.0;
                }
            })
            .unwrap();

        match &scene.data().objects[0].components[0].data {
            ComponentData::Transform(transform) => assert_eq!(transform.position.x, 5.0),
            other => panic!("expected a transform, got {other:?}"),
        }

        let result = controller.update_component(&mut scene, &uuid, component, |data| {
            *data = ComponentData::RectTransform(Default::default());
        });

        assert!(result.is_err());
        assert!(matches!(scene.data().objects[0].components[0].data, ComponentData::Transform(_)));
    }
}
