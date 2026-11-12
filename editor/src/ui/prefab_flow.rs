//! The prefab actions that need the disk, one function each:
//! create a prefab, apply an instance back to it, place it in the scene, revert a revision.
//!
//! Each one reads the models, changes them through their controllers, writes the files and
//! (if the project has Aphelion's Control) records a revision. The app calls `run` for every
//! `IoRequest` the command layer queued.

use super::commands::IoRequest;
use super::models::EditorModels;
use crate::hierarchy::HierarchyController;
use crate::history::HistoryController;
use crate::prefab_library::{query as library_query, PrefabLibraryController, PrefabListing};
use crate::project_io;
use flux_core::error::{EngineError, EngineResult};
use flux_core::project::controller::validate_name;
use flux_core::project::layout;
use flux_engine::control::{query as control_query, ControlController, SaveInfo};
use flux_engine::identity::AssetUuid;
use flux_engine::mivc::Model;
use flux_engine::prefab::{PrefabController, PrefabData, PrefabModel};
use flux_engine::scene::{query, SceneController};
use std::path::PathBuf;

fn invalid(message: &str) -> EngineError {
    EngineError::InvalidOperation(message.to_string())
}

fn project_root(models: &EditorModels) -> EngineResult<PathBuf> {
    models.project.as_ref().map(|project| project.root.clone()).ok_or_else(|| invalid("No project is open."))
}

fn control_enabled(models: &EditorModels) -> bool {
    models.project.as_ref().map_or(false, |project| project.data.data().aphelion_control)
}

fn selected_object(models: &EditorModels) -> EngineResult<AssetUuid> {
    models.hierarchy_ui.data().selected.first().copied().ok_or_else(|| invalid("Select an object first."))
}

/// Runs one request and returns the message for the status bar.
pub fn run(request: IoRequest, models: &mut EditorModels, today: &str) -> EngineResult<String> {
    match request {
        IoRequest::CreatePrefab => create_prefab(models, today),
        IoRequest::ApplyToPrefab => apply_to_prefab(models, today),
        IoRequest::PlacePrefab { file } => place_prefab(models, &file),
        IoRequest::RevertRevision { prefab, number } => revert_revision(models, &prefab, number, today),
    }
}

/// Records a save in Aphelion's Control, if the project has it, and writes the history file.
/// Returns the revision number (None if there is no Aphelion's Control or nothing changed).
fn record_revision(
    models: &mut EditorModels,
    before: Option<&PrefabData>,
    after: &PrefabData,
    info: SaveInfo,
) -> EngineResult<Option<u32>> {
    if !control_enabled(models) {
        return Ok(None);
    }

    let root = project_root(models)?;
    let number = ControlController.record_save(&mut models.control, before, after, info);

    if number.is_some() {
        if let Some(history) = control_query::find_history(models.control.data(), &after.uuid) {
            project_io::save_prefab_history(&root, history)?;
        }
    }

    Ok(number)
}

fn revision_note(number: Option<u32>) -> String {
    number.map_or_else(String::new, |number| format!(" (revision {number})"))
}

/// Saves the selected object (and everything under it) as a prefab, and turns the object
/// into an instance of it.
fn create_prefab(models: &mut EditorModels, today: &str) -> EngineResult<String> {
    let root = project_root(models)?;
    let object = selected_object(models)?;
    let found = query::find_object(&models.scene.data().objects, &object).ok_or_else(|| invalid("Select an object first."))?;
    let name = validate_name(&found.name)?;
    let path = layout::prefab_file(&root, &name);

    if path.exists() {
        return Err(EngineError::InvalidOperation(format!("A prefab called \"{name}\" already exists.")));
    }

    let before_scene = models.scene.data().clone();
    let prefab = PrefabController.create_from_object(&models.scene, &mut models.ids, &object, &name)?;
    let file = layout::prefab_relative_path(&name);

    project_io::save_prefab(&path, prefab.data())?;
    SceneController.set_prefab_source(&mut models.scene, &object, Some(prefab.data().uuid))?;
    HistoryController.record(&mut models.history, "Create prefab", None, before_scene);
    PrefabLibraryController.upsert(
        &mut models.library,
        PrefabListing { name: name.clone(), uuid: prefab.data().uuid, file: file.clone() },
    );

    let number = record_revision(
        models,
        None,
        prefab.data(),
        SaveInfo { file: &file, today, message: "Created", reverts: None },
    )?;

    // The prefab's ids are new, so the registry is saved right away.
    project_io::save_ids(&root, &models.ids)?;

    Ok(format!("Created prefab {name}{}", revision_note(number)))
}

/// Writes the selected instance back into its prefab.
fn apply_to_prefab(models: &mut EditorModels, today: &str) -> EngineResult<String> {
    let root = project_root(models)?;
    let object = selected_object(models)?;
    let prefab_uuid = query::find_object(&models.scene.data().objects, &object)
        .and_then(|found| found.prefab_source)
        .ok_or_else(|| invalid("That object is not a prefab instance."))?;
    let listing = library_query::find_by_uuid(models.library.data(), &prefab_uuid)
        .cloned()
        .ok_or_else(|| invalid("Its prefab is not in this project."))?;

    let path = root.join(&listing.file);
    let before = project_io::load_prefab(&path)?;
    let mut prefab = PrefabModel::new(before.clone());

    PrefabController.update_from_instance(&mut prefab, &models.scene, &mut models.ids, &object)?;

    if prefab.data() == &before {
        return Ok(format!("The prefab {} is already up to date", listing.name));
    }

    project_io::save_prefab(&path, prefab.data())?;

    let number = record_revision(
        models,
        Some(&before),
        prefab.data(),
        SaveInfo { file: &listing.file, today, message: "Applied changes from a scene object", reverts: None },
    )?;

    project_io::save_ids(&root, &models.ids)?;

    Ok(format!("Updated prefab {}{}", listing.name, revision_note(number)))
}

/// Puts a copy of a prefab in the scene (an undoable edit) and selects it.
fn place_prefab(models: &mut EditorModels, file: &str) -> EngineResult<String> {
    let root = project_root(models)?;
    let data = project_io::load_prefab(&root.join(file))?;
    let prefab = PrefabModel::new(data);
    let before_scene = models.scene.data().clone();

    let placed = PrefabController.instantiate(&prefab, &mut models.scene, &mut models.ids, None)?;

    HistoryController.record(&mut models.history, "Place prefab", None, before_scene);
    HierarchyController.select_only(&mut models.hierarchy_ui, placed);

    Ok(format!("Placed prefab {}", prefab.data().name))
}

/// Undoes one revision of a prefab and saves the result as a new revision.
/// Scene instances are not changed, they only follow when you apply or place again.
fn revert_revision(models: &mut EditorModels, prefab_uuid: &AssetUuid, number: u32, today: &str) -> EngineResult<String> {
    if !control_enabled(models) {
        return Err(invalid("Aphelion's Control is off for this project."));
    }

    let root = project_root(models)?;
    let listing = library_query::find_by_uuid(models.library.data(), prefab_uuid)
        .cloned()
        .ok_or_else(|| invalid("That prefab is not in this project."))?;
    let path = root.join(&listing.file);
    let current = project_io::load_prefab(&path)?;
    let report = ControlController.revert(&models.control, prefab_uuid, number, &current)?;

    project_io::save_prefab(&path, &report.prefab)?;

    let message = format!("Reverted revision {number}");

    record_revision(
        models,
        Some(&current),
        &report.prefab,
        SaveInfo { file: &listing.file, today, message: &message, reverts: Some(number) },
    )?;

    let kept = if report.skipped.is_empty() {
        String::new()
    } else {
        format!(", {} kept because later edits touched them", report.skipped.len())
    };

    Ok(format!("Reverted revision {number} of {}: {} undone{kept}", listing.name, report.applied))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_io::open_project;
    use flux_core::project::{disk, GraphicsPipeline, ProjectController};
    use std::fs;

    const TODAY: &str = "2026-11-10";

    fn open(name: &str, aphelion_control: bool) -> (EditorModels, PathBuf) {
        let root = std::env::temp_dir().join(format!("flux_flow_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let project = ProjectController
            .new_project("Test", "1.0.0", GraphicsPipeline::Simple, aphelion_control, TODAY)
            .unwrap();

        disk::create_project(&root, project.data()).unwrap();
        (EditorModels::from_opened(open_project(&root).unwrap()), root)
    }

    /// Makes an object, selects it, turns it into a prefab, and returns the object and the prefab's uuid.
    fn make_tree_prefab(models: &mut EditorModels) -> (AssetUuid, AssetUuid) {
        let tree = HierarchyController
            .create_object(&mut models.scene, &mut models.ids, &mut models.hierarchy_ui, "Tree", None)
            .unwrap();

        run(IoRequest::CreatePrefab, models, TODAY).unwrap();

        (tree, models.library.data().prefabs[0].uuid)
    }

    fn prefab_on_disk(root: &PathBuf) -> PrefabData {
        project_io::load_prefab(&layout::prefab_file(root, "Tree")).unwrap()
    }

    #[test]
    fn creating_a_prefab_saves_the_file_marks_the_instance_and_records_revision_one() {
        let (mut models, root) = open("create", true);
        let (tree, prefab_uuid) = make_tree_prefab(&mut models);

        assert!(layout::prefab_file(&root, "Tree").exists());
        assert_eq!(query::find_object(&models.scene.data().objects, &tree).unwrap().prefab_source, Some(prefab_uuid));
        assert_eq!(models.library.data().prefabs.len(), 1);
        assert_eq!(models.control.data().prefabs[0].revisions.len(), 1);
        assert!(layout::prefab_history_file(&root, &prefab_uuid.to_string()).exists());
        assert_eq!(models.history.data().undo.len(), 1);
    }

    #[test]
    fn a_second_prefab_with_the_same_name_is_refused() {
        let (mut models, _root) = open("same_name", true);

        make_tree_prefab(&mut models);

        assert!(run(IoRequest::CreatePrefab, &mut models, TODAY).is_err());
        assert_eq!(models.library.data().prefabs.len(), 1);
    }

    #[test]
    fn applying_adds_revisions_and_reverting_undoes_only_the_chosen_one() {
        let (mut models, root) = open("apply_revert", true);
        let (tree, prefab_uuid) = make_tree_prefab(&mut models);

        // Revision 2: the instance gets a child.
        SceneController.create_object(&mut models.scene, &mut models.ids, "Leaves", Some(tree)).unwrap();
        run(IoRequest::ApplyToPrefab, &mut models, TODAY).unwrap();
        assert_eq!(prefab_on_disk(&root).objects.len(), 2);

        // Revision 3: the instance is renamed.
        SceneController.rename_object(&mut models.scene, &tree, "Oak").unwrap();
        run(IoRequest::ApplyToPrefab, &mut models, TODAY).unwrap();
        assert_eq!(prefab_on_disk(&root).objects[0].name, "Oak");
        assert_eq!(models.control.data().prefabs[0].revisions.len(), 3);

        // Undo revision 2 only: the child goes, the new name stays.
        run(IoRequest::RevertRevision { prefab: prefab_uuid, number: 2 }, &mut models, TODAY).unwrap();

        let after = prefab_on_disk(&root);

        assert_eq!(after.objects.len(), 1);
        assert_eq!(after.objects[0].name, "Oak");

        let revisions = &models.control.data().prefabs[0].revisions;

        assert_eq!(revisions.len(), 4);
        assert_eq!(revisions[3].reverts, Some(2));
    }

    #[test]
    fn applying_without_changes_is_not_a_revision() {
        let (mut models, _root) = open("no_change", true);

        make_tree_prefab(&mut models);

        let message = run(IoRequest::ApplyToPrefab, &mut models, TODAY).unwrap();

        assert!(message.contains("up to date"));
        assert_eq!(models.control.data().prefabs[0].revisions.len(), 1);
    }

    #[test]
    fn a_project_without_aphelion_control_still_saves_prefabs_but_keeps_no_history() {
        let (mut models, root) = open("no_control", false);
        let (_tree, prefab_uuid) = make_tree_prefab(&mut models);

        assert!(layout::prefab_file(&root, "Tree").exists());
        assert!(models.control.data().prefabs.is_empty());
        assert!(!layout::aphelion_folder(&root).exists());
        assert!(run(IoRequest::RevertRevision { prefab: prefab_uuid, number: 2 }, &mut models, TODAY).is_err());
    }

    #[test]
    fn only_a_prefab_instance_can_be_applied() {
        let (mut models, _root) = open("not_instance", true);

        HierarchyController
            .create_object(&mut models.scene, &mut models.ids, &mut models.hierarchy_ui, "Rock", None)
            .unwrap();

        assert!(run(IoRequest::ApplyToPrefab, &mut models, TODAY).is_err());
    }

    #[test]
    fn placing_a_prefab_adds_a_selected_instance_and_is_one_undo_step() {
        let (mut models, _root) = open("place", true);
        let (_tree, prefab_uuid) = make_tree_prefab(&mut models);
        let file = models.library.data().prefabs[0].file.clone();
        let before = models.scene.data().objects.len();

        run(IoRequest::PlacePrefab { file }, &mut models, TODAY).unwrap();

        let placed = models.hierarchy_ui.data().selected[0];

        assert_eq!(models.scene.data().objects.len(), before + 1);
        assert_eq!(query::find_object(&models.scene.data().objects, &placed).unwrap().prefab_source, Some(prefab_uuid));
        assert_eq!(models.history.data().undo.len(), 2); // create prefab, place prefab
    }

    #[test]
    fn prefabs_and_their_history_are_there_again_after_reopening() {
        let (mut models, root) = open("reopen", true);
        let (tree, _prefab_uuid) = make_tree_prefab(&mut models);

        SceneController.rename_object(&mut models.scene, &tree, "Oak").unwrap();
        run(IoRequest::ApplyToPrefab, &mut models, TODAY).unwrap();

        let again = open_project(&root).unwrap();

        assert_eq!(again.library.data().prefabs.len(), 1);
        assert_eq!(again.control.data().prefabs[0].revisions.len(), 2);
        assert!(again.warnings.is_empty());
    }
}
