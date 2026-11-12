//! Opening a project folder and saving its files (scene, prefabs, id registry, Aphelion's Control).
//! Plain functions with disk access. The models are only filled or read here, never changed
//! except through their controllers.

use crate::prefab_library::{PrefabLibraryController, PrefabLibraryModel, PrefabListing};
use flux_core::error::{EngineError, EngineResult};
use flux_core::project::{disk, layout, ProjectData};
use flux_core::storage::yaml::{load_yaml, save_yaml};
use flux_engine::control::{query as control_query, ControlData, ControlModel, PrefabHistoryData};
use flux_engine::identity::{IdController, IdRegistryData, IdRegistryModel};
use flux_engine::inspect::{InspectController, SchemaLibraryData, SchemaLibraryModel, ScriptSchemaData};
use flux_engine::mivc::{Model, Store};
use flux_engine::prefab::PrefabData;
use flux_engine::scene::{query, ComponentData, SceneController, SceneData, SceneModel};
use std::fs;
use std::path::{Path, PathBuf};

/// Everything read from a project folder.
pub struct OpenedProject {
    pub root: PathBuf,
    pub data: ProjectData,
    pub scene: SceneModel,
    pub scene_path: PathBuf,
    pub ids: IdRegistryModel,
    pub library: PrefabLibraryModel,
    /// The histories of all prefabs. Empty if the project has no Aphelion's Control.
    pub control: ControlModel,
    /// The fields of every script (`Library/script_schemas`), for the Inspector.
    pub schemas: SchemaLibraryModel,
    /// Files that could not be read. The project opens anyway, the editor shows these.
    pub warnings: Vec<String>,
}

/// The files in a folder whose name ends with `extension`, sorted by name.
fn files_with_extension(folder: &Path, extension: &str) -> EngineResult<Vec<PathBuf>> {
    if !folder.exists() {
        return Ok(Vec::new());
    }

    let mut files: Vec<PathBuf> = fs::read_dir(folder)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|name| name.to_str()).map_or(false, |name| name.ends_with(extension))
        })
        .collect();

    files.sort();
    Ok(files)
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

fn register_prefab_ids(prefab: &PrefabData, ids: &mut IdRegistryModel) {
    IdController.ensure_uuid(ids, prefab.uuid);

    for object in &prefab.objects {
        IdController.ensure_uuid(ids, object.uuid);

        for component in &object.components {
            IdController.ensure_component_id(ids, component.id);
        }
    }
}

/// Reads every prefab file. Ids are registered (so they are never reused). A file that can't be
/// read is reported in `warnings` and left out.
fn load_prefabs(root: &Path, ids: &mut IdRegistryModel, warnings: &mut Vec<String>) -> EngineResult<Vec<PrefabListing>> {
    let mut listings = Vec::new();

    for path in files_with_extension(&layout::prefabs_folder(root), layout::PREFAB_EXTENSION)? {
        let name = path
            .file_name()
            .and_then(|file| file.to_str())
            .and_then(|file| file.strip_suffix(layout::PREFAB_EXTENSION))
            .unwrap_or_default()
            .to_string();

        match load_yaml::<PrefabData>(&path) {
            Ok(prefab) => {
                register_prefab_ids(&prefab, ids);
                listings.push(PrefabListing { uuid: prefab.uuid, file: layout::prefab_relative_path(&name), name });
            }
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }

    Ok(listings)
}

/// Reads the prefab histories of Aphelion's Control. Every id that ever appeared in them is
/// registered too, even ids of objects that were removed.
fn load_control(root: &Path, ids: &mut IdRegistryModel, warnings: &mut Vec<String>) -> EngineResult<ControlData> {
    let mut control = ControlData::default();

    for path in files_with_extension(&layout::aphelion_prefabs_folder(root), ".yaml")? {
        match load_yaml::<PrefabHistoryData>(&path) {
            Ok(history) => {
                let (uuids, component_ids) = control_query::used_ids(&history);

                for uuid in uuids {
                    IdController.ensure_uuid(ids, uuid);
                }

                for id in component_ids {
                    IdController.ensure_component_id(ids, id);
                }

                control.prefabs.push(history);
            }
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }

    Ok(control)
}

/// Reads the script schemas. A file that can't be read is reported in `warnings` and left out.
fn load_schemas(root: &Path, warnings: &mut Vec<String>) -> EngineResult<SchemaLibraryData> {
    let mut library = SchemaLibraryData::default();

    for path in files_with_extension(&layout::script_schemas_folder(root), ".yaml")? {
        match load_yaml::<ScriptSchemaData>(&path) {
            Ok(schema) => library.scripts.push(schema),
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }

    Ok(library)
}

/// Gives every script in the scene a value for each field of its schema that has none yet
/// (a script that was added before it had fields, or a schema that gained a field).
/// This changes the scene in memory only, it is saved with the next save.
fn sync_script_fields(scene: &mut SceneModel, schemas: &SchemaLibraryModel) {
    let scripts: Vec<_> = scene
        .data()
        .objects
        .iter()
        .flat_map(|object| {
            object
                .components
                .iter()
                .filter(|entry| matches!(entry.data, ComponentData::Script(_)))
                .map(move |entry| (object.uuid, entry.id))
        })
        .collect();

    for (object, component) in scripts {
        // A script without a schema simply has nothing to fill in.
        let _ = InspectController.sync_script_fields(scene, schemas, &object, component);
    }
}

/// Reads project.yaml, the id registry, the prefabs, Aphelion's Control and the first scene.
/// A project with no scene gets a new empty "Main" scene. A scene file that repeats an id is refused.
pub fn open_project(root: &Path) -> EngineResult<OpenedProject> {
    let data = disk::load_project(root)?;

    let registry_file = layout::id_registry_file(root);
    let registry: IdRegistryData = if registry_file.exists() { load_yaml(&registry_file)? } else { IdRegistryData::default() };
    let mut ids: IdRegistryModel = Store::new(registry);

    let (scene, scene_path) = match files_with_extension(&layout::scenes_folder(root), layout::SCENE_EXTENSION)?.into_iter().next() {
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

    let mut warnings = Vec::new();
    let mut scene = scene;
    let schemas: SchemaLibraryModel = Store::new(load_schemas(root, &mut warnings)?);

    sync_script_fields(&mut scene, &schemas);

    let listings = load_prefabs(root, &mut ids, &mut warnings)?;
    let control = if data.aphelion_control { load_control(root, &mut ids, &mut warnings)? } else { ControlData::default() };

    save_ids(root, &ids)?;

    let mut library = PrefabLibraryModel::default();

    PrefabLibraryController.set_all(&mut library, listings);

    Ok(OpenedProject { root: root.to_path_buf(), data, scene, scene_path, ids, library, control: Store::new(control), schemas, warnings })
}

pub fn save_scene(scene_path: &Path, scene: &SceneModel) -> EngineResult<()> {
    save_yaml(scene_path, scene.data())
}

pub fn save_ids(root: &Path, ids: &IdRegistryModel) -> EngineResult<()> {
    save_yaml(&layout::id_registry_file(root), ids.data())
}

pub fn load_prefab(path: &Path) -> EngineResult<PrefabData> {
    load_yaml(path)
}

pub fn save_prefab(path: &Path, prefab: &PrefabData) -> EngineResult<()> {
    save_yaml(path, prefab)
}

/// Writes one prefab's history to `.aphelion/prefabs/<prefab uuid>.yaml`.
pub fn save_prefab_history(root: &Path, history: &PrefabHistoryData) -> EngineResult<()> {
    save_yaml(&layout::prefab_history_file(root, &history.prefab.to_string()), history)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::project::{GraphicsPipeline, ProjectController};
    use flux_engine::mivc::Controller;
    use flux_engine::prefab::PrefabController;

    fn new_project_with(name: &str, aphelion_control: bool) -> PathBuf {
        let root = std::env::temp_dir().join(format!("flux_editor_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let model = ProjectController
            .new_project("Test", "1.0.0", GraphicsPipeline::Simple, aphelion_control, "2026-11-10")
            .unwrap();

        disk::create_project(&root, model.data()).unwrap();
        root
    }

    fn new_project(name: &str) -> PathBuf {
        new_project_with(name, false)
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

    #[test]
    fn prefabs_are_listed_and_their_ids_are_registered_when_a_project_opens() {
        let root = new_project("prefabs");
        let mut opened = open_project(&root).unwrap();
        let tree = SceneController.create_object(&mut opened.scene, &mut opened.ids, "Tree", None).unwrap();
        let prefab = PrefabController.create_from_object(&opened.scene, &mut opened.ids, &tree, "Tree").unwrap();

        save_prefab(&layout::prefab_file(&root, "Tree"), prefab.data()).unwrap();
        // The registry is NOT saved here, like after a crash. Opening must register the prefab ids.

        let again = open_project(&root).unwrap();

        assert_eq!(again.library.data().prefabs.len(), 1);
        assert_eq!(again.library.data().prefabs[0].name, "Tree");
        assert_eq!(again.library.data().prefabs[0].file, "Assets/Prefabs/Tree.prefab.yaml");
        assert!(IdController.is_uuid_used(&again.ids, &prefab.data().uuid));
        assert!(IdController.is_uuid_used(&again.ids, &prefab.data().objects[0].uuid));
    }

    #[test]
    fn a_broken_prefab_file_is_a_warning_and_does_not_block_the_project() {
        let root = new_project("broken");

        fs::write(layout::prefab_file(&root, "Bad"), "this is not a prefab").unwrap();

        let opened = open_project(&root).unwrap();

        assert_eq!(opened.warnings.len(), 1);
        assert!(opened.library.data().prefabs.is_empty());
    }

    #[test]
    fn histories_are_read_when_the_project_has_aphelion_control() {
        use flux_engine::control::{ControlController, SaveInfo};

        let root = new_project_with("control", true);
        let mut opened = open_project(&root).unwrap();
        let tree = SceneController.create_object(&mut opened.scene, &mut opened.ids, "Tree", None).unwrap();
        let prefab = PrefabController.create_from_object(&opened.scene, &mut opened.ids, &tree, "Tree").unwrap();
        let mut control = ControlModel::default();

        ControlController.record_save(
            &mut control,
            None,
            prefab.data(),
            SaveInfo { file: "Assets/Prefabs/Tree.prefab.yaml", today: "2026-11-10", message: "Created", reverts: None },
        );
        save_prefab_history(&root, &control.data().prefabs[0]).unwrap();

        let again = open_project(&root).unwrap();

        assert_eq!(again.control.data().prefabs.len(), 1);
        assert_eq!(again.control.data().prefabs[0].revisions.len(), 1);
    }

    #[test]
    fn script_schemas_are_loaded_and_scripts_get_their_default_values_on_open() {
        use flux_engine::identity::IdController;
        use flux_engine::inspect::{FieldDescriptor, FieldEntry, FieldKind, FieldValue};
        use flux_engine::scene::ScriptData;

        let root = new_project("schemas");
        let mut opened = open_project(&root).unwrap();
        let object = SceneController.create_object(&mut opened.scene, &mut opened.ids, "Player", None).unwrap();
        let script = IdController.new_uuid(&mut opened.ids);

        SceneController
            .add_component(&mut opened.scene, &mut opened.ids, &object, ComponentData::Script(ScriptData::new(script, true)))
            .unwrap();
        save_scene(&opened.scene_path, &opened.scene).unwrap();
        save_ids(&root, &opened.ids).unwrap();

        // The scripting host (later) writes a schema file for the script.
        let schema = ScriptSchemaData {
            script,
            class_name: "Player".to_string(),
            fields: vec![FieldDescriptor {
                name: "speed".to_string(),
                kind: FieldKind::Float,
                default: Some(FieldValue::Float(5.0)),
                attributes: Vec::new(),
            }],
        };

        save_yaml(&layout::script_schema_file(&root, &script.to_string()), &schema).unwrap();

        let again = open_project(&root).unwrap();

        assert_eq!(again.schemas.data().scripts.len(), 1);

        match &again.scene.data().objects[0].components[1].data {
            ComponentData::Script(data) => {
                assert_eq!(data.fields, vec![FieldEntry { name: "speed".to_string(), value: FieldValue::Float(5.0) }]);
            }
            other => panic!("expected a script, got {other:?}"),
        }
    }

    #[test]
    fn a_broken_schema_file_is_a_warning() {
        let root = new_project("broken_schema");

        fs::create_dir_all(layout::script_schemas_folder(&root)).unwrap();
        fs::write(layout::script_schema_file(&root, "bad"), "nonsense: [").unwrap();

        let opened = open_project(&root).unwrap();

        assert_eq!(opened.warnings.len(), 1);
        assert!(opened.schemas.data().scripts.is_empty());
    }
}
