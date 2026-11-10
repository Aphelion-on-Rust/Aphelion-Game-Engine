//! Reading and writing the project folder.

use super::controller::ProjectController;
use super::interface::ProjectData;
use super::layout;
use crate::error::{EngineError, EngineResult};
use crate::mivc::Store;
use crate::storage::yaml::{load_yaml, save_yaml};
use std::fs;
use std::path::Path;

/// Makes the folder layout and writes `project.yaml`.
/// Refuses if the folder already holds a project.
pub fn create_project(root: &Path, data: &ProjectData) -> EngineResult<()> {
    if layout::project_file(root).exists() {
        return Err(EngineError::InvalidOperation("that folder already holds a project".to_string()));
    }

    for folder in [
        layout::scenes_folder(root),
        layout::prefabs_folder(root),
        layout::scripts_folder(root),
        layout::settings_folder(root),
        layout::library_folder(root),
    ] {
        fs::create_dir_all(folder)?;
    }

    if data.aphelion_control {
        fs::create_dir_all(layout::aphelion_folder(root))?;
    }

    let ignore_file = root.join(".gitignore");

    if !ignore_file.exists() {
        fs::write(ignore_file, "Library/\n")?;
    }

    save_yaml(&layout::project_file(root), data)
}

pub fn load_project(root: &Path) -> EngineResult<ProjectData> {
    let file = layout::project_file(root);

    if !file.exists() {
        return Err(EngineError::InvalidOperation(format!("there is no {} in {}", layout::PROJECT_FILE, root.display())));
    }

    load_yaml(&file)
}

pub fn save_project(root: &Path, data: &ProjectData) -> EngineResult<()> {
    save_yaml(&layout::project_file(root), data)
}

/// Changes the engine version written in `project.yaml`.
pub fn set_engine_version(root: &Path, version: &str) -> EngineResult<()> {
    let mut model = Store::new(load_project(root)?);

    ProjectController.set_engine_version(&mut model, version)?;
    save_project(root, crate::mivc::Model::data(&model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::interface::GraphicsPipeline;

    fn fresh_folder(name: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("flux_core_project_{name}_{}", std::process::id()));

        let _ = fs::remove_dir_all(&folder);
        folder
    }

    fn data(aphelion_control: bool) -> ProjectData {
        ProjectData {
            name: "Test".to_string(),
            engine_version: "0.8.0".to_string(),
            pipeline: GraphicsPipeline::Heavy,
            aphelion_control,
            created_on: "2026-11-10".to_string(),
        }
    }

    #[test]
    fn a_created_project_has_its_folders_and_loads_back() {
        let root = fresh_folder("create");

        create_project(&root, &data(false)).unwrap();

        assert!(layout::scenes_folder(&root).is_dir());
        assert!(layout::prefabs_folder(&root).is_dir());
        assert!(layout::scripts_folder(&root).is_dir());
        assert!(root.join(".gitignore").is_file());
        assert_eq!(load_project(&root).unwrap(), data(false));
    }

    #[test]
    fn a_folder_can_only_hold_one_project() {
        let root = fresh_folder("twice");

        create_project(&root, &data(false)).unwrap();

        assert!(create_project(&root, &data(false)).is_err());
    }

    #[test]
    fn the_aphelion_folder_exists_only_when_it_was_chosen() {
        let with = fresh_folder("with");
        let without = fresh_folder("without");

        create_project(&with, &data(true)).unwrap();
        create_project(&without, &data(false)).unwrap();

        assert!(layout::aphelion_folder(&with).is_dir());
        assert!(!layout::aphelion_folder(&without).exists());
    }

    #[test]
    fn the_engine_version_in_the_file_can_change() {
        let root = fresh_folder("version");

        create_project(&root, &data(false)).unwrap();
        set_engine_version(&root, "0.9.0").unwrap();

        assert_eq!(load_project(&root).unwrap().engine_version, "0.9.0");
    }

    #[test]
    fn opening_a_folder_without_a_project_is_an_error() {
        assert!(load_project(&fresh_folder("none")).is_err());
    }
}
