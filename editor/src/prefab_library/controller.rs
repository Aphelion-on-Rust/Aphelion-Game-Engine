use super::interface::PrefabListing;
use super::model::PrefabLibraryModel;
use flux_engine::mivc::Controller;

pub struct PrefabLibraryController;

impl Controller for PrefabLibraryController {}

impl PrefabLibraryController {
    /// Replaces the whole list (when a project is opened).
    pub fn set_all(&self, model: &mut PrefabLibraryModel, mut listings: Vec<PrefabListing>) {
        listings.sort_by(|a, b| a.name.cmp(&b.name));
        self.write(model).prefabs = listings;
    }

    /// Adds a prefab, or replaces the entry with the same uuid.
    pub fn upsert(&self, model: &mut PrefabLibraryModel, listing: PrefabListing) {
        let data = self.write(model);

        match data.prefabs.iter_mut().find(|existing| existing.uuid == listing.uuid) {
            Some(existing) => *existing = listing,
            None => data.prefabs.push(listing),
        }

        data.prefabs.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefab_library::query::{find_by_name, find_by_uuid};
    use flux_engine::identity::{IdController, IdRegistryModel};
    use flux_engine::mivc::Model;

    fn listing(name: &str, ids: &mut IdRegistryModel) -> PrefabListing {
        PrefabListing {
            name: name.to_string(),
            uuid: IdController.new_uuid(ids),
            file: format!("Assets/Prefabs/{name}.prefab.yaml"),
        }
    }

    #[test]
    fn the_list_stays_sorted_and_an_entry_with_the_same_uuid_is_replaced() {
        let mut ids = IdRegistryModel::default();
        let mut model = PrefabLibraryModel::default();
        let tree = listing("Tree", &mut ids);
        let rock = listing("Rock", &mut ids);

        PrefabLibraryController.upsert(&mut model, tree.clone());
        PrefabLibraryController.upsert(&mut model, rock);
        PrefabLibraryController.upsert(&mut model, PrefabListing { file: "elsewhere".to_string(), ..tree.clone() });

        let names: Vec<&str> = model.data().prefabs.iter().map(|listing| listing.name.as_str()).collect();

        assert_eq!(names, vec!["Rock", "Tree"]);
        assert_eq!(find_by_uuid(model.data(), &tree.uuid).unwrap().file, "elsewhere");
        assert!(find_by_name(model.data(), "Rock").is_some());
    }
}
