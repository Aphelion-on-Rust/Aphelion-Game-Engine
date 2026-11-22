//! The asset actions that need the disk: look for pictures again, change how a texture is read.
//! The app calls these for the `IoRequest`s the command layer queued (see `prefab_flow::run`).

use super::models::EditorModels;
use super::prefab_flow::project_root;
use crate::{assets_io, project_io};
use flux_engine::assets::{query, AssetLibraryController, TextureFilter};
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::Model;

/// Scans the Assets folder: new pictures get a meta file (and a uuid), the list is replaced.
pub fn refresh(models: &mut EditorModels) -> EngineResult<String> {
    let root = project_root(models)?;
    let mut warnings = Vec::new();
    let entries = assets_io::scan_assets(&root, &mut models.ids, &mut warnings)?;
    let count = entries.len();

    AssetLibraryController.set_all(&mut models.assets, entries);
    // New assets have new uuids, so the registry is saved right away.
    project_io::save_ids(&root, &models.ids)?;

    Ok(match warnings.first() {
        Some(first) => format!("Found {count} assets, {} problem(s): {first}", warnings.len()),
        None => format!("Found {count} assets"),
    })
}

/// Changes how a texture is read and writes it to the asset's meta file.
pub fn set_texture_filter(models: &mut EditorModels, asset: &AssetUuid, filter: TextureFilter) -> EngineResult<String> {
    let root = project_root(models)?;
    let entry = query::find_by_uuid(models.assets.data(), asset)
        .cloned()
        .ok_or_else(|| EngineError::InvalidOperation("that asset is not in the project".to_string()))?;
    let file = root.join(&entry.path);
    let mut meta = assets_io::load_meta(&file)?;

    meta.filter = filter;
    assets_io::save_meta(&file, &meta)?;
    AssetLibraryController.set_filter(&mut models.assets, asset, filter)?;

    Ok(format!("{} is now read {}", entry.path, if filter == TextureFilter::Nearest { "with hard pixels" } else { "smoothly" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_io::open_project;
    use flux_core::project::{disk, GraphicsPipeline, ProjectController};
    use std::fs;

    fn open(name: &str) -> (EditorModels, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("flux_asset_flow_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let project = ProjectController.new_project("Test", "0.1.2-alpha", GraphicsPipeline::Simple, false, "2026-11-17").unwrap();

        disk::create_project(&root, project.data()).unwrap();
        (EditorModels::from_opened(open_project(&root).unwrap()), root)
    }

    #[test]
    fn refreshing_finds_pictures_added_while_the_editor_is_open() {
        let (mut models, root) = open("refresh");

        assert!(models.assets.data().assets.is_empty());

        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255])).save(root.join("Assets").join("new.png")).unwrap();

        let message = refresh(&mut models).unwrap();

        assert!(message.contains("1 assets"));
        assert_eq!(models.assets.data().assets.len(), 1);
    }

    #[test]
    fn the_texture_filter_is_written_to_the_meta_file_and_survives_reopening() {
        let (mut models, root) = open("filter");

        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255])).save(root.join("Assets").join("pixel.png")).unwrap();
        refresh(&mut models).unwrap();

        let asset = models.assets.data().assets[0].uuid;

        set_texture_filter(&mut models, &asset, TextureFilter::Nearest).unwrap();

        assert_eq!(models.assets.data().assets[0].filter, TextureFilter::Nearest);
        assert_eq!(open_project(&root).unwrap().assets.data().assets[0].filter, TextureFilter::Nearest);
    }

    #[test]
    fn an_unknown_asset_can_not_be_changed() {
        let (mut models, _root) = open("unknown");
        let unknown = flux_engine::identity::IdController.new_uuid(&mut models.ids);

        assert!(set_texture_filter(&mut models, &unknown, TextureFilter::Nearest).is_err());
    }
}
