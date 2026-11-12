use super::access;
use super::builtin::builtin_set;
use super::defaults;
use super::interface::{FieldKind, FieldPath, FieldValue, PathSegment};
use super::model::SchemaLibraryModel;
use super::query::{find_schema, read_field, schema_for};
use super::rules;
use crate::error::{EngineError, EngineResult};
use crate::identity::{AssetUuid, ComponentId};
use crate::mivc::{Controller, Model};
use crate::scene::{query, ComponentData, SceneController, SceneModel};

/// Changes the fields of a component in a scene, with every rule applied:
/// the kind must fit, [Range] / [Min] are kept, enum options are checked, read-only is respected.
/// A refused change leaves the scene as it was.
pub struct InspectController;

impl Controller for InspectController {}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::InvalidOperation(message.into())
}

/// A copy of a component's data (to look at its kind and schema).
fn component_data(scene: &SceneModel, object: &AssetUuid, component: ComponentId) -> EngineResult<ComponentData> {
    let found = query::find_object(&scene.data().objects, object)
        .ok_or_else(|| EngineError::ObjectNotFound(object.to_string()))?;

    found
        .components
        .iter()
        .find(|entry| entry.id == component)
        .map(|entry| entry.data.clone())
        .ok_or(EngineError::ComponentNotFound(component.value()))
}

/// Writes an already checked value into a component.
fn apply_value(data: &mut ComponentData, path: &FieldPath, value: FieldValue) -> Result<(), String> {
    match data {
        ComponentData::Script(script) => access::set_in(&mut script.fields, path, value),
        other => match path.segments() {
            [PathSegment::Field(name)] => builtin_set(other, name, value),
            _ => Err("this component has no nested fields".to_string()),
        },
    }
}

impl InspectController {
    /// The current value of a field (or of a value inside a struct or list) of a component.
    pub fn get_field(
        &self,
        scene: &SceneModel,
        object: &AssetUuid,
        component: ComponentId,
        path: &FieldPath,
    ) -> EngineResult<FieldValue> {
        let data = component_data(scene, object, component)?;

        read_field(&data, path).ok_or_else(|| invalid("that field has no value"))
    }

    /// Sets one field (or one value inside a struct or list) of a component.
    pub fn set_field(
        &self,
        scene: &mut SceneModel,
        library: &SchemaLibraryModel,
        object: &AssetUuid,
        component: ComponentId,
        path: &FieldPath,
        value: FieldValue,
    ) -> EngineResult<()> {
        let data = component_data(scene, object, component)?;
        let schema = schema_for(&data, library.data()).ok_or_else(|| invalid("this component has no field information"))?;
        let resolved = rules::resolve(&schema, path).map_err(invalid)?;

        if resolved.read_only {
            return Err(invalid("that field is read-only"));
        }

        let checked = rules::validate_value(resolved.kind, resolved.attributes, value).map_err(invalid)?;
        let mut outcome: Result<(), String> = Ok(());

        SceneController.update_component(scene, object, component, |data| {
            outcome = apply_value(data, path, checked);
        })?;

        outcome.map_err(invalid)
    }

    /// Puts a field back to its default (the one in its description, or the kind's own).
    pub fn reset_field(
        &self,
        scene: &mut SceneModel,
        library: &SchemaLibraryModel,
        object: &AssetUuid,
        component: ComponentId,
        path: &FieldPath,
    ) -> EngineResult<()> {
        let data = component_data(scene, object, component)?;
        let schema = schema_for(&data, library.data()).ok_or_else(|| invalid("this component has no field information"))?;
        let resolved = rules::resolve(&schema, path).map_err(invalid)?;
        let value = resolved.default.cloned().unwrap_or_else(|| defaults::default_value(resolved.kind));

        // Same rules as any other change (read-only fields can't be reset either).
        self.set_field(scene, library, object, component, path, value)
    }

    /// Adds an element (with the default value of the element kind) to a list field.
    pub fn list_push(
        &self,
        scene: &mut SceneModel,
        library: &SchemaLibraryModel,
        object: &AssetUuid,
        component: ComponentId,
        path: &FieldPath,
    ) -> EngineResult<()> {
        let data = component_data(scene, object, component)?;
        let schema = schema_for(&data, library.data()).ok_or_else(|| invalid("this component has no field information"))?;
        let resolved = rules::resolve(&schema, path).map_err(invalid)?;

        if resolved.read_only {
            return Err(invalid("that field is read-only"));
        }

        let FieldKind::List { element } = resolved.kind else {
            return Err(invalid("that field is not a list"));
        };
        let new_element = defaults::default_value(element);
        let mut outcome: Result<(), String> = Ok(());

        SceneController.update_component(scene, object, component, |data| {
            outcome = match data {
                ComponentData::Script(script) => access::list_push(&mut script.fields, path, new_element),
                _ => Err("built-in components have no lists".to_string()),
            };
        })?;

        outcome.map_err(invalid)
    }

    /// Removes one element of a list field.
    pub fn list_remove(
        &self,
        scene: &mut SceneModel,
        library: &SchemaLibraryModel,
        object: &AssetUuid,
        component: ComponentId,
        path: &FieldPath,
        index: usize,
    ) -> EngineResult<()> {
        let data = component_data(scene, object, component)?;
        let schema = schema_for(&data, library.data()).ok_or_else(|| invalid("this component has no field information"))?;
        let resolved = rules::resolve(&schema, path).map_err(invalid)?;

        if resolved.read_only {
            return Err(invalid("that field is read-only"));
        }

        if !matches!(resolved.kind, FieldKind::List { .. }) {
            return Err(invalid("that field is not a list"));
        }

        let mut outcome: Result<(), String> = Ok(());

        SceneController.update_component(scene, object, component, |data| {
            outcome = match data {
                ComponentData::Script(script) => access::list_remove(&mut script.fields, path, index),
                _ => Err("built-in components have no lists".to_string()),
            };
        })?;

        outcome.map_err(invalid)
    }

    /// Gives a script component a value for every field of its schema that has none yet.
    /// Call it when a script is added or its schema changed. Returns how many fields were added
    /// (0 if the script has no schema).
    pub fn sync_script_fields(
        &self,
        scene: &mut SceneModel,
        library: &SchemaLibraryModel,
        object: &AssetUuid,
        component: ComponentId,
    ) -> EngineResult<usize> {
        let ComponentData::Script(script) = component_data(scene, object, component)? else {
            return Ok(0);
        };
        let Some(schema) = find_schema(library.data(), &script.script) else {
            return Ok(0);
        };
        let fields = schema.fields.clone();
        let mut added = 0;

        SceneController.update_component(scene, object, component, |data| {
            if let ComponentData::Script(script) = data {
                added = defaults::sync_fields(&mut script.fields, &fields);
            }
        })?;

        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::inspect::interface::{FieldAttribute, FieldDescriptor, SchemaLibraryData, ScriptSchemaData};
    use crate::math::{Rgba, Vec2, Vec3};
    use crate::mivc::Store;
    use crate::scene::{ComponentEntry, GameObjectData, RectTransformData, ScriptData};
    use crate::storage::yaml::{from_yaml, to_yaml};

    fn field(name: &str, kind: FieldKind, attributes: Vec<FieldAttribute>) -> FieldDescriptor {
        FieldDescriptor { name: name.to_string(), kind, default: None, attributes }
    }

    fn range(min: f64, max: f64) -> Vec<FieldAttribute> {
        vec![FieldAttribute::Range { min, max }]
    }

    struct World {
        scene: SceneModel,
        ids: IdRegistryModel,
        library: SchemaLibraryModel,
        object: AssetUuid,
        transform: ComponentId,
        script: ComponentId,
    }

    fn schema_fields() -> Vec<FieldDescriptor> {
        vec![
            FieldDescriptor { default: Some(FieldValue::Float(5.0)), ..field("speed", FieldKind::Float, range(0.0, 10.0)) },
            field("can_fly", FieldKind::Bool, vec![]),
            field("mode", FieldKind::Enum { options: vec!["Walk".to_string(), "Run".to_string()] }, vec![]),
            field("locked", FieldKind::Float, vec![FieldAttribute::ReadOnly]),
            field(
                "stats",
                FieldKind::Struct {
                    fields: vec![
                        FieldDescriptor { default: Some(FieldValue::Int(100)), ..field("health", FieldKind::Int, range(0.0, 100.0)) },
                        field("armor", FieldKind::Float, vec![]),
                    ],
                },
                vec![],
            ),
            field("waypoints", FieldKind::List { element: Box::new(FieldKind::Vec3) }, vec![]),
        ]
    }

    fn world() -> World {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let object = SceneController.create_object(&mut scene, &mut ids, "Player", None).unwrap();
        let transform = scene.data().objects[0].components[0].id;
        let script_uuid = IdController.new_uuid(&mut ids);
        let library = Store::new(SchemaLibraryData {
            scripts: vec![ScriptSchemaData { script: script_uuid, class_name: "Player".to_string(), fields: schema_fields() }],
        });
        let script = SceneController
            .add_component(&mut scene, &mut ids, &object, ComponentData::Script(ScriptData::new(script_uuid, true)))
            .unwrap();

        InspectController.sync_script_fields(&mut scene, &library, &object, script).unwrap();

        World { scene, ids, library, object, transform, script }
    }

    fn script_value(world: &World, path: &FieldPath) -> Option<FieldValue> {
        match &world.scene.data().objects[0].components[1].data {
            ComponentData::Script(script) => access::get_in(&script.fields, path).cloned(),
            _ => None,
        }
    }

    fn set(world: &mut World, path: &FieldPath, value: FieldValue) -> EngineResult<()> {
        let (object, script) = (world.object, world.script);

        InspectController.set_field(&mut world.scene, &world.library, &object, script, path, value)
    }

    #[test]
    fn a_new_script_gets_the_defaults_of_its_schema_including_nested_ones() {
        let world = world();

        assert_eq!(script_value(&world, &FieldPath::field("speed")), Some(FieldValue::Float(5.0)));
        assert_eq!(script_value(&world, &FieldPath::field("stats").child_field("health")), Some(FieldValue::Int(100)));
        assert_eq!(script_value(&world, &FieldPath::field("mode")), Some(FieldValue::Enum("Walk".to_string())));
    }

    #[test]
    fn range_keeps_a_value_inside_and_the_rest_of_the_object_is_untouched() {
        let mut world = world();

        set(&mut world, &FieldPath::field("speed"), FieldValue::Float(50.0)).unwrap();

        assert_eq!(script_value(&world, &FieldPath::field("speed")), Some(FieldValue::Float(10.0)));
        assert_eq!(script_value(&world, &FieldPath::field("can_fly")), Some(FieldValue::Bool(false)));
    }

    #[test]
    fn a_refused_change_leaves_the_value_as_it_was() {
        let mut world = world();

        assert!(set(&mut world, &FieldPath::field("speed"), FieldValue::Bool(true)).is_err()); // wrong kind
        assert!(set(&mut world, &FieldPath::field("mode"), FieldValue::Enum("Fly".to_string())).is_err()); // not an option
        assert!(set(&mut world, &FieldPath::field("locked"), FieldValue::Float(1.0)).is_err()); // read-only
        assert!(set(&mut world, &FieldPath::field("missing"), FieldValue::Float(1.0)).is_err()); // no such field

        assert_eq!(script_value(&world, &FieldPath::field("speed")), Some(FieldValue::Float(5.0)));
        assert_eq!(script_value(&world, &FieldPath::field("mode")), Some(FieldValue::Enum("Walk".to_string())));
    }

    #[test]
    fn a_value_inside_a_struct_follows_its_own_range() {
        let mut world = world();

        set(&mut world, &FieldPath::field("stats").child_field("health"), FieldValue::Int(500)).unwrap();

        assert_eq!(script_value(&world, &FieldPath::field("stats").child_field("health")), Some(FieldValue::Int(100)));
    }

    #[test]
    fn list_elements_can_be_added_changed_checked_and_removed() {
        let mut world = world();
        let (object, script) = (world.object, world.script);
        let waypoints = FieldPath::field("waypoints");

        InspectController.list_push(&mut world.scene, &world.library, &object, script, &waypoints).unwrap();
        InspectController.list_push(&mut world.scene, &world.library, &object, script, &waypoints).unwrap();

        assert_eq!(
            script_value(&world, &waypoints),
            Some(FieldValue::List(vec![FieldValue::Vec3(Vec3::ZERO), FieldValue::Vec3(Vec3::ZERO)]))
        );

        set(&mut world, &waypoints.child_index(1), FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0))).unwrap();
        assert!(set(&mut world, &waypoints.child_index(1), FieldValue::Bool(true)).is_err());
        assert!(set(&mut world, &waypoints.child_index(7), FieldValue::Vec3(Vec3::ZERO)).is_err());

        InspectController.list_remove(&mut world.scene, &world.library, &object, script, &waypoints, 0).unwrap();

        assert_eq!(
            script_value(&world, &waypoints),
            Some(FieldValue::List(vec![FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0))]))
        );
        assert!(InspectController.list_remove(&mut world.scene, &world.library, &object, script, &waypoints, 5).is_err());
        assert!(InspectController.list_push(&mut world.scene, &world.library, &object, script, &FieldPath::field("speed")).is_err());
    }

    #[test]
    fn reset_brings_back_the_default_and_the_default_in_the_schema_wins() {
        let mut world = world();
        let (object, script) = (world.object, world.script);

        set(&mut world, &FieldPath::field("speed"), FieldValue::Float(1.0)).unwrap();
        InspectController.reset_field(&mut world.scene, &world.library, &object, script, &FieldPath::field("speed")).unwrap();

        assert_eq!(script_value(&world, &FieldPath::field("speed")), Some(FieldValue::Float(5.0)));
        assert!(InspectController.reset_field(&mut world.scene, &world.library, &object, script, &FieldPath::field("locked")).is_err());
    }

    #[test]
    fn syncing_again_adds_only_what_is_new() {
        let mut world = world();
        let (object, script) = (world.object, world.script);

        assert_eq!(InspectController.sync_script_fields(&mut world.scene, &world.library, &object, script).unwrap(), 0);

        // The schema gains a field.
        let mut fields = schema_fields();
        fields.push(field("nickname", FieldKind::Text, vec![]));
        let script_uuid = world.library.data().scripts[0].script;

        world.library = Store::new(SchemaLibraryData {
            scripts: vec![ScriptSchemaData { script: script_uuid, class_name: "Player".to_string(), fields }],
        });

        assert_eq!(InspectController.sync_script_fields(&mut world.scene, &world.library, &object, script).unwrap(), 1);
        assert_eq!(script_value(&world, &FieldPath::field("nickname")), Some(FieldValue::Text(String::new())));
    }

    #[test]
    fn built_in_components_use_the_same_rules() {
        let mut world = world();
        let (object, transform) = (world.object, world.transform);

        InspectController
            .set_field(&mut world.scene, &world.library, &object, transform, &FieldPath::field("position"), FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();

        match &world.scene.data().objects[0].components[0].data {
            ComponentData::Transform(data) => assert_eq!(data.position, Vec3::new(1.0, 2.0, 3.0)),
            other => panic!("expected a transform, got {other:?}"),
        }

        // A Vec2 doesn't fit a Vec3 field, and a Transform has no nested fields.
        assert!(InspectController
            .set_field(&mut world.scene, &world.library, &object, transform, &FieldPath::field("position"), FieldValue::Vec2(Vec2::ZERO))
            .is_err());
        assert!(InspectController
            .set_field(&mut world.scene, &world.library, &object, transform, &FieldPath::field("position").child_field("x"), FieldValue::Float(1.0))
            .is_err());
    }

    #[test]
    fn rect_transform_anchors_are_clamped_to_zero_to_one() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let object = IdController.new_uuid(&mut ids);
        let component = IdController.new_component_id(&mut ids);

        SceneController.write(&mut scene).objects.push(GameObjectData {
            uuid: object,
            name: "Panel".to_string(),
            active: true,
            parent: None,
            children: Vec::new(),
            components: vec![ComponentEntry { id: component, data: ComponentData::RectTransform(RectTransformData::default()) }],
            prefab_source: None,
        });

        let library = SchemaLibraryModel::default();

        InspectController
            .set_field(&mut scene, &library, &object, component, &FieldPath::field("anchor_min"), FieldValue::Vec2(Vec2::new(2.0, -1.0)))
            .unwrap();

        match &scene.data().objects[0].components[0].data {
            ComponentData::RectTransform(data) => assert_eq!(data.anchor_min, Vec2::new(1.0, 0.0)),
            other => panic!("expected a rect transform, got {other:?}"),
        }
    }

    #[test]
    fn a_script_without_a_schema_can_not_be_edited() {
        let mut world = world();
        let (object, script) = (world.object, world.script);
        let empty = SchemaLibraryModel::default();

        let result = InspectController.set_field(&mut world.scene, &empty, &object, script, &FieldPath::field("speed"), FieldValue::Float(1.0));

        assert!(result.is_err());
        assert_eq!(InspectController.sync_script_fields(&mut world.scene, &empty, &object, script).unwrap(), 0);
    }

    #[test]
    fn script_values_and_schemas_survive_yaml() {
        let mut world = world();

        set(&mut world, &FieldPath::field("stats").child_field("armor"), FieldValue::Float(2.5)).unwrap();
        set(&mut world, &FieldPath::field("mode"), FieldValue::Enum("Run".to_string())).unwrap();
        InspectController.list_push(&mut world.scene, &world.library, &world.object.clone(), world.script, &FieldPath::field("waypoints")).unwrap();

        // The scene, with the script values inside the (flattened, tagged) component data.
        let scene: crate::scene::SceneData = from_yaml(&to_yaml(world.scene.data()).unwrap()).unwrap();
        assert_eq!(&scene, world.scene.data());

        // A schema using every kind of field and attribute.
        let schema = ScriptSchemaData {
            script: world.object,
            class_name: "Everything".to_string(),
            fields: vec![
                FieldDescriptor {
                    default: Some(FieldValue::Color(Rgba::WHITE)),
                    ..field("tint", FieldKind::Color, vec![FieldAttribute::Label { text: "Tint".to_string() }, FieldAttribute::Tooltip { text: "t".to_string() }])
                },
                field("name", FieldKind::Text, vec![FieldAttribute::Multiline { lines: 3 }, FieldAttribute::Group { name: "Info".to_string() }]),
                field("target", FieldKind::ObjectRef, vec![FieldAttribute::ReadOnly, FieldAttribute::Hide]),
                field(
                    "extra",
                    FieldKind::Float,
                    vec![
                        FieldAttribute::Range { min: 0.0, max: 1.0 },
                        FieldAttribute::Min { value: 0.0 },
                        FieldAttribute::Header { text: "H".to_string() },
                        FieldAttribute::Space { height: 4.0 },
                        FieldAttribute::ShowIf { field: "can_fly".to_string(), equals: FieldValue::Bool(true) },
                        FieldAttribute::HideIf { field: "mode".to_string(), equals: FieldValue::Enum("Walk".to_string()) },
                    ],
                ),
                field("items", FieldKind::List { element: Box::new(FieldKind::Struct { fields: schema_fields() }) }, vec![]),
            ],
        };
        let loaded: ScriptSchemaData = from_yaml(&to_yaml(&schema).unwrap()).unwrap();

        assert_eq!(loaded, schema);

    }

    #[test]
    fn a_field_can_be_read_by_path_for_scripts_and_built_in_components() {
        let world = world();

        assert_eq!(
            InspectController.get_field(&world.scene, &world.object, world.script, &FieldPath::field("stats").child_field("health")).unwrap(),
            FieldValue::Int(100)
        );
        assert_eq!(
            InspectController.get_field(&world.scene, &world.object, world.transform, &FieldPath::field("scale")).unwrap(),
            FieldValue::Vec3(Vec3::ONE)
        );
        assert!(InspectController.get_field(&world.scene, &world.object, world.script, &FieldPath::field("missing")).is_err());
    }
}
