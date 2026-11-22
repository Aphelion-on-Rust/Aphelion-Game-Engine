use crate::hierarchy::{HierarchyController, HierarchyUiModel};
use crate::history::HistoryModel;
use crate::keybinds::KeybindModel;
use crate::prefab_library::PrefabLibraryModel;
use crate::scene_view::SceneCameraModel;
use crate::layout::LayoutModel;
use crate::number_field::NumberFieldModel;
use crate::theme::ThemeModel;
use crate::project_io::OpenedProject;
use flux_core::project::ProjectModel;
use flux_engine::control::ControlModel;
use flux_engine::inspect::{
    FieldAttribute, FieldDescriptor, FieldKind, FieldValue, InspectController, SchemaLibraryData, SchemaLibraryModel,
    ScriptSchemaData,
};
use flux_engine::identity::{IdController, IdRegistryModel};
use flux_engine::input::InputModel;
use flux_engine::math::{Rgba, Vec3};
use flux_engine::mivc::{Model, Store};
use flux_engine::scene::{query, ComponentData, ObjectPreset, SceneController, SceneModel, ScriptData};
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
    /// The prefab files of the open project.
    pub library: PrefabLibraryModel,
    /// The camera of the Scene view.
    pub scene_camera: SceneCameraModel,
    /// Aphelion's Control: prefab histories (empty if the project has none).
    pub control: ControlModel,
    /// The fields of every script, for the Inspector.
    pub schemas: SchemaLibraryModel,
    pub number_field: NumberFieldModel,
    pub input: InputModel,
}

const DEMO_ERROR: &str = "building the demo scene can't fail";

/// A ready-made shape with a name, a position and a color, for the demo scene.
fn demo_shape(scene: &mut SceneModel, ids: &mut IdRegistryModel, preset: ObjectPreset, name: &str, position: Vec3, color: Rgba) {
    let object = SceneController.create_from_preset(scene, ids, preset, None).expect(DEMO_ERROR);
    let (transform, mesh) = {
        let found = query::find_object(&scene.data().objects, &object).expect(DEMO_ERROR);

        (found.components[0].id, found.components[1].id)
    };

    SceneController.rename_object(scene, &object, name).expect(DEMO_ERROR);
    SceneController
        .update_component(scene, &object, transform, |data| {
            if let ComponentData::Transform(placement) = data {
                placement.position = position;
            }
        })
        .expect(DEMO_ERROR);
    SceneController
        .update_component(scene, &object, mesh, |data| {
            if let ComponentData::MeshDraw(draw) = data {
                draw.color = color;
            }
        })
        .expect(DEMO_ERROR);
}

fn field(name: &str, kind: FieldKind, attributes: Vec<FieldAttribute>) -> FieldDescriptor {
    FieldDescriptor { name: name.to_string(), kind, default: None, attributes }
}

fn group(name: &str) -> FieldAttribute {
    FieldAttribute::Group { name: name.to_string() }
}

/// The fields of the demo script, so the Inspector has something to show. A real project gets
/// these from the scripting host (step 9), read from the C# class.
fn demo_schema(script: flux_engine::identity::AssetUuid) -> ScriptSchemaData {
    let names = |list: &[&str]| FieldKind::Enum { options: list.iter().map(|name| name.to_string()).collect() };

    ScriptSchemaData {
        script,
        class_name: "PlayerController".to_string(),
        fields: vec![
            FieldDescriptor {
                default: Some(FieldValue::Float(5.0)),
                ..field(
                    "speed",
                    FieldKind::Float,
                    vec![FieldAttribute::Header { text: "Movement".to_string() }, FieldAttribute::Range { min: 0.0, max: 10.0 }],
                )
            },
            field(
                "jump_height",
                FieldKind::Float,
                vec![FieldAttribute::Min { value: 0.0 }, FieldAttribute::Tooltip { text: "In meters".to_string() }],
            ),
            field("can_fly", FieldKind::Bool, vec![]),
            field(
                "fly_speed",
                FieldKind::Float,
                vec![FieldAttribute::ShowIf { field: "can_fly".to_string(), equals: FieldValue::Bool(true) }],
            ),
            field("mode", names(&["Walk", "Run", "Sneak"]), vec![]),
            field("target", FieldKind::ObjectRef, vec![FieldAttribute::Label { text: "Follow Target".to_string() }]),
            field("waypoints", FieldKind::List { element: Box::new(FieldKind::Vec3) }, vec![group("Path")]),
            field(
                "stats",
                FieldKind::Struct {
                    fields: vec![
                        FieldDescriptor {
                            default: Some(FieldValue::Int(100)),
                            ..field("health", FieldKind::Int, vec![FieldAttribute::Range { min: 0.0, max: 100.0 }])
                        },
                        field("armor", FieldKind::Float, vec![FieldAttribute::Min { value: 0.0 }]),
                    ],
                },
                vec![group("Stats")],
            ),
            field("nickname", FieldKind::Text, vec![FieldAttribute::Multiline { lines: 2 }, group("Stats")]),
        ],
    }
}

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

        let camera = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::Camera, None).expect(DEMO_ERROR);
        let sun = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::DirectionalLight, None).expect(DEMO_ERROR);

        SceneController.rename_object(&mut scene, &camera, "Main Camera").expect(DEMO_ERROR);
        SceneController.rename_object(&mut scene, &sun, "Directional Light").expect(DEMO_ERROR);

        let player = new_object(&mut scene, &mut ids, "Player", None);
        new_object(&mut scene, &mut ids, "Body", Some(player));
        new_object(&mut scene, &mut ids, "Weapon", Some(player));

        let script = IdController.new_uuid(&mut ids);
        let schemas: SchemaLibraryModel = Store::new(SchemaLibraryData { scripts: vec![demo_schema(script)] });
        let component = SceneController
            .add_component(&mut scene, &mut ids, &player, ComponentData::Script(ScriptData::new(script, true)))
            .expect(DEMO_ERROR);

        InspectController.sync_script_fields(&mut scene, &schemas, &player, component).expect(DEMO_ERROR);

        HierarchyController.toggle_expanded(&mut hierarchy_ui, player);

        demo_shape(&mut scene, &mut ids, ObjectPreset::Plane, "Ground", Vec3::ZERO, Rgba::new(0.35, 0.40, 0.35, 1.0));
        demo_shape(&mut scene, &mut ids, ObjectPreset::Cube, "Crate", Vec3::new(-2.0, 0.5, 0.0), Rgba::new(0.80, 0.45, 0.25, 1.0));
        demo_shape(&mut scene, &mut ids, ObjectPreset::Sphere, "Ball", Vec3::new(2.0, 0.5, 0.0), Rgba::new(0.30, 0.50, 0.85, 1.0));

        Self {
            project: None,
            scene,
            ids,
            layout: LayoutModel::default(),
            theme: ThemeModel::default(),
            keybinds: KeybindModel::default(),
            hierarchy_ui,
            history: HistoryModel::default(),
            library: PrefabLibraryModel::default(),
            scene_camera: SceneCameraModel::default(),
            control: ControlModel::default(),
            schemas,
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
            library: opened.library,
            scene_camera: SceneCameraModel::default(),
            control: opened.control,
            schemas: opened.schemas,
            number_field: NumberFieldModel::default(),
            input: InputModel::default(),
        }
    }
}
