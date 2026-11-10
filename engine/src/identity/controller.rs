use super::interface::{AssetUuid, ComponentId};
use super::model::IdRegistryModel;
use crate::error::{EngineError, EngineResult};
use crate::mivc::{Controller, Model};

/// Hands out ids and refuses duplicates.
pub struct IdController;

impl Controller for IdController {}

impl IdController {
    /// A new uuid that nobody else has.
    pub fn new_uuid(&self, model: &mut IdRegistryModel) -> AssetUuid {
        loop {
            let candidate = AssetUuid::generate();

            // insert returns false if it was already there, so we try again.
            if self.write(model).used_uuids.insert(candidate) {
                return candidate;
            }
        }
    }

    /// The next component id: 10000, 10001, 10002 ...
    pub fn new_component_id(&self, model: &mut IdRegistryModel) -> ComponentId {
        let data = self.write(model);
        let id = ComponentId::new(data.next_component_id);

        data.next_component_id += 1;
        data.used_component_ids.insert(id);
        id
    }

    /// Use when loading a file that already has a uuid. Fails if it is a duplicate.
    pub fn register_uuid(&self, model: &mut IdRegistryModel, uuid: AssetUuid) -> EngineResult<()> {
        if self.write(model).used_uuids.insert(uuid) {
            Ok(())
        } else {
            Err(EngineError::DuplicateUuid(uuid.to_string()))
        }
    }

    /// Use when loading a component that already has an id. Fails if it is a duplicate.
    pub fn register_component_id(&self, model: &mut IdRegistryModel, id: ComponentId) -> EngineResult<()> {
        let data = self.write(model);

        if !data.used_component_ids.insert(id) {
            return Err(EngineError::DuplicateComponentId(id.value()));
        }

        // Keep the counter ahead of every loaded id.
        data.next_component_id = data.next_component_id.max(id.value() + 1);
        Ok(())
    }

    /// For ids read from a file: marks the uuid as used. Unlike `register_uuid`, it is fine
    /// if it is already known (the id registry file remembers ids from earlier sessions).
    pub fn ensure_uuid(&self, model: &mut IdRegistryModel, uuid: AssetUuid) {
        self.write(model).used_uuids.insert(uuid);
    }

    /// Same for component ids. It also keeps the counter ahead of the id.
    pub fn ensure_component_id(&self, model: &mut IdRegistryModel, id: ComponentId) {
        let data = self.write(model);

        data.used_component_ids.insert(id);
        data.next_component_id = data.next_component_id.max(id.value() + 1);
    }

    pub fn is_uuid_used(&self, model: &IdRegistryModel, uuid: &AssetUuid) -> bool {
        model.data().used_uuids.contains(uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::FIRST_COMPONENT_ID;

    #[test]
    fn uuids_never_repeat() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();

        for _ in 0..1000 {
            controller.new_uuid(&mut model);
        }

        assert_eq!(model.data().used_uuids.len(), 1000);
    }

    #[test]
    fn component_ids_start_at_10000_and_count_up() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();

        assert_eq!(controller.new_component_id(&mut model).value(), FIRST_COMPONENT_ID);
        assert_eq!(controller.new_component_id(&mut model).value(), FIRST_COMPONENT_ID + 1);
    }

    #[test]
    fn duplicate_uuid_is_refused() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();
        let uuid = controller.new_uuid(&mut model);

        assert!(controller.register_uuid(&mut model, uuid).is_err());
    }

    #[test]
    fn loaded_component_id_pushes_counter_forward() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();

        controller.register_component_id(&mut model, ComponentId::new(10_050)).unwrap();

        assert_eq!(controller.new_component_id(&mut model).value(), 10_051);
    }

    #[test]
    fn registry_survives_a_yaml_round_trip() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();
        controller.new_uuid(&mut model);
        controller.new_component_id(&mut model);

        let text = crate::storage::yaml::to_yaml(model.data()).unwrap();
        let loaded: crate::identity::IdRegistryData = crate::storage::yaml::from_yaml(&text).unwrap();

        assert_eq!(loaded.used_uuids.len(), 1);
        assert_eq!(loaded.next_component_id, 10_001);
    }

    #[test]
    fn ensuring_known_ids_is_not_an_error_and_moves_the_counter() {
        let controller = IdController;
        let mut model = IdRegistryModel::default();
        let uuid = controller.new_uuid(&mut model);

        controller.ensure_uuid(&mut model, uuid);
        controller.ensure_component_id(&mut model, ComponentId::new(10_200));
        controller.ensure_component_id(&mut model, ComponentId::new(10_200));

        assert_eq!(model.data().used_uuids.len(), 1);
        assert_eq!(controller.new_component_id(&mut model).value(), 10_201);
    }
}
