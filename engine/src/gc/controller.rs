use super::interface::{CollectedResource, GcEntry, ResourceHandle, ResourceKind};
use super::model::GcModel;
use crate::error::{EngineError, EngineResult};
use crate::identity::AssetUuid;
use crate::mivc::Controller;

pub struct GcController;

impl Controller for GcController {}

fn stale_handle() -> EngineError {
    EngineError::InvalidOperation("the resource handle is stale or unknown".to_string())
}

fn live_entry_mut(entries: &mut [GcEntry], handle: ResourceHandle) -> EngineResult<&mut GcEntry> {
    entries
        .get_mut(handle.slot as usize)
        .filter(|entry| entry.alive && entry.generation == handle.generation)
        .ok_or_else(stale_handle)
}

impl GcController {
    /// Starts tracking a resource. Registering the same uuid again gives the same handle.
    /// A new resource starts with 0 users, so `retain` it right away.
    pub fn register(&self, model: &mut GcModel, uuid: AssetUuid, kind: ResourceKind) -> ResourceHandle {
        let data = self.write(model);

        if let Some(slot) = data.entries.iter().position(|entry| entry.alive && entry.uuid == uuid) {
            data.entries[slot].idle_frames = 0;
            return ResourceHandle { slot: slot as u32, generation: data.entries[slot].generation };
        }

        if let Some(slot) = data.free_slots.pop() {
            let entry = &mut data.entries[slot as usize];

            // The generation was already raised when this slot was collected.
            entry.uuid = uuid;
            entry.kind = kind;
            entry.ref_count = 0;
            entry.idle_frames = 0;
            entry.alive = true;

            return ResourceHandle { slot, generation: entry.generation };
        }

        data.entries.push(GcEntry { uuid, kind, generation: 0, ref_count: 0, idle_frames: 0, alive: true });
        ResourceHandle { slot: (data.entries.len() - 1) as u32, generation: 0 }
    }

    /// Someone starts using the resource.
    pub fn retain(&self, model: &mut GcModel, handle: ResourceHandle) -> EngineResult<()> {
        let entry = live_entry_mut(&mut self.write(model).entries, handle)?;

        entry.ref_count += 1;
        entry.idle_frames = 0;
        Ok(())
    }

    /// Someone stops using the resource.
    pub fn release(&self, model: &mut GcModel, handle: ResourceHandle) -> EngineResult<()> {
        let entry = live_entry_mut(&mut self.write(model).entries, handle)?;

        if entry.ref_count == 0 {
            return Err(EngineError::InvalidOperation("released more times than retained".to_string()));
        }

        entry.ref_count -= 1;
        Ok(())
    }

    pub fn set_grace_frames(&self, model: &mut GcModel, frames: u32) {
        self.write(model).grace_frames = frames;
    }

    /// Call once per frame. Counts how long unused resources have been idle.
    pub fn begin_frame(&self, model: &mut GcModel) {
        let data = self.write(model);

        data.frame += 1;

        for entry in data.entries.iter_mut().filter(|entry| entry.alive && entry.ref_count == 0) {
            entry.idle_frames += 1;
        }
    }

    /// Collects resources that were unused for longer than the grace period.
    /// Returns what was collected so the owner can free the real memory.
    pub fn collect(&self, model: &mut GcModel) -> Vec<CollectedResource> {
        let data = self.write(model);
        let mut collected = Vec::new();

        for (slot, entry) in data.entries.iter_mut().enumerate() {
            if entry.alive && entry.ref_count == 0 && entry.idle_frames >= data.grace_frames {
                entry.alive = false;
                entry.generation += 1; // old handles are now stale
                data.free_slots.push(slot as u32);
                collected.push(CollectedResource { uuid: entry.uuid, kind: entry.kind });
            }
        }

        collected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gc::query::{alive_count, is_alive};
    use crate::identity::{IdController, IdRegistryModel};
    use crate::mivc::Model;

    fn new_uuid() -> AssetUuid {
        IdController.new_uuid(&mut IdRegistryModel::default())
    }

    #[test]
    fn unused_resource_is_collected_after_the_grace_period() {
        let mut model = GcModel::default();
        GcController.set_grace_frames(&mut model, 2);
        let uuid = new_uuid();
        let handle = GcController.register(&mut model, uuid, ResourceKind::Texture);

        GcController.begin_frame(&mut model);
        assert!(GcController.collect(&mut model).is_empty());

        GcController.begin_frame(&mut model);
        let collected = GcController.collect(&mut model);

        assert_eq!(collected, vec![CollectedResource { uuid, kind: ResourceKind::Texture }]);
        assert!(!is_alive(model.data(), handle));
    }

    #[test]
    fn a_resource_in_use_is_never_collected() {
        let mut model = GcModel::default();
        GcController.set_grace_frames(&mut model, 1);
        let handle = GcController.register(&mut model, new_uuid(), ResourceKind::Mesh);
        GcController.retain(&mut model, handle).unwrap();

        for _ in 0..10 {
            GcController.begin_frame(&mut model);
            assert!(GcController.collect(&mut model).is_empty());
        }

        assert_eq!(alive_count(model.data()), 1);
    }

    #[test]
    fn registering_twice_gives_the_same_handle() {
        let mut model = GcModel::default();
        let uuid = new_uuid();

        let first = GcController.register(&mut model, uuid, ResourceKind::Audio);
        let second = GcController.register(&mut model, uuid, ResourceKind::Audio);

        assert_eq!(first, second);
    }

    #[test]
    fn a_reused_slot_does_not_accept_old_handles() {
        let mut model = GcModel::default();
        GcController.set_grace_frames(&mut model, 1);
        let old = GcController.register(&mut model, new_uuid(), ResourceKind::Texture);

        GcController.begin_frame(&mut model);
        GcController.collect(&mut model);

        let new = GcController.register(&mut model, new_uuid(), ResourceKind::Texture);

        assert_eq!(old.slot, new.slot);
        assert_ne!(old.generation, new.generation);
        assert!(GcController.retain(&mut model, old).is_err());
        assert!(GcController.retain(&mut model, new).is_ok());
    }

    #[test]
    fn releasing_too_often_is_an_error() {
        let mut model = GcModel::default();
        let handle = GcController.register(&mut model, new_uuid(), ResourceKind::Shader);

        assert!(GcController.release(&mut model, handle).is_err());
    }
}
