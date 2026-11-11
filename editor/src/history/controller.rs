use super::interface::{HistoryData, HistoryEntry, OpenGroup};
use super::model::HistoryModel;
use flux_engine::mivc::Controller;
use flux_engine::scene::{SceneData, SceneModel};

pub struct HistoryController;

impl Controller for HistoryController {}

/// Adds a step to the undo list and drops the oldest ones beyond the limit.
fn push_step(data: &mut HistoryData, entry: HistoryEntry) {
    data.redo.clear(); // a new edit makes the redo steps meaningless
    data.undo.push(entry);

    let too_many = data.undo.len().saturating_sub(data.limit);

    if too_many > 0 {
        data.undo.drain(..too_many);
    }
}

impl HistoryController {
    /// Remembers one finished edit. `before` is the scene as it was before the edit.
    /// If `merge_key` is the same as the previous step's, the two become one step
    /// (the older `before` is kept).
    pub fn record(&self, model: &mut HistoryModel, label: &str, merge_key: Option<String>, before: SceneData) {
        let data = self.write(model);

        // While a group is open (a drag), everything inside it is one step recorded at the end.
        if data.open_group.is_some() {
            return;
        }

        if let (Some(key), Some(last)) = (&merge_key, data.undo.last()) {
            if last.merge_key.as_ref() == Some(key) {
                data.redo.clear();
                return;
            }
        }

        push_step(data, HistoryEntry { label: label.to_string(), merge_key, scene: before });
    }

    /// A step that takes a while is starting (the mouse went down on a number).
    pub fn begin_group(&self, model: &mut HistoryModel, label: &str, before: SceneData) {
        self.write(model).open_group = Some(OpenGroup { label: label.to_string(), before });
    }

    /// The step ended. It is recorded only if the scene really changed.
    pub fn commit_group(&self, model: &mut HistoryModel, current: &SceneData) {
        let data = self.write(model);

        if let Some(group) = data.open_group.take() {
            if &group.before != current {
                push_step(data, HistoryEntry { label: group.label, merge_key: None, scene: group.before });
            }
        }
    }

    /// The step was cancelled (Escape). Nothing is recorded.
    pub fn cancel_group(&self, model: &mut HistoryModel) {
        self.write(model).open_group = None;
    }

    /// Puts the previous scene back. Returns what was undone, or None if there is nothing to undo.
    pub fn undo(&self, model: &mut HistoryModel, scene: &mut SceneModel) -> Option<String> {
        let data = self.write(model);
        let entry = data.undo.pop()?;
        let current = std::mem::replace(self.write(scene), entry.scene);

        data.redo.push(HistoryEntry { label: entry.label.clone(), merge_key: None, scene: current });
        Some(entry.label)
    }

    /// Undoes the undo. Returns what was redone, or None if there is nothing to redo.
    pub fn redo(&self, model: &mut HistoryModel, scene: &mut SceneModel) -> Option<String> {
        let data = self.write(model);
        let entry = data.redo.pop()?;
        let current = std::mem::replace(self.write(scene), entry.scene);

        data.undo.push(HistoryEntry { label: entry.label.clone(), merge_key: None, scene: current });
        Some(entry.label)
    }

    /// Forgets everything (a different scene was opened).
    pub fn clear(&self, model: &mut HistoryModel) {
        let data = self.write(model);

        data.undo.clear();
        data.redo.clear();
        data.open_group = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::query::{can_redo, can_undo, next_undo_label};
    use flux_engine::identity::IdRegistryModel;
    use flux_engine::mivc::Model;
    use flux_engine::scene::SceneController;

    struct World {
        scene: SceneModel,
        ids: IdRegistryModel,
        history: HistoryModel,
    }

    fn world() -> World {
        let mut ids = IdRegistryModel::default();
        let scene = SceneController.create_scene(&mut ids, "Test");

        World { scene, ids, history: HistoryModel::default() }
    }

    /// Adds an object and records the step, like the editor does.
    fn add_object(world: &mut World, name: &str) {
        let before = world.scene.data().clone();

        SceneController.create_object(&mut world.scene, &mut world.ids, name, None).unwrap();
        HistoryController.record(&mut world.history, "Create object", None, before);
    }

    #[test]
    fn undo_and_redo_restore_the_scene() {
        let mut world = world();

        add_object(&mut world, "A");
        assert_eq!(world.scene.data().objects.len(), 1);

        assert_eq!(HistoryController.undo(&mut world.history, &mut world.scene).as_deref(), Some("Create object"));
        assert!(world.scene.data().objects.is_empty());
        assert!(can_redo(world.history.data()));

        HistoryController.redo(&mut world.history, &mut world.scene);
        assert_eq!(world.scene.data().objects.len(), 1);
        assert_eq!(world.scene.data().objects[0].name, "A");
    }

    #[test]
    fn nothing_to_undo_or_redo_is_not_an_error() {
        let mut world = world();

        assert!(HistoryController.undo(&mut world.history, &mut world.scene).is_none());
        assert!(HistoryController.redo(&mut world.history, &mut world.scene).is_none());
        assert!(!can_undo(world.history.data()));
    }

    #[test]
    fn a_new_edit_after_an_undo_removes_the_redo_steps() {
        let mut world = world();

        add_object(&mut world, "A");
        HistoryController.undo(&mut world.history, &mut world.scene);
        add_object(&mut world, "B");

        assert!(!can_redo(world.history.data()));
    }

    #[test]
    fn steps_with_the_same_key_become_one_step() {
        let mut world = world();
        let original = world.scene.data().clone();

        for _ in 0..3 {
            HistoryController.record(&mut world.history, "Rename", Some("rename:x".to_string()), original.clone());
        }

        assert_eq!(world.history.data().undo.len(), 1);

        HistoryController.record(&mut world.history, "Rename", Some("rename:y".to_string()), original);
        assert_eq!(world.history.data().undo.len(), 2);
    }

    #[test]
    fn the_oldest_steps_are_dropped_beyond_the_limit() {
        let mut world = world();

        world.history = HistoryModel::new(HistoryData { limit: 3, ..Default::default() });

        for index in 0..5 {
            add_object(&mut world, &format!("Object {index}"));
        }

        assert_eq!(world.history.data().undo.len(), 3);
    }

    #[test]
    fn a_group_is_one_step_and_an_unchanged_group_is_none() {
        let mut world = world();

        HistoryController.begin_group(&mut world.history, "Change value", world.scene.data().clone());
        // Edits inside a group are not recorded one by one.
        add_object(&mut world, "A");
        add_object(&mut world, "B");
        HistoryController.commit_group(&mut world.history, world.scene.data());

        assert_eq!(world.history.data().undo.len(), 1);
        assert_eq!(next_undo_label(world.history.data()), Some("Change value"));

        HistoryController.begin_group(&mut world.history, "Nothing", world.scene.data().clone());
        HistoryController.commit_group(&mut world.history, world.scene.data());
        assert_eq!(world.history.data().undo.len(), 1);

        HistoryController.begin_group(&mut world.history, "Cancelled", world.scene.data().clone());
        HistoryController.cancel_group(&mut world.history);
        assert!(world.history.data().open_group.is_none());
    }

    #[test]
    fn clearing_forgets_everything() {
        let mut world = world();

        add_object(&mut world, "A");
        HistoryController.undo(&mut world.history, &mut world.scene);
        HistoryController.clear(&mut world.history);

        assert!(!can_undo(world.history.data()));
        assert!(!can_redo(world.history.data()));
    }
}
