//! The descriptions of the built-in components, and how to read and write their fields by name.
//!
//! Every field is written once: its name, kind, attributes, and how to get and set it on the
//! component. The description, the reader and the writer are all made from that one list.
//! A new built-in component adds one list and one line in each of the three functions at the bottom.
//! (A derive macro that writes the list from the struct could come later.)

use super::interface::{FieldAttribute, FieldDescriptor, FieldKind, FieldValue};
use crate::components::{LightData, LightKind, MeshDrawData, PrimitiveMesh, Projection, SpriteDrawData, ViewCameraData};
use crate::math::{Rgba, Vec2, Vec3};
use crate::scene::{ComponentData, RectTransformData, TransformData};
use std::sync::OnceLock;

type Getter<T> = Box<dyn Fn(&T) -> FieldValue + Send + Sync>;
type Setter<T> = Box<dyn Fn(&mut T, FieldValue) -> Result<(), String> + Send + Sync>;

/// One field of a component of type `T`.
struct Field<T> {
    descriptor: FieldDescriptor,
    get: Getter<T>,
    set: Setter<T>,
}

fn wrong_kind(name: &str) -> String {
    format!("this component has no field \"{name}\" of that kind")
}

fn field<T>(name: &str, kind: FieldKind, attributes: Vec<FieldAttribute>, get: Getter<T>, set: Setter<T>) -> Field<T> {
    Field { descriptor: FieldDescriptor { name: name.to_string(), kind, default: None, attributes }, get, set }
}

fn float<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> f32, set: fn(&mut T, f32)) -> Field<T> {
    field(
        name,
        FieldKind::Float,
        attributes,
        Box::new(move |data| FieldValue::Float(get(data) as f64)),
        Box::new(move |data, value| match value {
            FieldValue::Float(number) => {
                set(data, number as f32);
                Ok(())
            }
            FieldValue::Int(number) => {
                set(data, number as f32);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn int<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> i32, set: fn(&mut T, i32)) -> Field<T> {
    field(
        name,
        FieldKind::Int,
        attributes,
        Box::new(move |data| FieldValue::Int(get(data) as i64)),
        Box::new(move |data, value| match value {
            FieldValue::Int(number) => {
                set(data, number as i32);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn boolean<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> bool, set: fn(&mut T, bool)) -> Field<T> {
    field(
        name,
        FieldKind::Bool,
        attributes,
        Box::new(move |data| FieldValue::Bool(get(data))),
        Box::new(move |data, value| match value {
            FieldValue::Bool(flag) => {
                set(data, flag);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn color<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> Rgba, set: fn(&mut T, Rgba)) -> Field<T> {
    field(
        name,
        FieldKind::Color,
        attributes,
        Box::new(move |data| FieldValue::Color(get(data))),
        Box::new(move |data, value| match value {
            FieldValue::Color(rgba) => {
                set(data, rgba);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn vec2<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> Vec2, set: fn(&mut T, Vec2)) -> Field<T> {
    field(
        name,
        FieldKind::Vec2,
        attributes,
        Box::new(move |data| FieldValue::Vec2(get(data))),
        Box::new(move |data, value| match value {
            FieldValue::Vec2(vector) => {
                set(data, vector);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn vec3<T: 'static>(name: &'static str, attributes: Vec<FieldAttribute>, get: fn(&T) -> Vec3, set: fn(&mut T, Vec3)) -> Field<T> {
    field(
        name,
        FieldKind::Vec3,
        attributes,
        Box::new(move |data| FieldValue::Vec3(get(data))),
        Box::new(move |data, value| match value {
            FieldValue::Vec3(vector) => {
                set(data, vector);
                Ok(())
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

/// A field that is one of a fixed list of names. `set` returns false for a name it doesn't know.
fn choice<T: 'static>(
    name: &'static str,
    options: &'static [&'static str],
    attributes: Vec<FieldAttribute>,
    get: fn(&T) -> &'static str,
    set: fn(&mut T, &str) -> bool,
) -> Field<T> {
    field(
        name,
        FieldKind::Enum { options: options.iter().map(|option| option.to_string()).collect() },
        attributes,
        Box::new(move |data| FieldValue::Enum(get(data).to_string())),
        Box::new(move |data, value| match value {
            FieldValue::Enum(text) => {
                if set(data, &text) {
                    Ok(())
                } else {
                    Err(format!("\"{text}\" is not an option of {name}"))
                }
            }
            _ => Err(wrong_kind(name)),
        }),
    )
}

fn label(text: &str) -> FieldAttribute {
    FieldAttribute::Label { text: text.to_string() }
}

fn tooltip(text: &str) -> FieldAttribute {
    FieldAttribute::Tooltip { text: text.to_string() }
}

fn range(min: f64, max: f64) -> FieldAttribute {
    FieldAttribute::Range { min, max }
}

fn min(value: f64) -> FieldAttribute {
    FieldAttribute::Min { value }
}

fn show_if(field: &str, option: &str) -> FieldAttribute {
    FieldAttribute::ShowIf { field: field.to_string(), equals: FieldValue::Enum(option.to_string()) }
}

fn hide_if(field: &str, option: &str) -> FieldAttribute {
    FieldAttribute::HideIf { field: field.to_string(), equals: FieldValue::Enum(option.to_string()) }
}

// ----- the fields of each component -----

fn transform_fields() -> &'static [Field<TransformData>] {
    static FIELDS: OnceLock<Vec<Field<TransformData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            vec3("position", vec![], |t: &TransformData| t.position, |t, v| t.position = v),
            vec3("rotation", vec![tooltip("Euler angles in degrees")], |t: &TransformData| t.rotation, |t, v| t.rotation = v),
            vec3("scale", vec![], |t: &TransformData| t.scale, |t, v| t.scale = v),
        ]
    })
}

fn rect_transform_fields() -> &'static [Field<RectTransformData>] {
    static FIELDS: OnceLock<Vec<Field<RectTransformData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            vec2("position", vec![], |r: &RectTransformData| r.position, |r, v| r.position = v),
            vec2("size", vec![], |r: &RectTransformData| r.size, |r, v| r.size = v),
            vec2("anchor_min", vec![range(0.0, 1.0)], |r: &RectTransformData| r.anchor_min, |r, v| r.anchor_min = v),
            vec2("anchor_max", vec![range(0.0, 1.0)], |r: &RectTransformData| r.anchor_max, |r, v| r.anchor_max = v),
            vec2("pivot", vec![range(0.0, 1.0)], |r: &RectTransformData| r.pivot, |r, v| r.pivot = v),
        ]
    })
}

fn camera_fields() -> &'static [Field<ViewCameraData>] {
    static FIELDS: OnceLock<Vec<Field<ViewCameraData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            choice(
                "projection",
                Projection::NAMES,
                vec![],
                |c: &ViewCameraData| c.projection.name(),
                |c, name| Projection::from_name(name).map(|projection| c.projection = projection).is_some(),
            ),
            float(
                "fov_degrees",
                vec![label("Field of View"), range(1.0, 179.0), show_if("projection", "Perspective")],
                |c: &ViewCameraData| c.fov_degrees,
                |c, v| c.fov_degrees = v,
            ),
            float(
                "ortho_size",
                vec![label("Size"), min(0.01), show_if("projection", "Orthographic")],
                |c: &ViewCameraData| c.ortho_size,
                |c, v| c.ortho_size = v,
            ),
            float("near", vec![min(0.001)], |c: &ViewCameraData| c.near, |c, v| c.near = v),
            float("far", vec![min(0.01)], |c: &ViewCameraData| c.far, |c, v| c.far = v),
            color("clear_color", vec![label("Background")], |c: &ViewCameraData| c.clear_color, |c, v| c.clear_color = v),
            int(
                "priority",
                vec![tooltip("The camera with the highest priority is the one the Game view shows")],
                |c: &ViewCameraData| c.priority,
                |c, v| c.priority = v,
            ),
        ]
    })
}

fn light_fields() -> &'static [Field<LightData>] {
    static FIELDS: OnceLock<Vec<Field<LightData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            choice(
                "kind",
                LightKind::NAMES,
                vec![],
                |l: &LightData| l.kind.name(),
                |l, name| LightKind::from_name(name).map(|kind| l.kind = kind).is_some(),
            ),
            color("color", vec![], |l: &LightData| l.color, |l, v| l.color = v),
            float("intensity", vec![min(0.0)], |l: &LightData| l.intensity, |l, v| l.intensity = v),
            float("range", vec![min(0.0), hide_if("kind", "Directional")], |l: &LightData| l.range, |l, v| l.range = v),
            float(
                "spot_angle_degrees",
                vec![label("Spot Angle"), range(1.0, 179.0), show_if("kind", "Spot")],
                |l: &LightData| l.spot_angle_degrees,
                |l, v| l.spot_angle_degrees = v,
            ),
            boolean("shadows", vec![], |l: &LightData| l.shadows, |l, v| l.shadows = v),
        ]
    })
}

fn mesh_draw_fields() -> &'static [Field<MeshDrawData>] {
    static FIELDS: OnceLock<Vec<Field<MeshDrawData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            choice(
                "mesh",
                PrimitiveMesh::NAMES,
                vec![],
                |m: &MeshDrawData| m.mesh.name(),
                |m, name| PrimitiveMesh::from_name(name).map(|mesh| m.mesh = mesh).is_some(),
            ),
            color("color", vec![], |m: &MeshDrawData| m.color, |m, v| m.color = v),
            boolean(
                "unlit",
                vec![tooltip("Draw the color as it is, ignoring lights")],
                |m: &MeshDrawData| m.unlit,
                |m, v| m.unlit = v,
            ),
            boolean("cast_shadows", vec![], |m: &MeshDrawData| m.cast_shadows, |m, v| m.cast_shadows = v),
            boolean("receive_shadows", vec![], |m: &MeshDrawData| m.receive_shadows, |m, v| m.receive_shadows = v),
            boolean("visible", vec![], |m: &MeshDrawData| m.visible, |m, v| m.visible = v),
        ]
    })
}

fn sprite_draw_fields() -> &'static [Field<SpriteDrawData>] {
    static FIELDS: OnceLock<Vec<Field<SpriteDrawData>>> = OnceLock::new();

    FIELDS.get_or_init(|| {
        vec![
            color("color", vec![], |s: &SpriteDrawData| s.color, |s, v| s.color = v),
            boolean("flip_x", vec![], |s: &SpriteDrawData| s.flip_x, |s, v| s.flip_x = v),
            boolean("flip_y", vec![], |s: &SpriteDrawData| s.flip_y, |s, v| s.flip_y = v),
            int(
                "order",
                vec![tooltip("Sprites with a higher order are drawn on top")],
                |s: &SpriteDrawData| s.order,
                |s, v| s.order = v,
            ),
            vec2("size", vec![min(0.0), tooltip("Width and height in world units")], |s: &SpriteDrawData| s.size, |s, v| s.size = v),
            boolean("visible", vec![], |s: &SpriteDrawData| s.visible, |s, v| s.visible = v),
        ]
    })
}

// ----- the three functions the rest of the engine uses -----

fn describe_all<T>(fields: &[Field<T>]) -> Vec<FieldDescriptor> {
    fields.iter().map(|field| field.descriptor.clone()).collect()
}

fn get_from<T>(fields: &[Field<T>], data: &T, name: &str) -> Option<FieldValue> {
    fields.iter().find(|field| field.descriptor.name == name).map(|field| (field.get)(data))
}

fn set_in<T>(fields: &[Field<T>], data: &mut T, name: &str, value: FieldValue) -> Result<(), String> {
    let field = fields.iter().find(|field| field.descriptor.name == name).ok_or_else(|| wrong_kind(name))?;

    (field.set)(data, value)
}

/// The fields of a built-in component. None for a Script (its fields come from its schema).
pub fn builtin_schema(data: &ComponentData) -> Option<Vec<FieldDescriptor>> {
    Some(match data {
        ComponentData::Transform(_) => describe_all(transform_fields()),
        ComponentData::RectTransform(_) => describe_all(rect_transform_fields()),
        ComponentData::ViewCamera(_) => describe_all(camera_fields()),
        ComponentData::LightSource(_) => describe_all(light_fields()),
        ComponentData::MeshDraw(_) => describe_all(mesh_draw_fields()),
        ComponentData::SpriteDraw(_) => describe_all(sprite_draw_fields()),
        ComponentData::Script(_) => return None,
    })
}

/// The value of a field of a built-in component.
pub fn builtin_get(data: &ComponentData, name: &str) -> Option<FieldValue> {
    match data {
        ComponentData::Transform(d) => get_from(transform_fields(), d, name),
        ComponentData::RectTransform(d) => get_from(rect_transform_fields(), d, name),
        ComponentData::ViewCamera(d) => get_from(camera_fields(), d, name),
        ComponentData::LightSource(d) => get_from(light_fields(), d, name),
        ComponentData::MeshDraw(d) => get_from(mesh_draw_fields(), d, name),
        ComponentData::SpriteDraw(d) => get_from(sprite_draw_fields(), d, name),
        ComponentData::Script(_) => None,
    }
}

/// Sets a field of a built-in component. The value should already be checked by the rules.
pub fn builtin_set(data: &mut ComponentData, name: &str, value: FieldValue) -> Result<(), String> {
    match data {
        ComponentData::Transform(d) => set_in(transform_fields(), d, name, value),
        ComponentData::RectTransform(d) => set_in(rect_transform_fields(), d, name, value),
        ComponentData::ViewCamera(d) => set_in(camera_fields(), d, name, value),
        ComponentData::LightSource(d) => set_in(light_fields(), d, name, value),
        ComponentData::MeshDraw(d) => set_in(mesh_draw_fields(), d, name, value),
        ComponentData::SpriteDraw(d) => set_in(sprite_draw_fields(), d, name, value),
        ComponentData::Script(_) => Err("a script's fields come from its schema".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::rules::range_of;

    fn all_built_in() -> Vec<ComponentData> {
        vec![
            ComponentData::Transform(TransformData::default()),
            ComponentData::RectTransform(RectTransformData::default()),
            ComponentData::ViewCamera(ViewCameraData::default()),
            ComponentData::LightSource(LightData::default()),
            ComponentData::MeshDraw(MeshDrawData::default()),
            ComponentData::SpriteDraw(SpriteDrawData::default()),
        ]
    }

    #[test]
    fn every_described_field_can_be_read_and_the_same_value_can_be_written_back() {
        for mut data in all_built_in() {
            for descriptor in builtin_schema(&data).unwrap() {
                let value = builtin_get(&data, &descriptor.name)
                    .unwrap_or_else(|| panic!("{} of {} can't be read", descriptor.name, data.type_name()));

                builtin_set(&mut data, &descriptor.name, value.clone())
                    .unwrap_or_else(|error| panic!("{} of {}: {error}", descriptor.name, data.type_name()));
                assert_eq!(builtin_get(&data, &descriptor.name), Some(value));
            }
        }
    }

    #[test]
    fn a_field_can_be_set_and_read_back_but_only_with_the_right_kind() {
        let mut data = ComponentData::Transform(TransformData::default());

        builtin_set(&mut data, "position", FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0))).unwrap();

        assert_eq!(builtin_get(&data, "position"), Some(FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0))));
        assert!(builtin_set(&mut data, "position", FieldValue::Vec2(Vec2::ZERO)).is_err());
        assert!(builtin_set(&mut data, "nope", FieldValue::Vec3(Vec3::ZERO)).is_err());
    }

    #[test]
    fn rect_transform_anchors_are_limited_to_zero_to_one() {
        let data = ComponentData::RectTransform(RectTransformData::default());
        let schema = builtin_schema(&data).unwrap();
        let anchor = schema.iter().find(|descriptor| descriptor.name == "anchor_min").unwrap();

        assert_eq!(range_of(&anchor.attributes), Some((0.0, 1.0)));
    }

    #[test]
    fn a_script_has_no_built_in_fields() {
        let script = ComponentData::Script(crate::scene::ScriptData::new(
            crate::identity::IdController.new_uuid(&mut crate::identity::IdRegistryModel::default()),
            true,
        ));

        assert!(builtin_schema(&script).is_none());
    }

    #[test]
    fn an_enum_field_takes_its_option_names_and_refuses_others() {
        let mut data = ComponentData::ViewCamera(ViewCameraData::default());

        builtin_set(&mut data, "projection", FieldValue::Enum("Orthographic".to_string())).unwrap();

        match &data {
            ComponentData::ViewCamera(camera) => assert_eq!(camera.projection, Projection::Orthographic),
            other => panic!("expected a camera, got {other:?}"),
        }

        assert!(builtin_set(&mut data, "projection", FieldValue::Enum("Cylindrical".to_string())).is_err());
    }

    #[test]
    fn the_camera_shows_the_field_of_view_only_for_a_perspective_projection() {
        let schema = builtin_schema(&ComponentData::ViewCamera(ViewCameraData::default())).unwrap();
        let fov = schema.iter().find(|descriptor| descriptor.name == "fov_degrees").unwrap();
        let perspective = |name: &str| (name == "projection").then(|| FieldValue::Enum("Perspective".to_string()));
        let orthographic = |name: &str| (name == "projection").then(|| FieldValue::Enum("Orthographic".to_string()));

        assert!(crate::inspect::query::is_visible(fov, &perspective));
        assert!(!crate::inspect::query::is_visible(fov, &orthographic));
    }
}
