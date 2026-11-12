//! Where things live inside a project folder.

use std::path::{Path, PathBuf};

pub const PROJECT_FILE: &str = "project.yaml";
pub const SCENE_EXTENSION: &str = ".scene.yaml";
pub const PREFAB_EXTENSION: &str = ".prefab.yaml";

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

/// Assets/Prefabs/<name>.prefab.yaml
pub fn prefab_file(root: &Path, name: &str) -> PathBuf {
    prefabs_folder(root).join(format!("{name}{PREFAB_EXTENSION}"))
}

/// The same file as a path inside the project, written with `/`, like `Assets/Prefabs/Tree.prefab.yaml`.
/// This is how a prefab is remembered in the lists and in Aphelion's Control.
pub fn prefab_relative_path(name: &str) -> String {
    format!("Assets/Prefabs/{name}{PREFAB_EXTENSION}")
}

/// .aphelion/prefabs: one YAML file per prefab with its revisions.
pub fn aphelion_prefabs_folder(root: &Path) -> PathBuf {
    aphelion_folder(root).join("prefabs")
}

/// .aphelion/prefabs/<prefab uuid>.yaml
pub fn prefab_history_file(root: &Path, prefab_uuid: &str) -> PathBuf {
    aphelion_prefabs_folder(root).join(format!("{prefab_uuid}.yaml"))
}

/// Library/script_schemas: one file per script describing its serialized fields.
/// The scripting host writes them, they can always be rebuilt.
pub fn script_schemas_folder(root: &Path) -> PathBuf {
    library_folder(root).join("script_schemas")
}

/// Library/script_schemas/<script uuid>.yaml
pub fn script_schema_file(root: &Path, script_uuid: &str) -> PathBuf {
    script_schemas_folder(root).join(format!("{script_uuid}.yaml"))
}
