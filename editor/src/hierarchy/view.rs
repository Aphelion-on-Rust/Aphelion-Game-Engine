use super::interface::HierarchyRow;
use super::model::HierarchyUiModel;
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::{Model, View};
use flux_engine::scene::{query, GameObjectData, SceneModel};

/// Builds the rows of the hierarchy panel. It only reads, it can't change the scene.
#[derive(Default)]
pub struct HierarchyView {
    rows: Vec<HierarchyRow>,
}

impl HierarchyView {
    pub fn rows(&self) -> &[HierarchyRow] {
        &self.rows
    }

    /// Adds the object, then its children if it is expanded.
    fn add_rows(
        &mut self,
        objects: &[GameObjectData],
        uuid: &AssetUuid,
        depth: u32,
        ui: &HierarchyUiModel,
    ) {
        let Some(object) = query::find_object(objects, uuid) else {
            return;
        };

        let expanded = ui.data().expanded.contains(uuid);

        self.rows.push(HierarchyRow {
            uuid: *uuid,
            name: object.name.clone(),
            depth,
            has_children: !object.children.is_empty(),
            expanded,
            selected: ui.data().selected.contains(uuid),
            active: object.active,
            active_in_hierarchy: query::is_active_in_hierarchy(objects, uuid),
            is_prefab_root: object.prefab_source.is_some(),
            renaming: ui.data().renaming == Some(*uuid),
        });

        if expanded {
            for child in &object.children {
                self.add_rows(objects, child, depth + 1, ui);
            }
        }
    }
}

impl View for HierarchyView {
    type Models<'a> = (&'a SceneModel, &'a HierarchyUiModel);

    fn render(&mut self, models: Self::Models<'_>) {
        let (scene, ui) = models;
        let objects = &scene.data().objects;
        let roots: Vec<AssetUuid> = query::root_objects(objects).iter().map(|object| object.uuid).collect();

        self.rows.clear();

        for root in &roots {
            self.add_rows(objects, root, 0, ui);
        }
    }
}
