//! Opening a project folder and saving the scene and the id registry.
//! Plain functions with disk access. The models are only filled or read here, never changed
//! except through their controllers.

use flux_core::error::{EngineError, EngineResult};
use flux_core::project::{disk, layout, ProjectData};
use flux_core::storage::yaml::{load_yaml, save_yaml};
use flux_engine::identity::{IdController, IdRegistryData, IdRegistryModel};
use flux_engine::mivc::{Model, Store};
use flux_engine::scene::{query, SceneController, SceneData, SceneModel};
use std::fs;
use std::path::{Path, PathBuf};

/// Everything read from a project folder.
pub struct OpenedProject {
    pub root: PathBuf,
    pub data: ProjectData,
    pub scene: SceneModel,
    pub scene_path: PathBuf,
    pub ids: IdRegistryModel,
}

/// The first scene file in Assets/Scenes (by name), if there is one.
fn first_scene_file(root: &Path) -> EngineResult<Option<PathBuf>> {
    let folder = layout::scenes_folder(root);

    if !folder.exists() {
        return Ok(None);
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&folder)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| name.ends_with(layout::SCENE_EXTENSION))
        })
        .collect();

    files.sort();
    Ok(files.into_iter().next())
}

/// Marks every id used in the scene as taken, so new objects never get one of them.
fn register_scene_ids(scene: &SceneModel, ids: &mut IdRegistryModel) {
    IdController.ensure_uuid(ids, scene.data().uuid);

    for object in &scene.data().objects {
        IdController.ensure_uuid(ids, object.uuid);

        for component in &object.components {
            IdController.ensure_component_id(ids, component.id);
        }
    }
}

/// Reads project.yaml, the id registry and the first scene. A project with no scene
/// gets a new empty "Main" scene. A scene file that repeats an id is refused.
pub fn open_project(root: &Path) -> EngineResult<OpenedProject> {
    let data = disk::load_project(root)?;

    let registry_file = layout::id_registry_file(root);
    let registry: IdRegistryData = if registry_file.exists() { load_yaml(&registry_file)? } else { IdRegistryData::default() };
    let mut ids: IdRegistryModel = Store::new(registry);

    let (scene, scene_path) = match first_scene_file(root)? {
        Some(path) => (Store::new(load_yaml::<SceneData>(&path)?), path),
        None => {
            let scene = SceneController.create_scene(&mut ids, "Main");
            let path = layout::scene_file(root, "Main");

            save_yaml(&path, scene.data())?;
            (scene, path)
        }
    };

    if let Some(problem) = query::find_duplicate_ids(&scene.data().objects) {
        return Err(EngineError::InvalidOperation(format!("{}: {problem}", scene_path.display())));
    }

    register_scene_ids(&scene, &mut ids);
    save_ids(root, &ids)?;

    Ok(OpenedProject { root: root.to_path_buf(), data, scene, scene_path, ids })
}

pub fn save_scene(scene_path: &Path, scene: &SceneModel) -> EngineResult<()> {
    save_yaml(scene_path, scene.data())
}

pub fn save_ids(root: &Path, ids: &IdRegistryModel) -> EngineResult<()> {
    save_yaml(&layout::id_registry_file(root), ids.data())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::project::{GraphicsPipeline, ProjectController};
    use flux_engine::mivc::Controller;

    fn new_project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("flux_editor_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let model = ProjectController
            .new_project("Test", "0.8.0", GraphicsPipeline::Simple, false, "2026-11-10")
            .unwrap();

        disk::create_project(&root, model.data()).unwrap();
        root
    }

    #[test]
    fn an_empty_project_gets_a_main_scene() {
        let root = new_project("empty");
        let opened = open_project(&root).unwrap();

        assert_eq!(opened.scene.data().name, "Main");
        assert!(opened.scene_path.exists());
        assert!(layout::id_registry_file(&root).exists());
    }

    #[test]
    fn a_saved_scene_and_its_ids_come_back() {
        let root = new_project("roundtrip");
        let mut opened = open_project(&root).unwrap();

        let object = SceneController.create_object(&mut opened.scene, &mut opened.ids, "Player", None).unwrap();

        save_scene(&opened.scene_path, &opened.scene).unwrap();
        save_ids(&root, &opened.ids).unwrap();

        let mut again = open_project(&root).unwrap();

        assert_eq!(again.scene.data().objects.len(), 1);
        assert_eq!(again.scene.data().objects[0].name, "Player");
        assert!(IdController.is_uuid_used(&again.ids, &object));

        // The next object must not reuse an id.
        let next = SceneController.create_object(&mut again.scene, &mut again.ids, "Enemy", None).unwrap();
        assert_ne!(next, object);
    }

    #[test]
    fn a_scene_file_with_a_repeated_id_is_refused() {
        let root = new_project("duplicate");
        let mut opened = open_project(&root).unwrap();

        SceneController.create_object(&mut opened.scene, &mut opened.ids, "A", None).unwrap();

        let copy = opened.scene.data().objects[0].clone();
        SceneController.write(&mut opened.scene).objects.push(copy);
        save_scene(&opened.scene_path, &opened.scene).unwrap();

        assert!(open_project(&root).is_err());
    }

    #[test]
    fn a_folder_without_a_project_is_refused() {
        let folder = std::env::temp_dir().join(format!("flux_editor_none_{}", std::process::id()));

        assert!(open_project(&folder).is_err());
    }
}
