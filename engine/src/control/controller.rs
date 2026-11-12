use super::diff::{self, RevertReport};
use super::interface::{PrefabHistoryData, Revision};
use super::model::ControlModel;
use super::query;
use crate::error::{EngineError, EngineResult};
use crate::identity::AssetUuid;
use crate::mivc::{Controller, Model};
use crate::prefab::PrefabData;

pub struct ControlController;

impl Controller for ControlController {}

/// Details of a save, for the history.
pub struct SaveInfo<'a> {
    /// The prefab file inside the project.
    pub file: &'a str,
    pub today: &'a str,
    pub message: &'a str,
    /// Set when the save was made by reverting that revision.
    pub reverts: Option<u32>,
}

fn invalid(message: String) -> EngineError {
    EngineError::InvalidOperation(message)
}

impl ControlController {
    /// Call when a prefab was saved. `before` is the prefab as it was on disk (None for a new one).
    /// Returns the new revision number, or None if nothing changed.
    pub fn record_save(
        &self,
        control: &mut ControlModel,
        before: Option<&PrefabData>,
        after: &PrefabData,
        info: SaveInfo,
    ) -> Option<u32> {
        let nothing = PrefabData { uuid: after.uuid, name: after.name.clone(), objects: Vec::new() };
        let changes = diff::diff(before.unwrap_or(&nothing), after);

        if changes.is_empty() {
            return None;
        }

        let data = self.write(control);
        let index = match data.prefabs.iter().position(|history| history.prefab == after.uuid) {
            Some(index) => index,
            None => {
                data.prefabs.push(PrefabHistoryData {
                    prefab: after.uuid,
                    name: after.name.clone(),
                    file: info.file.to_string(),
                    revisions: Vec::new(),
                });
                data.prefabs.len() - 1
            }
        };

        let history = &mut data.prefabs[index];
        let number = history.revisions.last().map_or(1, |last| last.number + 1);

        history.name = after.name.clone();
        history.file = info.file.to_string();
        history.revisions.push(Revision {
            number,
            saved_on: info.today.to_string(),
            message: info.message.to_string(),
            reverts: info.reverts,
            changes,
        });

        Some(number)
    }

    /// Works out the prefab with one revision undone. It does not change the control model or save
    /// anything: the caller saves the result, which becomes a new revision of its own.
    ///
    /// Only the changes of that revision are undone. A change is skipped if a later edit touched
    /// the same thing. The first revision (which created the prefab) can't be reverted.
    pub fn revert(
        &self,
        control: &ControlModel,
        prefab: &AssetUuid,
        number: u32,
        current: &PrefabData,
    ) -> EngineResult<RevertReport> {
        let history = query::find_history(control.data(), prefab)
            .ok_or_else(|| invalid("this prefab has no history".to_string()))?;
        let revision = history
            .revisions
            .iter()
            .find(|revision| revision.number == number)
            .ok_or_else(|| invalid(format!("there is no revision {number}")))?;

        if history.revisions.first().map(|first| first.number) == Some(number) {
            return Err(invalid("the first revision creates the prefab and can't be reverted".to_string()));
        }

        let report = diff::revert(current, &revision.changes);

        if report.applied == 0 {
            return Err(invalid(format!("nothing could be reverted: {}", report.skipped.join("; "))));
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::interface::PrefabChange;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::prefab::{PrefabController, PrefabModel};
    use crate::scene::{ComponentData, SceneController};
    use crate::storage::yaml::{from_yaml, to_yaml};

    const FILE: &str = "Assets/Prefabs/Tree.prefab.yaml";

    /// A prefab "Tree" with one child "Leaves".
    fn tree() -> (PrefabModel, IdRegistryModel) {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let root = SceneController.create_object(&mut scene, &mut ids, "Tree", None).unwrap();

        SceneController.create_object(&mut scene, &mut ids, "Leaves", Some(root)).unwrap();

        (PrefabController.create_from_object(&scene, &mut ids, &root, "Tree").unwrap(), ids)
    }

    fn save(control: &mut ControlModel, before: Option<&PrefabData>, after: &PrefabData, message: &str) -> Option<u32> {
        ControlController.record_save(
            control,
            before,
            after,
            SaveInfo { file: FILE, today: "2026-11-10", message, reverts: None },
        )
    }

    /// Changes the prefab and returns how it was before.
    fn edit(prefab: &mut PrefabModel, change: impl FnOnce(&mut PrefabData)) -> PrefabData {
        let before = prefab.data().clone();

        change(PrefabController.write(prefab));
        before
    }

    fn root_x(prefab: &PrefabData) -> f32 {
        match &prefab.objects[0].components[0].data {
            ComponentData::Transform(transform) => transform.position.x,
            other => panic!("expected a transform, got {other:?}"),
        }
    }

    fn move_root(prefab: &mut PrefabModel, x: f32) -> PrefabData {
        edit(prefab, |data| {
            if let ComponentData::Transform(transform) = &mut data.objects[0].components[0].data {
                transform.position.x = x;
            }
        })
    }

    #[test]
    fn the_first_save_is_revision_one_and_an_unchanged_save_is_nothing() {
        let (prefab, _ids) = tree();
        let mut control = ControlModel::default();

        assert_eq!(save(&mut control, None, prefab.data(), "Created"), Some(1));
        assert_eq!(save(&mut control, Some(prefab.data()), prefab.data(), "Again"), None);

        let history = &control.data().prefabs[0];

        assert_eq!(history.revisions.len(), 1);
        assert!(history.revisions[0].changes.iter().all(|change| matches!(change, PrefabChange::ObjectAdded { .. })));
    }

    #[test]
    fn reverting_a_revision_undoes_only_what_that_revision_changed() {
        let (mut prefab, _ids) = tree();
        let mut control = ControlModel::default();
        let uuid = prefab.data().uuid;

        save(&mut control, None, prefab.data(), "Created");

        let before = edit(&mut prefab, |data| data.objects[1].name = "Branches".to_string());
        save(&mut control, Some(&before), prefab.data(), "Renamed the child"); // revision 2

        let before = move_root(&mut prefab, 5.0);
        save(&mut control, Some(&before), prefab.data(), "Moved the root"); // revision 3

        let report = ControlController.revert(&control, &uuid, 2, prefab.data()).unwrap();

        assert_eq!(report.prefab.objects[1].name, "Leaves"); // revision 2 is undone
        assert_eq!(root_x(&report.prefab), 5.0); // revision 3 stays
        assert_eq!(report.applied, 1);
        assert!(report.skipped.is_empty());
    }

    #[test]
    fn a_change_that_was_edited_again_is_kept_and_reported() {
        let (mut prefab, _ids) = tree();
        let mut control = ControlModel::default();
        let uuid = prefab.data().uuid;

        save(&mut control, None, prefab.data(), "Created");

        let before = edit(&mut prefab, |data| data.objects[1].name = "A".to_string());
        save(&mut control, Some(&before), prefab.data(), "First rename"); // revision 2

        let before = edit(&mut prefab, |data| data.objects[1].name = "B".to_string());
        save(&mut control, Some(&before), prefab.data(), "Second rename"); // revision 3

        let error = ControlController.revert(&control, &uuid, 2, prefab.data()).err().unwrap();

        assert!(error.to_string().contains("renamed again"));
    }

    #[test]
    fn a_removed_object_comes_back_in_its_place() {
        let (mut prefab, _ids) = tree();
        let mut control = ControlModel::default();
        let uuid = prefab.data().uuid;
        let leaves = prefab.data().objects[1].uuid;

        save(&mut control, None, prefab.data(), "Created");

        let before = edit(&mut prefab, |data| {
            data.objects.remove(1);
            data.objects[0].children.clear();
        });
        save(&mut control, Some(&before), prefab.data(), "Removed the child");

        let report = ControlController.revert(&control, &uuid, 2, prefab.data()).unwrap();

        assert_eq!(report.prefab.objects.len(), 2);
        assert_eq!(report.prefab.objects[1].uuid, leaves);
        assert_eq!(report.prefab.objects[0].children, vec![leaves]);
    }

    #[test]
    fn an_added_object_is_removed_unless_it_has_newer_children() {
        let (mut prefab, mut ids) = tree();
        let mut control = ControlModel::default();
        let uuid = prefab.data().uuid;
        let root = prefab.data().objects[0].uuid;

        save(&mut control, None, prefab.data(), "Created");

        // Revision 2 adds "Fruit".
        let fruit_uuid = IdController.new_uuid(&mut ids);
        let before = edit(&mut prefab, |data| {
            let mut fruit = data.objects[1].clone();

            fruit.uuid = fruit_uuid;
            fruit.name = "Fruit".to_string();
            fruit.parent = Some(root);
            fruit.children.clear();
            fruit.components[0].id = IdController.new_component_id(&mut ids);
            data.objects.push(fruit);
        });
        save(&mut control, Some(&before), prefab.data(), "Added fruit");

        let report = ControlController.revert(&control, &uuid, 2, prefab.data()).unwrap();
        assert_eq!(report.prefab.objects.len(), 2);

        // Revision 3 puts a "Seed" under the fruit. Now the fruit can't simply go.
        let seed_uuid = IdController.new_uuid(&mut ids);
        let before = edit(&mut prefab, |data| {
            let mut seed = data.objects[1].clone();

            seed.uuid = seed_uuid;
            seed.name = "Seed".to_string();
            seed.parent = Some(fruit_uuid);
            seed.children.clear();
            seed.components[0].id = IdController.new_component_id(&mut ids);
            data.objects.push(seed);
        });
        save(&mut control, Some(&before), prefab.data(), "Added a seed");

        assert!(ControlController.revert(&control, &uuid, 2, prefab.data()).is_err());
    }

    #[test]
    fn the_first_revision_can_not_be_reverted() {
        let (prefab, _ids) = tree();
        let mut control = ControlModel::default();
        let uuid = prefab.data().uuid;

        save(&mut control, None, prefab.data(), "Created");

        assert!(ControlController.revert(&control, &uuid, 1, prefab.data()).is_err());
        assert!(ControlController.revert(&control, &uuid, 9, prefab.data()).is_err());
    }

    #[test]
    fn revision_numbers_keep_counting_and_a_revert_is_a_revision_too() {
        let (mut prefab, _ids) = tree();
        let mut control = ControlModel::default();

        save(&mut control, None, prefab.data(), "Created");
        let before = move_root(&mut prefab, 1.0);
        save(&mut control, Some(&before), prefab.data(), "Moved");

        let number = ControlController.record_save(
            &mut control,
            Some(prefab.data()),
            &move_root_copy(prefab.data(), 0.0),
            SaveInfo { file: FILE, today: "2026-11-10", message: "Reverted revision 2", reverts: Some(2) },
        );

        assert_eq!(number, Some(3));
        assert_eq!(control.data().prefabs[0].revisions[2].reverts, Some(2));
    }

    fn move_root_copy(prefab: &PrefabData, x: f32) -> PrefabData {
        let mut copy = prefab.clone();

        if let ComponentData::Transform(transform) = &mut copy.objects[0].components[0].data {
            transform.position.x = x;
        }

        copy
    }

    #[test]
    fn a_history_survives_a_yaml_round_trip() {
        let (mut prefab, _ids) = tree();
        let mut control = ControlModel::default();

        save(&mut control, None, prefab.data(), "Created");

        // Several kinds of change in one save: a rename, a switch, an edited component.
        edit(&mut prefab, |data| {
            data.objects[1].name = "Branches".to_string();
            data.objects[1].active = false;
        });
        let before = move_root(&mut prefab, 2.0);
        save(&mut control, Some(&before), prefab.data(), "Several edits");

        let history = control.data().prefabs[0].clone();
        let loaded: PrefabHistoryData = from_yaml(&to_yaml(&history).unwrap()).unwrap();

        assert_eq!(loaded, history);
    }
}
