//! Commands: what the user asked for. Views push them, `apply_all` runs them.
//! This is the only place where the UI layer changes models, always through a controller.

use super::field::{with_number, NumberTarget};
use super::models::EditorModels;
use crate::hierarchy::HierarchyController;
use crate::history::HistoryController;
use crate::keybinds::actions;
use crate::layout::{DropZone, LayoutController, PanelKind};
use crate::number_field::{DragPhase, DragSpeed, NumberFieldConfig, NumberFieldController};
use crate::theme::ThemeController;
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::identity::{AssetUuid, ComponentId};
use flux_engine::inspect::{FieldPath, FieldValue, InspectController};
use flux_engine::mivc::Model;
use flux_engine::scene::{ComponentData, ObjectPreset, SceneController};

#[derive(Clone, Debug)]
pub enum UiCommand {
    // layout
    SetActiveTab(PanelKind),
    ClosePanel(PanelKind),
    OpenPanel(PanelKind),
    MovePanel { panel: PanelKind, target: PanelKind, zone: DropZone },
    SetSplitRatio { path: Vec<bool>, ratio: f32 },
    ResetLayout,
    // theme
    SetTheme(String),
    // project
    SaveScene,
    // prefabs (they need the disk, so the app does the work, see IoRequest)
    CreatePrefab,
    ApplyToPrefab,
    PlacePrefab { file: String },
    RevertRevision { prefab: AssetUuid, number: u32 },
    // history
    Undo,
    Redo,
    /// Go back this many steps (a click in the history panel).
    UndoSteps(usize),
    /// Go forward this many steps.
    RedoSteps(usize),
    // hierarchy
    SelectObject { object: AssetUuid, additive: bool },
    ToggleExpanded(AssetUuid),
    CreateObject { parent: Option<AssetUuid> },
    DeleteSelected,
    Reparent { object: AssetUuid, new_parent: Option<AssetUuid> },
    SetObjectActive { object: AssetUuid, active: bool },
    RenameObject { object: AssetUuid, name: String },
    /// Start renaming in the hierarchy (double-click or F2).
    BeginRename(AssetUuid),
    /// The text box was confirmed. An empty name changes nothing.
    FinishRename { object: AssetUuid, name: String },
    CancelRename,
    // components
    SetScriptEnabled { object: AssetUuid, component: ComponentId, enabled: bool },
    /// Take a component off an object. A Transform or RectTransform can't be removed.
    RemoveComponent { object: AssetUuid, component: ComponentId },
    /// Put a Transform or RectTransform back to its default values.
    ResetComponent { object: AssetUuid, component: ComponentId },
    /// Make a ready-made object (cube, camera, light ...) from the Create menu.
    CreatePreset { preset: ObjectPreset, parent: Option<AssetUuid> },
    /// Add a component of this kind (`ViewCamera`, `LightSource` ...) to an object.
    AddComponent { object: AssetUuid, kind: String },
    /// Set a field (or a value inside a struct or list) from the Eclipse Inspector.
    SetField { object: AssetUuid, component: ComponentId, path: FieldPath, value: FieldValue },
    /// Put a field back to its default.
    ResetField { object: AssetUuid, component: ComponentId, path: FieldPath },
    /// Add an element to a list field.
    ListPush { object: AssetUuid, component: ComponentId, path: FieldPath },
    /// Remove one element of a list field.
    ListRemove { object: AssetUuid, component: ComponentId, path: FieldPath, index: usize },
    // slide-to-change number fields
    NumberHover,
    NumberDragBegin { target: NumberTarget, value: f64, config: NumberFieldConfig },
    NumberDragMove { delta_x: f64, speed: DragSpeed },
    NumberDragEnd,
    NumberDragCancel,
}

impl UiCommand {
    /// For commands that edit the scene: the name of the undo step, and a merge key
    /// (steps with the same key that follow each other become one step).
    fn history_info(&self) -> Option<(String, Option<String>)> {
        match self {
            UiCommand::CreateObject { .. } => Some(("Create object".to_string(), None)),
            UiCommand::DeleteSelected => Some(("Delete".to_string(), None)),
            UiCommand::Reparent { .. } => Some(("Move object".to_string(), None)),
            UiCommand::SetObjectActive { .. } => Some(("Turn object on/off".to_string(), None)),
            UiCommand::SetScriptEnabled { .. } => Some(("Turn script on/off".to_string(), None)),
            UiCommand::RenameObject { object, .. } | UiCommand::FinishRename { object, .. } => {
                Some(("Rename".to_string(), Some(format!("rename:{object}"))))
            }
            UiCommand::RemoveComponent { .. } => Some(("Remove component".to_string(), None)),
            UiCommand::ResetComponent { .. } => Some(("Reset component".to_string(), None)),
            UiCommand::CreatePreset { .. } => Some(("Create object".to_string(), None)),
            UiCommand::AddComponent { .. } => Some(("Add component".to_string(), None)),
            // Typing in one field is one step: steps for the same field are merged.
            UiCommand::SetField { object, component, path, .. } => Some((
                "Change value".to_string(),
                Some(format!("field:{object}:{}:{:?}", component.value(), path.segments())),
            )),
            UiCommand::ResetField { .. } => Some(("Reset value".to_string(), None)),
            UiCommand::ListPush { .. } => Some(("Add list element".to_string(), None)),
            UiCommand::ListRemove { .. } => Some(("Remove list element".to_string(), None)),
            _ => None,
        }
    }
}

/// Work that needs the disk. The command layer only asks for it, the app does it
/// (see `prefab_flow`).
#[derive(Clone, Debug, PartialEq)]
pub enum IoRequest {
    /// Save the selected object as a prefab.
    CreatePrefab,
    /// Write the selected prefab instance back into its prefab.
    ApplyToPrefab,
    /// Put a copy of a prefab in the scene. `file` is inside the project.
    PlacePrefab { file: String },
    /// Undo one revision of a prefab (Aphelion's Control).
    RevertRevision { prefab: AssetUuid, number: u32 },
}

/// State the controller side needs between frames. Not a model, nothing here is saved.
pub struct ControllerState {
    /// Message shown in the status bar.
    pub status: String,
    /// The number being dragged right now.
    pub active_target: Option<NumberTarget>,
    /// Total horizontal drag so far. The pointer is hidden while dragging,
    /// so we add up its movement instead of reading its position.
    pub drag_x: f64,
    /// Layout or theme changed, the settings files should be saved.
    pub settings_dirty: bool,
    /// The user asked to save the scene. The app does the file writing.
    pub save_requested: bool,
    /// Prefab work waiting for the app (it needs the disk).
    pub pending_io: Vec<IoRequest>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            status: "Ready".to_string(),
            active_target: None,
            drag_x: 0.0,
            settings_dirty: false,
            save_requested: false,
            pending_io: Vec::new(),
        }
    }
}

/// Runs the commands in order. An error does not stop the rest, it shows in the status bar.
pub fn apply_all(commands: Vec<UiCommand>, models: &mut EditorModels, state: &mut ControllerState) {
    for command in commands {
        // Commands that edit the scene become an undo step, if they really changed something.
        let edit = command.history_info();
        let before = edit.as_ref().map(|_| models.scene.data().clone());

        if let Err(error) = apply(command, models, state) {
            state.status = error.to_string();
        }

        if let (Some((label, merge_key)), Some(before)) = (edit, before) {
            if models.scene.data() != &before {
                HistoryController.record(&mut models.history, &label, merge_key, before);
            }
        }
    }
}

/// Turns a shortcut action into a command. None for actions that are not built yet.
pub fn action_to_command(action: &str, models: &EditorModels) -> Option<UiCommand> {
    match action {
        actions::SAVE => Some(UiCommand::SaveScene),
        actions::RENAME => models.hierarchy_ui.data().selected.first().copied().map(UiCommand::BeginRename),
        actions::UNDO => Some(UiCommand::Undo),
        actions::REDO => Some(UiCommand::Redo),
        actions::DELETE => Some(UiCommand::DeleteSelected),
        actions::NEW_OBJECT => Some(UiCommand::CreateObject {
            parent: models.hierarchy_ui.data().selected.first().copied(),
        }),
        _ => None,
    }
}

/// A number box changed one number of a field. The field's rules (range, min ...) are applied.
fn write_to_scene(models: &mut EditorModels, target: &NumberTarget, number: f64) -> EngineResult<()> {
    let current = InspectController.get_field(&models.scene, &target.object, target.component, &target.path)?;
    let value = with_number(&current, target.part, number)
        .ok_or_else(|| EngineError::InvalidOperation("that field has no such number".to_string()))?;

    InspectController.set_field(&mut models.scene, &models.schemas, &target.object, target.component, &target.path, value)
}

enum Direction {
    Back,
    Forward,
}

/// Undo or redo `count` steps. Not while a number is being dragged.
fn step_history(models: &mut EditorModels, state: &mut ControllerState, direction: Direction, count: usize) {
    if is_dragging(models) {
        return;
    }

    let mut last = None;

    for _ in 0..count {
        let step = match direction {
            Direction::Back => HistoryController.undo(&mut models.history, &mut models.scene),
            Direction::Forward => HistoryController.redo(&mut models.history, &mut models.scene),
        };

        match step {
            Some(label) => last = Some(label),
            None => break,
        }
    }

    match last {
        Some(label) => {
            let verb = match direction {
                Direction::Back => "Undid",
                Direction::Forward => "Redid",
            };

            state.status = format!("{verb} {label}");
            HierarchyController.prune_missing(&mut models.hierarchy_ui, &models.scene);
        }
        None => {
            state.status = match direction {
                Direction::Back => "Nothing to undo".to_string(),
                Direction::Forward => "Nothing to redo".to_string(),
            };
        }
    }
}

fn is_dragging(models: &EditorModels) -> bool {
    models.number_field.data().phase == DragPhase::Dragging
}

fn apply(command: UiCommand, models: &mut EditorModels, state: &mut ControllerState) -> EngineResult<()> {
    match command {
        // ----- layout -----
        UiCommand::SetActiveTab(panel) => {
            LayoutController.set_active_tab(&mut models.layout, panel)?;
            state.settings_dirty = true;
        }
        UiCommand::ClosePanel(panel) => {
            LayoutController.close_panel(&mut models.layout, panel)?;
            state.settings_dirty = true;
        }
        UiCommand::OpenPanel(panel) => {
            LayoutController.open_panel(&mut models.layout, panel);
            state.settings_dirty = true;
        }
        UiCommand::MovePanel { panel, target, zone } => {
            LayoutController.move_panel(&mut models.layout, panel, target, zone)?;
            state.settings_dirty = true;
        }
        UiCommand::SetSplitRatio { path, ratio } => {
            LayoutController.set_split_ratio(&mut models.layout, &path, ratio)?;
            state.settings_dirty = true;
        }
        UiCommand::ResetLayout => {
            LayoutController.reset(&mut models.layout);
            state.settings_dirty = true;
        }

        // ----- theme -----
        UiCommand::SetTheme(name) => {
            ThemeController.set_active(&mut models.theme, &name)?;
            state.settings_dirty = true;
        }

        // ----- project -----
        UiCommand::SaveScene => state.save_requested = true,

        // ----- prefabs -----
        UiCommand::CreatePrefab => state.pending_io.push(IoRequest::CreatePrefab),
        UiCommand::ApplyToPrefab => state.pending_io.push(IoRequest::ApplyToPrefab),
        UiCommand::PlacePrefab { file } => state.pending_io.push(IoRequest::PlacePrefab { file }),
        UiCommand::RevertRevision { prefab, number } => state.pending_io.push(IoRequest::RevertRevision { prefab, number }),

        // ----- history -----
        UiCommand::Undo => step_history(models, state, Direction::Back, 1),
        UiCommand::Redo => step_history(models, state, Direction::Forward, 1),
        UiCommand::UndoSteps(count) => step_history(models, state, Direction::Back, count),
        UiCommand::RedoSteps(count) => step_history(models, state, Direction::Forward, count),

        // ----- hierarchy -----
        UiCommand::SelectObject { object, additive } => {
            if additive {
                HierarchyController.toggle_select(&mut models.hierarchy_ui, object);
            } else {
                HierarchyController.select_only(&mut models.hierarchy_ui, object);
            }
        }
        UiCommand::ToggleExpanded(object) => HierarchyController.toggle_expanded(&mut models.hierarchy_ui, object),
        UiCommand::CreateObject { parent } => {
            HierarchyController.create_object(
                &mut models.scene,
                &mut models.ids,
                &mut models.hierarchy_ui,
                "GameObject",
                parent,
            )?;
        }
        UiCommand::DeleteSelected => {
            HierarchyController.delete_selected(&mut models.scene, &mut models.hierarchy_ui)?;
        }
        UiCommand::Reparent { object, new_parent } => {
            HierarchyController.reparent(&mut models.scene, &mut models.hierarchy_ui, &object, new_parent)?;
        }
        UiCommand::SetObjectActive { object, active } => {
            SceneController.set_active(&mut models.scene, &object, active)?;
        }
        UiCommand::RenameObject { object, name } => {
            SceneController.rename_object(&mut models.scene, &object, &name)?;
        }
        UiCommand::BeginRename(object) => HierarchyController.begin_rename(&mut models.hierarchy_ui, object),
        UiCommand::FinishRename { object, name } => {
            HierarchyController.end_rename(&mut models.hierarchy_ui);

            if !name.trim().is_empty() {
                SceneController.rename_object(&mut models.scene, &object, name.trim())?;
            }
        }
        UiCommand::CancelRename => HierarchyController.end_rename(&mut models.hierarchy_ui),

        // ----- components -----
        UiCommand::SetScriptEnabled { object, component, enabled } => {
            SceneController.update_component(&mut models.scene, &object, component, |data| {
                if let ComponentData::Script(script) = data {
                    script.enabled = enabled;
                }
            })?;
        }

        UiCommand::RemoveComponent { object, component } => {
            SceneController.remove_component(&mut models.scene, &object, component)?;
        }
        UiCommand::ResetComponent { object, component } => {
            SceneController.update_component(&mut models.scene, &object, component, |data| data.reset())?;
        }

        UiCommand::CreatePreset { preset, parent } => {
            HierarchyController.create_preset(&mut models.scene, &mut models.ids, &mut models.hierarchy_ui, preset, parent)?;
        }
        UiCommand::AddComponent { object, kind } => {
            let data = ComponentData::new_of_kind(&kind)
                .ok_or_else(|| EngineError::InvalidOperation(format!("there is no component called \"{kind}\"")))?;

            SceneController.add_component(&mut models.scene, &mut models.ids, &object, data)?;
        }

        // ----- fields (Eclipse Inspector) -----
        UiCommand::SetField { object, component, path, value } => {
            InspectController.set_field(&mut models.scene, &models.schemas, &object, component, &path, value)?;
        }
        UiCommand::ResetField { object, component, path } => {
            InspectController.reset_field(&mut models.scene, &models.schemas, &object, component, &path)?;
        }
        UiCommand::ListPush { object, component, path } => {
            InspectController.list_push(&mut models.scene, &models.schemas, &object, component, &path)?;
        }
        UiCommand::ListRemove { object, component, path, index } => {
            InspectController.list_remove(&mut models.scene, &models.schemas, &object, component, &path, index)?;
        }

        // ----- number fields -----
        UiCommand::NumberHover => NumberFieldController.hover_enter(&mut models.number_field),
        UiCommand::NumberDragBegin { target, value, config } => {
            NumberFieldController.begin_drag(&mut models.number_field, 0.0, value, config)?;
            // The whole drag becomes one undo step.
            HistoryController.begin_group(&mut models.history, "Change value", models.scene.data().clone());
            state.active_target = Some(target);
            state.drag_x = 0.0;
        }
        UiCommand::NumberDragMove { delta_x, speed } => {
            // The pointer may still move for a frame after Escape. Ignore that.
            if let (true, Some(target)) = (is_dragging(models), state.active_target.clone()) {
                state.drag_x += delta_x;

                let value = NumberFieldController.drag_to(&mut models.number_field, state.drag_x, speed)?;
                write_to_scene(models, &target, value)?;
            }
        }
        UiCommand::NumberDragEnd => {
            if is_dragging(models) {
                NumberFieldController.end_drag(&mut models.number_field)?;
                HistoryController.commit_group(&mut models.history, models.scene.data());
            }

            state.active_target = None;
        }
        UiCommand::NumberDragCancel => {
            if let (true, Some(target)) = (is_dragging(models), state.active_target.clone()) {
                let old_value = NumberFieldController.cancel_drag(&mut models.number_field)?;
                write_to_scene(models, &target, old_value)?;
            }

            HistoryController.cancel_group(&mut models.history);
            state.active_target = None;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::field::Part;
    use crate::number_field::NumberFieldConfig;
    use crate::theme::query::active_theme;
    use flux_engine::scene::query;

    fn first_transform_x(models: &EditorModels) -> f32 {
        match &models.scene.data().objects[0].components[0].data {
            ComponentData::Transform(transform) => transform.position.x,
            other => panic!("expected a transform, got {other:?}"),
        }
    }

    fn first_target(models: &EditorModels) -> NumberTarget {
        let object = &models.scene.data().objects[0];

        NumberTarget {
            object: object.uuid,
            component: object.components[0].id,
            path: FieldPath::field("position"),
            part: Part::X,
        }
    }

    #[test]
    fn dragging_a_number_changes_the_scene_and_escape_puts_it_back() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let target = first_target(&models);

        apply_all(
            vec![
                UiCommand::NumberDragBegin { target, value: 0.0, config: NumberFieldConfig::default() },
                UiCommand::NumberDragMove { delta_x: 100.0, speed: DragSpeed::Normal },
            ],
            &mut models,
            &mut state,
        );

        // 100 pixels at 0.1 per pixel.
        assert!((first_transform_x(&models) - 10.0).abs() < 1e-4);

        apply_all(vec![UiCommand::NumberDragCancel], &mut models, &mut state);
        assert_eq!(first_transform_x(&models), 0.0);
        assert!(state.active_target.is_none());
    }

    #[test]
    fn finishing_a_drag_keeps_the_value() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let target = first_target(&models);

        apply_all(
            vec![
                UiCommand::NumberDragBegin { target, value: 0.0, config: NumberFieldConfig::default() },
                UiCommand::NumberDragMove { delta_x: 50.0, speed: DragSpeed::Normal },
                UiCommand::NumberDragEnd,
                UiCommand::NumberDragMove { delta_x: 500.0, speed: DragSpeed::Normal },
            ],
            &mut models,
            &mut state,
        );

        assert!((first_transform_x(&models) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn creating_an_object_selects_it() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let before = models.scene.data().objects.len();

        apply_all(vec![UiCommand::CreateObject { parent: None }], &mut models, &mut state);

        assert_eq!(models.scene.data().objects.len(), before + 1);
        assert_eq!(models.hierarchy_ui.data().selected.len(), 1);
    }

    #[test]
    fn a_refused_command_shows_in_the_status_bar() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let player = models.scene.data().objects.iter().find(|o| o.name == "Player").unwrap().uuid;
        let body = models.scene.data().objects.iter().find(|o| o.name == "Body").unwrap().uuid;

        apply_all(vec![UiCommand::Reparent { object: player, new_parent: Some(body) }], &mut models, &mut state);

        assert_ne!(state.status, "Ready");
        assert!(query::find_object(&models.scene.data().objects, &player).unwrap().parent.is_none());
    }

    #[test]
    fn switching_theme_marks_settings_for_saving() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();

        apply_all(vec![UiCommand::SetTheme("Light".to_string())], &mut models, &mut state);

        assert_eq!(active_theme(models.theme.data()).name, "Light");
        assert!(state.settings_dirty);
    }

    #[test]
    fn shortcut_actions_become_commands() {
        let models = EditorModels::new_demo();

        assert!(matches!(action_to_command(actions::DELETE, &models), Some(UiCommand::DeleteSelected)));
        assert!(matches!(action_to_command(actions::SAVE, &models), Some(UiCommand::SaveScene)));
        assert!(action_to_command(actions::PLAY, &models).is_none());
    }

    #[test]
    fn undo_and_redo_bring_a_new_object_back_and_forth() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let before = models.scene.data().objects.len();

        apply_all(vec![UiCommand::CreateObject { parent: None }], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), before + 1);

        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), before);
        assert!(state.status.contains("Undid"));

        apply_all(vec![UiCommand::Redo], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), before + 1);
    }

    #[test]
    fn a_whole_number_drag_is_one_undo_step() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let target = first_target(&models);

        apply_all(
            vec![
                UiCommand::NumberDragBegin { target, value: 0.0, config: NumberFieldConfig::default() },
                UiCommand::NumberDragMove { delta_x: 30.0, speed: DragSpeed::Normal },
                UiCommand::NumberDragMove { delta_x: 30.0, speed: DragSpeed::Normal },
                UiCommand::NumberDragEnd,
            ],
            &mut models,
            &mut state,
        );

        assert_eq!(models.history.data().undo.len(), 1);

        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(first_transform_x(&models), 0.0);
    }

    #[test]
    fn a_cancelled_drag_leaves_no_history() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let target = first_target(&models);

        apply_all(
            vec![
                UiCommand::NumberDragBegin { target, value: 0.0, config: NumberFieldConfig::default() },
                UiCommand::NumberDragMove { delta_x: 30.0, speed: DragSpeed::Normal },
                UiCommand::NumberDragCancel,
            ],
            &mut models,
            &mut state,
        );

        assert!(models.history.data().undo.is_empty());
    }

    #[test]
    fn typing_a_name_is_one_undo_step() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let object = models.scene.data().objects[0].uuid;
        let original = models.scene.data().objects[0].name.clone();
        let rename = |name: &str| UiCommand::RenameObject { object, name: name.to_string() };

        apply_all(vec![rename("A"), rename("Ab"), rename("Abc")], &mut models, &mut state);

        assert_eq!(models.history.data().undo.len(), 1);

        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(models.scene.data().objects[0].name, original);
    }

    #[test]
    fn a_command_that_changes_nothing_is_not_a_step() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();

        // Nothing is selected, so there is nothing to delete.
        apply_all(vec![UiCommand::DeleteSelected], &mut models, &mut state);

        assert!(models.history.data().undo.is_empty());
    }

    #[test]
    fn undoing_a_creation_drops_a_stale_selection_and_redo_brings_the_object_back() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let count = models.scene.data().objects.len();

        apply_all(vec![UiCommand::CreateObject { parent: None }], &mut models, &mut state);
        let created = models.hierarchy_ui.data().selected[0];

        // Undo the creation: the object (and the selection pointing at it) is gone.
        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), count);
        assert!(!models.hierarchy_ui.data().selected.contains(&created));

        // Redo brings the same object, with the same uuid, back.
        apply_all(vec![UiCommand::Redo], &mut models, &mut state);
        assert!(models.scene.data().objects.iter().any(|object| object.uuid == created));
    }

    #[test]
    fn clicking_a_history_row_goes_back_several_steps() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let count = models.scene.data().objects.len();

        for _ in 0..3 {
            apply_all(vec![UiCommand::CreateObject { parent: None }], &mut models, &mut state);
        }

        apply_all(vec![UiCommand::UndoSteps(2)], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), count + 1);

        apply_all(vec![UiCommand::RedoSteps(2)], &mut models, &mut state);
        assert_eq!(models.scene.data().objects.len(), count + 3);
    }

    #[test]
    fn prefab_commands_are_queued_for_the_app() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let prefab = models.scene.data().uuid; // any uuid will do for this check

        apply_all(
            vec![
                UiCommand::CreatePrefab,
                UiCommand::PlacePrefab { file: "Assets/Prefabs/Tree.prefab.yaml".to_string() },
                UiCommand::RevertRevision { prefab, number: 2 },
            ],
            &mut models,
            &mut state,
        );

        assert_eq!(
            state.pending_io,
            vec![
                IoRequest::CreatePrefab,
                IoRequest::PlacePrefab { file: "Assets/Prefabs/Tree.prefab.yaml".to_string() },
                IoRequest::RevertRevision { prefab, number: 2 },
            ]
        );
    }

    #[test]
    fn renaming_in_the_hierarchy_is_one_undo_step_and_cancel_or_an_empty_name_change_nothing() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let object = models.scene.data().objects[0].uuid;
        let original = models.scene.data().objects[0].name.clone();

        apply_all(vec![UiCommand::BeginRename(object)], &mut models, &mut state);
        assert_eq!(models.hierarchy_ui.data().renaming, Some(object));

        apply_all(vec![UiCommand::FinishRename { object, name: "  Camera  ".to_string() }], &mut models, &mut state);
        assert_eq!(models.scene.data().objects[0].name, "Camera");
        assert_eq!(models.hierarchy_ui.data().renaming, None);
        assert_eq!(models.history.data().undo.len(), 1);

        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(models.scene.data().objects[0].name, original);

        apply_all(vec![UiCommand::BeginRename(object), UiCommand::CancelRename], &mut models, &mut state);
        apply_all(
            vec![UiCommand::BeginRename(object), UiCommand::FinishRename { object, name: "   ".to_string() }],
            &mut models,
            &mut state,
        );

        assert_eq!(models.scene.data().objects[0].name, original);
        assert_eq!(models.hierarchy_ui.data().renaming, None);
        assert!(models.history.data().undo.is_empty());
    }

    #[test]
    fn f2_starts_renaming_the_selected_object() {
        let mut models = EditorModels::new_demo();
        let object = models.scene.data().objects[0].uuid;

        assert!(action_to_command(actions::RENAME, &models).is_none()); // nothing selected

        crate::hierarchy::HierarchyController.select_only(&mut models.hierarchy_ui, object);

        assert!(matches!(action_to_command(actions::RENAME, &models), Some(UiCommand::BeginRename(found)) if found == object));
    }

    #[test]
    fn a_script_can_be_removed_and_a_transform_can_be_reset_but_a_transform_cannot_be_removed() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let player = models.scene.data().objects.iter().find(|object| object.name == "Player").unwrap().clone();
        let transform = player.components[0].id;
        let script = player.components[1].id;
        let components = |models: &EditorModels| {
            models.scene.data().objects.iter().find(|object| object.name == "Player").unwrap().components.len()
        };

        // Reset: move the player, then reset the Transform.
        InspectController
            .set_field(
                &mut models.scene,
                &models.schemas,
                &player.uuid,
                transform,
                &FieldPath::field("position"),
                FieldValue::Vec3(flux_engine::math::Vec3::new(9.0, 0.0, 0.0)),
            )
            .unwrap();
        apply_all(vec![UiCommand::ResetComponent { object: player.uuid, component: transform }], &mut models, &mut state);

        let moved_back = models.scene.data().objects.iter().find(|object| object.name == "Player").unwrap();
        assert_eq!(
            flux_engine::inspect::query::read_field(&moved_back.components[0].data, &FieldPath::field("position")),
            Some(FieldValue::Vec3(flux_engine::math::Vec3::ZERO))
        );

        // Remove the script, then undo that.
        apply_all(vec![UiCommand::RemoveComponent { object: player.uuid, component: script }], &mut models, &mut state);
        assert_eq!(components(&models), 1);
        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(components(&models), 2);

        // The Transform stays, and the refusal shows in the status bar.
        apply_all(vec![UiCommand::RemoveComponent { object: player.uuid, component: transform }], &mut models, &mut state);
        assert_eq!(components(&models), 2);
        assert_ne!(state.status, "Ready");
    }

    // ----- the Eclipse Inspector commands, tried on the demo Player script -----

    fn player_script(models: &EditorModels) -> (AssetUuid, ComponentId) {
        let player = models.scene.data().objects.iter().find(|object| object.name == "Player").unwrap();

        (player.uuid, player.components[1].id)
    }

    fn script_value(models: &EditorModels, path: &FieldPath) -> Option<FieldValue> {
        let player = models.scene.data().objects.iter().find(|object| object.name == "Player").unwrap();

        flux_engine::inspect::query::read_field(&player.components[1].data, path)
    }

    #[test]
    fn a_script_field_follows_its_range_and_typing_is_one_undo_step() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let (object, component) = player_script(&models);
        let speed = FieldPath::field("speed");
        let set = |value: f64| UiCommand::SetField { object, component, path: FieldPath::field("speed"), value: FieldValue::Float(value) };

        apply_all(vec![set(50.0), set(3.0), set(4.0)], &mut models, &mut state);

        assert_eq!(script_value(&models, &speed), Some(FieldValue::Float(4.0)));
        assert_eq!(models.history.data().undo.len(), 1);

        apply_all(vec![UiCommand::Undo], &mut models, &mut state);
        assert_eq!(script_value(&models, &speed), Some(FieldValue::Float(5.0))); // the schema's default
    }

    #[test]
    fn a_field_value_of_the_wrong_kind_changes_nothing_and_says_why() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let (object, component) = player_script(&models);

        apply_all(
            vec![UiCommand::SetField { object, component, path: FieldPath::field("speed"), value: FieldValue::Bool(true) }],
            &mut models,
            &mut state,
        );

        assert_eq!(script_value(&models, &FieldPath::field("speed")), Some(FieldValue::Float(5.0)));
        assert!(state.status.contains("can't hold"));
        assert!(models.history.data().undo.is_empty());
    }

    #[test]
    fn list_elements_can_be_added_changed_and_removed_and_a_field_can_be_reset() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let (object, component) = player_script(&models);
        let waypoints = FieldPath::field("waypoints");

        apply_all(
            vec![
                UiCommand::ListPush { object, component, path: waypoints.clone() },
                UiCommand::ListPush { object, component, path: waypoints.clone() },
                UiCommand::SetField {
                    object,
                    component,
                    path: waypoints.child_index(1),
                    value: FieldValue::Vec3(flux_engine::math::Vec3::new(1.0, 2.0, 3.0)),
                },
                UiCommand::ListRemove { object, component, path: waypoints.clone(), index: 0 },
            ],
            &mut models,
            &mut state,
        );

        assert_eq!(
            script_value(&models, &waypoints),
            Some(FieldValue::List(vec![FieldValue::Vec3(flux_engine::math::Vec3::new(1.0, 2.0, 3.0))]))
        );

        apply_all(
            vec![
                UiCommand::SetField { object, component, path: FieldPath::field("speed"), value: FieldValue::Float(9.0) },
                UiCommand::ResetField { object, component, path: FieldPath::field("speed") },
            ],
            &mut models,
            &mut state,
        );

        assert_eq!(script_value(&models, &FieldPath::field("speed")), Some(FieldValue::Float(5.0)));
    }

    #[test]
    fn dragging_a_number_inside_a_struct_of_a_script_changes_it_in_one_undo_step() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let (object, component) = player_script(&models);
        let armor = FieldPath::field("stats").child_field("armor");
        let target = NumberTarget { object, component, path: armor.clone(), part: Part::Whole };

        apply_all(
            vec![
                UiCommand::NumberDragBegin { target, value: 0.0, config: NumberFieldConfig::default() },
                UiCommand::NumberDragMove { delta_x: 100.0, speed: DragSpeed::Normal },
                UiCommand::NumberDragEnd,
            ],
            &mut models,
            &mut state,
        );

        match script_value(&models, &armor) {
            Some(FieldValue::Float(value)) => assert!((value - 10.0).abs() < 1e-9),
            other => panic!("expected a float, got {other:?}"),
        }

        assert_eq!(models.history.data().undo.len(), 1);
    }

    #[test]
    fn presets_and_added_components_show_up_and_a_component_can_be_reset() {
        let mut models = EditorModels::new_demo();
        let mut state = ControllerState::default();
        let fov = FieldPath::field("fov_degrees");
        let read = |models: &EditorModels, object: AssetUuid, index: usize, path: &FieldPath| {
            let found = models.scene.data().objects.iter().find(|candidate| candidate.uuid == object).unwrap();

            flux_engine::inspect::query::read_field(&found.components[index].data, path)
        };

        apply_all(vec![UiCommand::CreatePreset { preset: ObjectPreset::Camera, parent: None }], &mut models, &mut state);

        let camera = models.hierarchy_ui.data().selected[0];
        let component = models.scene.data().objects.iter().find(|o| o.uuid == camera).unwrap().components[1].id;

        assert_eq!(read(&models, camera, 1, &fov), Some(FieldValue::Float(60.0)));

        apply_all(
            vec![UiCommand::SetField { object: camera, component, path: fov.clone(), value: FieldValue::Float(500.0) }],
            &mut models,
            &mut state,
        );
        assert_eq!(read(&models, camera, 1, &fov), Some(FieldValue::Float(179.0))); // kept inside its range

        apply_all(vec![UiCommand::ResetComponent { object: camera, component }], &mut models, &mut state);
        assert_eq!(read(&models, camera, 1, &fov), Some(FieldValue::Float(60.0)));

        let count = |models: &EditorModels| models.scene.data().objects.iter().find(|o| o.uuid == camera).unwrap().components.len();

        apply_all(vec![UiCommand::AddComponent { object: camera, kind: "LightSource".to_string() }], &mut models, &mut state);
        assert_eq!(count(&models), 3);

        apply_all(vec![UiCommand::AddComponent { object: camera, kind: "Nonsense".to_string() }], &mut models, &mut state);
        assert_eq!(count(&models), 3);
        assert_ne!(state.status, "Ready");
    }
}
