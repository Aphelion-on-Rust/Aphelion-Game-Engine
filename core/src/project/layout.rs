//! Where things live inside a project folder.

use std::path::{Path, PathBuf};

pub const PROJECT_FILE: &str = "project.yaml";
pub const SCENE_EXTENSION: &str = ".scene.yaml";

pub fn project_file(root: &Path) -> PathBuf {
    root.join(PROJECT_FILE)
}

pub fn assets_folder(root: &Path) -> PathBuf {
    root.join("Assets")
}

pub fn scenes_folder(root: &Path) -> PathBuf {
    assets_folder(root).join("Scenes")
}

pub fn prefabs_folder(root: &Path) -> PathBuf {
    assets_folder(root).join("Prefabs")
}

pub fn scripts_folder(root: &Path) -> PathBuf {
    assets_folder(root).join("Scripts")
}

pub fn settings_folder(root: &Path) -> PathBuf {
    root.join("Settings")
}

/// Files the engine can rebuild. Not meant for version control.
pub fn library_folder(root: &Path) -> PathBuf {
    root.join("Library")
}

pub fn id_registry_file(root: &Path) -> PathBuf {
    library_folder(root).join("id_registry.yaml")
}

/// Only exists when the project uses Aphelion's Control.
pub fn aphelion_folder(root: &Path) -> PathBuf {
    root.join(".aphelion")
}

/// Assets/Scenes/<name>.scene.yaml
pub fn scene_file(root: &Path, name: &str) -> PathBuf {
    scenes_folder(root).join(format!("{name}{SCENE_EXTENSION}"))
}
