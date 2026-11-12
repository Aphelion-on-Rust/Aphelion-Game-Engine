use super::model::ControlModel;
use super::query::summarize;
use crate::identity::AssetUuid;
use crate::mivc::{Model, View};

/// One line of a prefab's revision list.
#[derive(Clone, Debug, PartialEq)]
pub struct RevisionRow {
    pub number: u32,
    pub saved_on: String,
    pub message: String,
    /// Like "+1 object, 2 edits".
    pub summary: String,
    /// Set when this revision was made by reverting another one.
    pub reverts: Option<u32>,
    /// False for the first revision, which creates the prefab.
    pub can_revert: bool,
}

/// Builds the lists of the Aphelion's Control panel. It only reads.
/// It remembers which prefab is selected (screen state).
#[derive(Default)]
pub struct ControlView {
    selected: Option<AssetUuid>,
    prefabs: Vec<(AssetUuid, String)>,
    rows: Vec<RevisionRow>,
}

impl ControlView {
    pub fn select(&mut self, prefab: Option<AssetUuid>) {
        self.selected = prefab;
    }

    pub fn selected(&self) -> Option<AssetUuid> {
        self.selected
    }

    /// The prefabs that have a history: (uuid, name).
    pub fn prefabs(&self) -> &[(AssetUuid, String)] {
        &self.prefabs
    }

    /// The revisions of the selected prefab, newest first.
    pub fn rows(&self) -> &[RevisionRow] {
        &self.rows
    }
}

impl View for ControlView {
    type Models<'a> = &'a ControlModel;

    fn render(&mut self, model: Self::Models<'_>) {
        let data = model.data();

        self.prefabs = data.prefabs.iter().map(|history| (history.prefab, history.name.clone())).collect();

        // Nothing selected, or the selected prefab is gone: take the first one.
        if !self.prefabs.iter().any(|(uuid, _)| Some(*uuid) == self.selected) {
            self.selected = self.prefabs.first().map(|(uuid, _)| *uuid);
        }

        self.rows.clear();

        let Some(history) = data.prefabs.iter().find(|history| Some(history.prefab) == self.selected) else {
            return;
        };
        let first = history.revisions.first().map(|revision| revision.number);

        for revision in history.revisions.iter().rev() {
            self.rows.push(RevisionRow {
                number: revision.number,
                saved_on: revision.saved_on.clone(),
                message: revision.message.clone(),
                summary: summarize(revision).to_text(),
                reverts: revision.reverts,
                can_revert: Some(revision.number) != first,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlController, SaveInfo};
    use crate::identity::IdRegistryModel;
    use crate::mivc::Controller;
    use crate::prefab::PrefabController;
    use crate::scene::SceneController;

    #[test]
    fn rows_are_newest_first_and_the_first_revision_can_not_be_reverted() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let root = SceneController.create_object(&mut scene, &mut ids, "Tree", None).unwrap();
        let mut prefab = PrefabController.create_from_object(&scene, &mut ids, &root, "Tree").unwrap();
        let mut control = ControlModel::default();
        let info = |message| SaveInfo { file: "f", today: "2026-11-10", message, reverts: None };

        ControlController.record_save(&mut control, None, prefab.data(), info("Created"));

        let before = prefab.data().clone();
        PrefabController.write(&mut prefab).objects[0].name = "Oak".to_string();
        ControlController.record_save(&mut control, Some(&before), prefab.data(), info("Renamed"));

        let mut view = ControlView::default();
        view.render(&control);

        assert_eq!(view.prefabs().len(), 1);
        assert_eq!(view.selected(), Some(prefab.data().uuid));

        let rows: Vec<(u32, bool)> = view.rows().iter().map(|row| (row.number, row.can_revert)).collect();

        assert_eq!(rows, vec![(2, true), (1, false)]);
        assert_eq!(view.rows()[0].summary, "1 edit");
        assert_eq!(view.rows()[1].summary, "+1 object");
    }

    #[test]
    fn an_empty_control_has_nothing_selected() {
        let mut view = ControlView::default();

        view.render(&ControlModel::default());

        assert!(view.prefabs().is_empty());
        assert!(view.selected().is_none());
        assert!(view.rows().is_empty());
    }
}
