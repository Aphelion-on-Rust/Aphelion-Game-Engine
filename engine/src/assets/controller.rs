use super::interface::{AssetEntry, TextureFilter};
use super::model::AssetLibraryModel;
use super::query;
use crate::error::{EngineError, EngineResult};
use crate::identity::AssetUuid;
use crate::mivc::{Controller, Model};

pub struct AssetLibraryController;

impl Controller for AssetLibraryController {}

impl AssetLibraryController {
    /// Replaces the whole list (after scanning the project). It is kept sorted by path.
    pub fn set_all(&self, model: &mut AssetLibraryModel, mut assets: Vec<AssetEntry>) {
        assets.sort_by(|a, b| a.path.cmp(&b.path));
        self.write(model).assets = assets;
    }

    /// Changes how a texture is read. (The caller also writes it to the asset's meta file.)
    pub fn set_filter(&self, model: &mut AssetLibraryModel, asset: &AssetUuid, filter: TextureFilter) -> EngineResult<()> {
        if query::find_by_uuid(model.data(), asset).is_none() {
            return Err(EngineError::InvalidOperation("that asset is not in the project".to_string()));
        }

        if let Some(entry) = self.write(model).assets.iter_mut().find(|entry| entry.uuid == *asset) {
            entry.filter = filter;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::interface::{AssetKind, AssetMetaData};
    use crate::identity::{IdController, IdRegistryModel};
    use crate::storage::yaml::{from_yaml, to_yaml};

    fn entry(path: &str, ids: &mut IdRegistryModel) -> AssetEntry {
        AssetEntry {
            uuid: IdController.new_uuid(ids),
            path: path.to_string(),
            kind: AssetKind::Texture,
            filter: TextureFilter::Linear,
        }
    }

    #[test]
    fn the_extension_decides_the_kind_of_asset() {
        assert_eq!(AssetKind::from_extension("png"), Some(AssetKind::Texture));
        assert_eq!(AssetKind::from_extension("JPG"), Some(AssetKind::Texture));
        assert_eq!(AssetKind::from_extension("jpeg"), Some(AssetKind::Texture));
        assert_eq!(AssetKind::from_extension("txt"), None);
        assert_eq!(AssetKind::from_extension("yaml"), None);
    }

    #[test]
    fn the_list_is_sorted_and_assets_are_found_by_uuid_and_path() {
        let mut ids = IdRegistryModel::default();
        let mut model = AssetLibraryModel::default();
        let b = entry("Assets/b.png", &mut ids);
        let a = entry("Assets/a.png", &mut ids);

        AssetLibraryController.set_all(&mut model, vec![b.clone(), a.clone()]);

        assert_eq!(model.data().assets[0].path, "Assets/a.png");
        assert_eq!(query::find_by_uuid(model.data(), &b.uuid).unwrap().path, "Assets/b.png");
        assert_eq!(query::find_by_path(model.data(), "Assets/a.png").unwrap().uuid, a.uuid);
        assert_eq!(query::of_kind(model.data(), AssetKind::Texture).len(), 2);
    }

    #[test]
    fn the_filter_of_a_texture_can_change_but_not_of_an_unknown_asset() {
        let mut ids = IdRegistryModel::default();
        let mut model = AssetLibraryModel::default();
        let sprite = entry("Assets/hero.png", &mut ids);

        AssetLibraryController.set_all(&mut model, vec![sprite.clone()]);
        AssetLibraryController.set_filter(&mut model, &sprite.uuid, TextureFilter::Nearest).unwrap();

        assert_eq!(model.data().assets[0].filter, TextureFilter::Nearest);

        let unknown = IdController.new_uuid(&mut ids);

        assert!(AssetLibraryController.set_filter(&mut model, &unknown, TextureFilter::Nearest).is_err());
    }

    #[test]
    fn a_meta_file_survives_yaml_and_an_old_one_without_a_filter_still_loads() {
        let mut ids = IdRegistryModel::default();
        let meta = AssetMetaData { uuid: IdController.new_uuid(&mut ids), filter: TextureFilter::Nearest };

        assert_eq!(from_yaml::<AssetMetaData>(&to_yaml(&meta).unwrap()).unwrap(), meta);

        let old = format!("format: asset_meta\nversion: 1\ndata:\n  uuid: {}\n", meta.uuid);

        assert_eq!(from_yaml::<AssetMetaData>(&old).unwrap().filter, TextureFilter::Linear);
    }
}
