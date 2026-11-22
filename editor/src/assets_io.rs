//! Scanning the Assets folder for asset files (pictures) and decoding them.
//! Disk access only. The models are filled by the callers, through their controllers.
//!
//! Every asset file gets a `<file>.meta.yaml` with its uuid the first time it is seen. The uuid never
//! changes after that. If a file is copied together with its meta file, two assets would share a uuid:
//! one of them gets a new one and a warning is reported.

use flux_core::error::{EngineError, EngineResult};
use flux_core::project::layout;
use flux_core::storage::yaml::{load_yaml, save_yaml};
use flux_engine::assets::{AssetEntry, AssetKind, AssetMetaData, TextureFilter};
use flux_engine::identity::{AssetUuid, IdController, IdRegistryModel};
use flux_grp::frame::TextureImage;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(folder: &Path, out: &mut Vec<PathBuf>) -> EngineResult<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }

    Ok(())
}

pub fn load_meta(asset_file: &Path) -> EngineResult<AssetMetaData> {
    load_yaml(&layout::meta_file(asset_file))
}

pub fn save_meta(asset_file: &Path, meta: &AssetMetaData) -> EngineResult<()> {
    save_yaml(&layout::meta_file(asset_file), meta)
}

/// Finds every asset file under `Assets`, makes missing meta files, and returns the list.
/// Problems (a meta file that can't be read, a repeated uuid) are added to `warnings`.
pub fn scan_assets(root: &Path, ids: &mut IdRegistryModel, warnings: &mut Vec<String>) -> EngineResult<Vec<AssetEntry>> {
    let folder = layout::assets_folder(root);

    if !folder.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    collect_files(&folder, &mut files)?;
    files.sort();

    let mut seen: HashSet<AssetUuid> = HashSet::new();
    let mut entries = Vec::new();

    for file in files {
        let name = file.file_name().and_then(|name| name.to_str()).unwrap_or_default();

        if name.ends_with(layout::ASSET_META_SUFFIX) {
            continue;
        }

        let Some(kind) = file.extension().and_then(|extension| extension.to_str()).and_then(AssetKind::from_extension) else {
            continue;
        };

        let mut meta = if layout::meta_file(&file).exists() {
            match load_meta(&file) {
                Ok(meta) => meta,
                Err(error) => {
                    warnings.push(format!("{}: {error}", layout::meta_file(&file).display()));
                    continue;
                }
            }
        } else {
            let meta = AssetMetaData { uuid: IdController.new_uuid(ids), filter: TextureFilter::default() };

            save_meta(&file, &meta)?;
            meta
        };

        if seen.insert(meta.uuid) {
            IdController.ensure_uuid(ids, meta.uuid);
        } else {
            meta.uuid = IdController.new_uuid(ids);
            save_meta(&file, &meta)?;
            seen.insert(meta.uuid);
            warnings.push(format!("{} had the same uuid as another asset (a copied file?), it got a new one", file.display()));
        }

        let path = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .components()
            .map(|part| part.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/");

        entries.push(AssetEntry { uuid: meta.uuid, path, kind, filter: meta.filter });
    }

    Ok(entries)
}

/// Reads a picture file into memory.
pub fn load_texture(path: &Path, nearest: bool) -> EngineResult<TextureImage> {
    let picture = image::open(path)
        .map_err(|error| EngineError::InvalidOperation(format!("{}: {error}", path.display())))?
        .to_rgba8();
    let (width, height) = picture.dimensions();

    Ok(TextureImage { width: width as usize, height: height as usize, rgba: picture.into_raw(), nearest })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_folder(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("flux_assets_{name}_{}", std::process::id()));

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Assets").join("Art")).unwrap();
        root
    }

    fn write_picture(path: &Path) {
        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255])).save(path).unwrap();
    }

    #[test]
    fn pictures_get_meta_files_and_other_files_are_ignored() {
        let root = project_folder("scan");
        let mut ids = IdRegistryModel::default();
        let mut warnings = Vec::new();

        write_picture(&root.join("Assets/Art/hero.png"));
        fs::write(root.join("Assets/notes.txt"), "not an asset").unwrap();

        let entries = scan_assets(&root, &mut ids, &mut warnings).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "Assets/Art/hero.png");
        assert!(root.join("Assets/Art/hero.png.meta.yaml").exists());
        assert!(!root.join("Assets/notes.txt.meta.yaml").exists());
        assert!(warnings.is_empty());
        assert!(IdController.is_uuid_used(&ids, &entries[0].uuid));
    }

    #[test]
    fn scanning_again_keeps_the_same_uuid_even_after_the_picture_changes() {
        let root = project_folder("stable");
        let mut ids = IdRegistryModel::default();
        let picture = root.join("Assets/Art/hero.png");

        write_picture(&picture);

        let first = scan_assets(&root, &mut ids, &mut Vec::new()).unwrap();

        image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 255, 0, 255])).save(&picture).unwrap();

        let second = scan_assets(&root, &mut IdRegistryModel::default(), &mut Vec::new()).unwrap();

        assert_eq!(first[0].uuid, second[0].uuid);
    }

    #[test]
    fn a_copied_file_with_its_meta_file_gets_a_new_uuid_and_a_warning() {
        let root = project_folder("copy");
        let mut ids = IdRegistryModel::default();

        write_picture(&root.join("Assets/Art/hero.png"));
        scan_assets(&root, &mut ids, &mut Vec::new()).unwrap();

        fs::copy(root.join("Assets/Art/hero.png"), root.join("Assets/Art/hero copy.png")).unwrap();
        fs::copy(root.join("Assets/Art/hero.png.meta.yaml"), root.join("Assets/Art/hero copy.png.meta.yaml")).unwrap();

        let mut warnings = Vec::new();
        let entries = scan_assets(&root, &mut ids, &mut warnings).unwrap();

        assert_eq!(entries.len(), 2);
        assert_ne!(entries[0].uuid, entries[1].uuid);
        assert_eq!(warnings.len(), 1);

        // The fix was written to disk, so the next scan is quiet.
        let mut again = Vec::new();

        scan_assets(&root, &mut ids, &mut again).unwrap();
        assert!(again.is_empty());
    }

    #[test]
    fn a_broken_meta_file_is_a_warning_and_the_asset_is_left_out() {
        let root = project_folder("broken");
        let mut warnings = Vec::new();

        write_picture(&root.join("Assets/Art/hero.png"));
        fs::write(root.join("Assets/Art/hero.png.meta.yaml"), "nonsense: [").unwrap();

        let entries = scan_assets(&root, &mut IdRegistryModel::default(), &mut warnings).unwrap();

        assert!(entries.is_empty());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn the_filter_in_the_meta_file_is_read_back() {
        let root = project_folder("filter");
        let picture = root.join("Assets/Art/pixel.png");
        let mut ids = IdRegistryModel::default();

        write_picture(&picture);
        scan_assets(&root, &mut ids, &mut Vec::new()).unwrap();

        let mut meta = load_meta(&picture).unwrap();

        meta.filter = TextureFilter::Nearest;
        save_meta(&picture, &meta).unwrap();

        let entries = scan_assets(&root, &mut ids, &mut Vec::new()).unwrap();

        assert_eq!(entries[0].filter, TextureFilter::Nearest);
    }

    #[test]
    fn a_picture_is_decoded_into_red_green_blue_alpha_bytes() {
        let root = project_folder("decode");
        let picture = root.join("Assets/Art/hero.png");

        write_picture(&picture);

        let texture = load_texture(&picture, true).unwrap();

        assert_eq!((texture.width, texture.height, texture.rgba.len()), (2, 2, 16));
        assert_eq!(&texture.rgba[0..4], &[255, 0, 0, 255]);
        assert!(texture.nearest);
        assert!(load_texture(&root.join("Assets/missing.png"), false).is_err());
    }
}
