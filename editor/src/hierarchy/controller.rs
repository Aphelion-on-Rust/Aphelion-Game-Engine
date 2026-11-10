use super::model::HierarchyUiModel;
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::identity::{AssetUuid, IdRegistryModel};
use flux_engine::mivc::{Controller, Model};
use flux_engine::scene::{query, SceneController, SceneModel};

pub struct HierarchyController;

impl Controller for HierarchyController {}

impl HierarchyController {
    /// Click on the arrow: open or close an object.
    pub fn toggle_expanded(&self, ui: &mut HierarchyUiModel, object: AssetUuid) {
        let data = self.write(ui);

        if !data.expanded.remove(&object) {
            data.expanded.insert(object);
        }
    }

    /// Plain click: select only this object.
    pub fn select_only(&self, ui: &mut HierarchyUiModel, object: AssetUuid) {
        self.write(ui).selected = vec![object];
    }

    /// Ctrl+click: add the object to the selection, or remove it if it is already selected.
    pub fn toggle_select(&self, ui: &mut HierarchyUiModel, object: AssetUuid) {
        let data = self.write(ui);

        match data.selected.iter().position(|selected| *selected == object) {
            Some(position) => {
                data.selected.remove(position);
            }
            None => data.selected.push(object),
        }
    }

    pub fn clear_selection(&self, ui: &mut HierarchyUiModel) {
        self.write(ui).selected.clear();
    }

    /// Opens every parent of the object so its row is visible.
    pub fn expand_to_show(&self, ui: &mut HierarchyUiModel, scene: &SceneModel, object: &AssetUuid) {
        let objects = &scene.data().objects;
        let mut current = query::find_object(objects, object).and_then(|found| found.parent);
        let data = self.write(ui);

        while let Some(parent) = current {
            data.expanded.insert(parent);
            current = query::find_object(objects, &parent).and_then(|found| found.parent);
        }
    }

    /// Makes a new object, selects it, and opens its parent so it can be seen.
    pub fn create_object(
        &self,
        scene: &mut SceneModel,
        ids: &mut IdRegistryModel,
        ui: &mut HierarchyUiModel,
        name: &str,
        parent: Option<AssetUuid>,
    ) -> EngineResult<AssetUuid> {
        let uuid = SceneController.create_object(scene, ids, name, parent)?;

        if let Some(parent_uuid) = parent {
            self.write(ui).expanded.insert(parent_uuid);
        }

        self.select_only(ui, uuid);
        Ok(uuid)
    }

    /// Drag and drop: move an object under another one (or to the top with None).
    pub fn reparent(
        &self,
        scene: &mut SceneModel,
        ui: &mut HierarchyUiModel,
        object: &AssetUuid,
        new_parent: Option<AssetUuid>,
    ) -> EngineResult<()> {
        SceneController.set_parent(scene, object, new_parent)?;

        if let Some(parent_uuid) = new_parent {
            self.write(ui).expanded.insert(parent_uuid);
        }

        Ok(())
    }

    /// Deletes everything selected (with their children), then clears the selection.
    pub fn delete_selected(&self, scene: &mut SceneModel, ui: &mut HierarchyUiModel) -> EngineResult<()> {
        let doomed = ui.data().selected.clone();

        for uuid in doomed {
            match SceneController.destroy_object(scene, &uuid) {
                // A child of something already deleted is gone too, that is fine.
                Ok(()) | Err(EngineError::ObjectNotFound(_)) => {}
                Err(other) => return Err(other),
            }
        }

        let objects = &scene.data().objects;
        let data = self.write(ui);

        data.selected.clear();
        data.expanded.retain(|uuid| query::find_object(objects, uuid).is_some());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::view::HierarchyView;
    use flux_engine::mivc::View;

    struct Setup {
        scene: SceneModel,
        ids: IdRegistryModel,
        ui: HierarchyUiModel,
        parent: AssetUuid,
        child: AssetUuid,
        other: AssetUuid,
    }

    /// Parent > Child, and a separate Other.
    fn setup() -> Setup {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let mut ui = HierarchyUiModel::default();
        let parent = HierarchyController.create_object(&mut scene, &mut ids, &mut ui, "Parent", None).unwrap();
        let child = HierarchyController.create_object(&mut scene, &mut ids, &mut ui, "Child", Some(parent)).unwrap();
        let other = HierarchyController.create_object(&mut scene, &mut ids, &mut ui, "Other", None).unwrap();

        HierarchyController.clear_selection(&mut ui);
        HierarchyController.toggle_expanded(&mut ui, parent); // creating a child opened it, close again

        Setup { scene, ids, ui, parent, child, other }
    }

    fn row_names(setup: &Setup) -> Vec<(String, u32)> {
        let mut view = HierarchyView::default();

        view.render((&setup.scene, &setup.ui));
        view.rows().iter().map(|row| (row.name.clone(), row.depth)).collect()
    }

    #[test]
    fn collapsed_parents_hide_their_children() {
        let setup = setup();

        assert_eq!(row_names(&setup), vec![("Parent".to_string(), 0), ("Other".to_string(), 0)]);
    }

    #[test]
    fn expanding_shows_children_indented() {
        let mut setup = setup();

        HierarchyController.toggle_expanded(&mut setup.ui, setup.parent);

        assert_eq!(
            row_names(&setup),
            vec![("Parent".to_string(), 0), ("Child".to_string(), 1), ("Other".to_string(), 0)]
        );
    }

    #[test]
    fn ctrl_click_builds_a_selection() {
        let mut setup = setup();

        HierarchyController.select_only(&mut setup.ui, setup.parent);
        HierarchyController.toggle_select(&mut setup.ui, setup.other);
        assert_eq!(setup.ui.data().selected, vec![setup.parent, setup.other]);

        HierarchyController.toggle_select(&mut setup.ui, setup.parent);
        assert_eq!(setup.ui.data().selected, vec![setup.other]);
    }

    #[test]
    fn selected_and_greyed_out_rows_are_flagged() {
        let mut setup = setup();

        HierarchyController.toggle_expanded(&mut setup.ui, setup.parent);
        HierarchyController.select_only(&mut setup.ui, setup.child);
        SceneController.set_active(&mut setup.scene, &setup.parent, false).unwrap();

        let mut view = HierarchyView::default();
        view.render((&setup.scene, &setup.ui));
        let child_row = &view.rows()[1];

        assert!(child_row.selected);
        assert!(child_row.active);
        assert!(!child_row.active_in_hierarchy);
    }

    #[test]
    fn dragging_onto_another_object_reparents_and_opens_it() {
        let mut setup = setup();

        HierarchyController.reparent(&mut setup.scene, &mut setup.ui, &setup.other, Some(setup.parent)).unwrap();

        assert!(setup.ui.data().expanded.contains(&setup.parent));
        assert_eq!(
            row_names(&setup),
            vec![
                ("Parent".to_string(), 0),
                ("Child".to_string(), 1),
                ("Other".to_string(), 1),
            ]
        );
    }

    #[test]
    fn dragging_a_parent_into_its_child_is_refused() {
        let mut setup = setup();

        let result = HierarchyController.reparent(&mut setup.scene, &mut setup.ui, &setup.parent, Some(setup.child));

        assert!(result.is_err());
    }

    #[test]
    fn deleting_a_selected_parent_and_its_selected_child_works() {
        let mut setup = setup();

        HierarchyController.select_only(&mut setup.ui, setup.parent);
        HierarchyController.toggle_select(&mut setup.ui, setup.child);
        HierarchyController.delete_selected(&mut setup.scene, &mut setup.ui).unwrap();

        assert_eq!(row_names(&setup), vec![("Other".to_string(), 0)]);
        assert!(setup.ui.data().selected.is_empty());
        let _ = &setup.ids;
    }

    #[test]
    fn expand_to_show_opens_every_parent() {
        let mut setup = setup();

        HierarchyController.expand_to_show(&mut setup.ui, &setup.scene, &setup.child);

        assert!(setup.ui.data().expanded.contains(&setup.parent));
    }
}
