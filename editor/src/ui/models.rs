use crate::hierarchy::{HierarchyController, HierarchyUiModel};
use crate::history::HistoryModel;
use crate::keybinds::KeybindModel;
use crate::layout::LayoutModel;
use crate::number_field::NumberFieldModel;
use crate::theme::ThemeModel;
use crate::project_io::OpenedProject;
use flux_core::project::ProjectModel;
use flux_engine::identity::{IdController, IdRegistryModel};
use flux_engine::input::InputModel;
use flux_engine::mivc::Store;
use flux_engine::scene::{ComponentData, SceneController, SceneModel, ScriptData};
use std::path::PathBuf;

/// The project that is open: where it is, and its `project.yaml`.
pub struct LoadedProject {
    pub root: PathBuf,
    /// The scene file the editor saves to.
    pub scene_path: PathBuf,
    pub data: ProjectModel,
}

/// Every model the editor uses, in one place. Views get this as `&EditorModels`.
pub struct EditorModels {
    /// None when the editor was started without a project (demo scene).
    pub project: Option<LoadedProject>,
    pub scene: SceneModel,
    pub ids: IdRegistryModel,
    pub layout: LayoutModel,
    pub theme: ThemeModel,
    pub keybinds: KeybindModel,
    pub hierarchy_ui: HierarchyUiModel,
    pub history: HistoryModel,
    pub number_field: NumberFieldModel,
    pub input: InputModel,
}

const DEMO_ERROR: &str = "building the demo scene can't fail";

impl EditorModels {
    /// Editor models with a small sample scene, so there is something to look at.
    /// Real projects arrive with the launcher (step 5).
    pub fn new_demo() -> Self {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Sample Scene");
        let mut hierarchy_ui = HierarchyUiModel::default();

        let new_object = |scene: &mut SceneModel, ids: &mut IdRegistryModel, name: &str, parent| {
            SceneController.create_object(scene, ids, name, parent).expect(DEMO_ERROR)
        };

        new_object(&mut scene, &mut ids, "Main Camera", None);
        new_object(&mut scene, &mut ids, "Directional Light", None);

        let player = new_object(&mut scene, &mut ids, "Player", None);
        new_object(&mut scene, &mut ids, "Body", Some(player));
        new_object(&mut scene, &mut ids, "Weapon", Some(player));

        let script = IdController.new_uuid(&mut ids);
        SceneController
            .add_component(&mut scene, &mut ids, &player, ComponentData::Script(ScriptData { script, enabled: true }))
            .expect(DEMO_ERROR);

        HierarchyController.toggle_expanded(&mut hierarchy_ui, player);

        Self {
            project: None,
            scene,
            ids,
            layout: LayoutModel::default(),
            theme: ThemeModel::default(),
            keybinds: KeybindModel::default(),
            hierarchy_ui,
            history: HistoryModel::default(),
            number_field: NumberFieldModel::default(),
            input: InputModel::default(),
        }
    }

    /// Editor models for a project that was opened from disk.
    pub fn from_opened(opened: OpenedProject) -> Self {
        Self {
            project: Some(LoadedProject {
                root: opened.root,
                scene_path: opened.scene_path,
                data: Store::new(opened.data),
            }),
            scene: opened.scene,
            ids: opened.ids,
            layout: LayoutModel::default(),
            theme: ThemeModel::default(),
            keybinds: KeybindModel::default(),
            hierarchy_ui: HierarchyUiModel::default(),
            history: HistoryModel::default(),
            number_field: NumberFieldModel::default(),
            input: InputModel::default(),
        }
    }
}
