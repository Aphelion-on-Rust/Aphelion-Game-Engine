use super::interface::{DropZone, LayoutData, PanelKind};
use super::model::LayoutModel;
use super::{query, tree};
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::mivc::{Controller, Model};

pub struct LayoutController;

impl Controller for LayoutController {}

fn not_open(panel: PanelKind) -> EngineError {
    EngineError::InvalidOperation(format!("the {panel:?} panel is not open"))
}

impl LayoutController {
    pub fn reset(&self, model: &mut LayoutModel) {
        *self.write(model) = LayoutData::default();
    }

    /// Drag `panel` and drop it on `target`'s group.
    /// Edges make a new split, Center adds a tab.
    pub fn move_panel(
        &self,
        model: &mut LayoutModel,
        panel: PanelKind,
        target: PanelKind,
        zone: DropZone,
    ) -> EngineResult<()> {
        let data = model.data();

        if !query::is_open(data, panel) {
            return Err(not_open(panel));
        }

        if !query::is_open(data, target) {
            return Err(not_open(target));
        }

        if panel == target {
            return Ok(());
        }

        // Take the panel out first, then place it. The target is still in the tree
        // because it is a different panel.
        let without = tree::remove(data.root.clone(), panel).ok_or_else(|| not_open(target))?;

        self.write(model).root = tree::insert(without, target, panel, zone);
        Ok(())
    }

    /// Closes a panel. The last panel can't be closed, the editor needs something to show.
    pub fn close_panel(&self, model: &mut LayoutModel, panel: PanelKind) -> EngineResult<()> {
        if !query::is_open(model.data(), panel) {
            return Err(not_open(panel));
        }

        match tree::remove(model.data().root.clone(), panel) {
            Some(root) => {
                self.write(model).root = root;
                Ok(())
            }
            None => Err(EngineError::InvalidOperation("the last panel can't be closed".to_string())),
        }
    }

    /// Opens a panel (as a tab in the first group) or just shows it if it is already open.
    pub fn open_panel(&self, model: &mut LayoutModel, panel: PanelKind) {
        let root = &mut self.write(model).root;

        if !tree::activate(root, panel) {
            tree::add_to_first_group(root, panel);
        }
    }

    pub fn set_active_tab(&self, model: &mut LayoutModel, panel: PanelKind) -> EngineResult<()> {
        if tree::activate(&mut self.write(model).root, panel) {
            Ok(())
        } else {
            Err(not_open(panel))
        }
    }

    /// Moves the divider of one split. `path` walks down from the root:
    /// false = first child, true = second child. The ratio stays between 0.1 and 0.9.
    pub fn set_split_ratio(&self, model: &mut LayoutModel, path: &[bool], ratio: f32) -> EngineResult<()> {
        match tree::split_ratio_at(&mut self.write(model).root, path) {
            Some(slot) => {
                *slot = ratio.clamp(0.1, 0.9);
                Ok(())
            }
            None => Err(EngineError::InvalidOperation("there is no split at that path".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::interface::DockNode;
    use flux_engine::storage::yaml::{from_yaml, to_yaml};

    fn assert_no_duplicates(model: &LayoutModel) {
        let mut panels = query::all_panels(model.data());
        let count = panels.len();

        panels.sort();
        panels.dedup();
        assert_eq!(panels.len(), count);
    }

    #[test]
    fn default_layout_has_the_seven_main_panels() {
        let model = LayoutModel::default();

        assert_eq!(query::all_panels(model.data()).len(), 7);
        assert_no_duplicates(&model);
    }

    #[test]
    fn dropping_in_the_center_makes_a_tab() {
        let mut model = LayoutModel::default();

        LayoutController.move_panel(&mut model, PanelKind::Inspector, PanelKind::Hierarchy, DropZone::Center).unwrap();

        assert_no_duplicates(&model);
        assert_eq!(query::all_panels(model.data()).len(), 7);

        // Inspector's old group is gone, so the first group now holds both.
        match &model.data().root {
            DockNode::Split { first, .. } => assert_eq!(
                **first,
                DockNode::Tabs { panels: vec![PanelKind::Hierarchy, PanelKind::Inspector], active: 1 }
            ),
            other => panic!("expected a split, got {other:?}"),
        }
    }

    #[test]
    fn dropping_on_an_edge_makes_a_new_split() {
        let mut model = LayoutModel::default();

        LayoutController.move_panel(&mut model, PanelKind::Console, PanelKind::Scene, DropZone::Right).unwrap();

        assert_no_duplicates(&model);
        assert!(query::is_open(model.data(), PanelKind::Console));
    }

    #[test]
    fn the_last_panel_cannot_be_closed() {
        let mut model = LayoutModel::default();

        for panel in [
            PanelKind::Hierarchy,
            PanelKind::Scene,
            PanelKind::Game,
            PanelKind::Inspector,
            PanelKind::Project,
            PanelKind::History,
        ] {
            LayoutController.close_panel(&mut model, panel).unwrap();
        }

        assert!(LayoutController.close_panel(&mut model, PanelKind::Console).is_err());
        assert_eq!(query::all_panels(model.data()), vec![PanelKind::Console]);
    }

    #[test]
    fn open_panel_adds_a_closed_panel_and_shows_an_open_one() {
        let mut model = LayoutModel::default();

        LayoutController.open_panel(&mut model, PanelKind::Terrain);
        assert!(query::is_open(model.data(), PanelKind::Terrain));

        LayoutController.open_panel(&mut model, PanelKind::Terrain);
        assert_no_duplicates(&model);
    }

    #[test]
    fn split_ratio_is_kept_between_10_and_90_percent() {
        let mut model = LayoutModel::default();

        LayoutController.set_split_ratio(&mut model, &[], 5.0).unwrap();

        match &model.data().root {
            DockNode::Split { ratio, .. } => assert_eq!(*ratio, 0.9),
            other => panic!("expected a split, got {other:?}"),
        }

        assert!(LayoutController.set_split_ratio(&mut model, &[false], 0.5).is_err());
    }

    #[test]
    fn layout_survives_a_yaml_round_trip() {
        let model = LayoutModel::default();
        let loaded: LayoutData = from_yaml(&to_yaml(model.data()).unwrap()).unwrap();

        assert_eq!(&loaded, model.data());
    }
}
