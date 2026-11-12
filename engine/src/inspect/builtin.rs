//! The descriptions of the built-in components, and how to read and write their fields by name.
//! Adding a component later means adding one more arm to each of the three functions.
//! (A derive macro that writes this from the struct is a possible later step.)

use super::interface::{FieldAttribute, FieldDescriptor, FieldKind, FieldValue};
use crate::scene::ComponentData;

fn field(name: &str, kind: FieldKind, attributes: Vec<FieldAttribute>) -> FieldDescriptor {
    FieldDescriptor { name: name.to_string(), kind, default: None, attributes }
}

/// The fields of a built-in component. None for a Script (its fields come from its schema).
pub fn builtin_schema(data: &ComponentData) -> Option<Vec<FieldDescriptor>> {
    match data {
        ComponentData::Transform(_) => Some(vec![
            field("position", FieldKind::Vec3, vec![]),
            field("rotation", FieldKind::Vec3, vec![FieldAttribute::Tooltip { text: "Euler angles in degrees".to_string() }]),
            field("scale", FieldKind::Vec3, vec![]),
        ]),
        ComponentData::RectTransform(_) => {
            let unit = || vec![FieldAttribute::Range { min: 0.0, max: 1.0 }];

            Some(vec![
                field("position", FieldKind::Vec2, vec![]),
                field("size", FieldKind::Vec2, vec![]),
                field("anchor_min", FieldKind::Vec2, unit()),
                field("anchor_max", FieldKind::Vec2, unit()),
                field("pivot", FieldKind::Vec2, unit()),
            ])
        }
        ComponentData::Script(_) => None,
    }
}

/// The value of a field of a built-in component.
pub fn builtin_get(data: &ComponentData, name: &str) -> Option<FieldValue> {
    match (data, name) {
        (ComponentData::Transform(t), "position") => Some(FieldValue::Vec3(t.position)),
        (ComponentData::Transform(t), "rotation") => Some(FieldValue::Vec3(t.rotation)),
        (ComponentData::Transform(t), "scale") => Some(FieldValue::Vec3(t.scale)),
        (ComponentData::RectTransform(r), "position") => Some(FieldValue::Vec2(r.position)),
        (ComponentData::RectTransform(r), "size") => Some(FieldValue::Vec2(r.size)),
        (ComponentData::RectTransform(r), "anchor_min") => Some(FieldValue::Vec2(r.anchor_min)),
        (ComponentData::RectTransform(r), "anchor_max") => Some(FieldValue::Vec2(r.anchor_max)),
        (ComponentData::RectTransform(r), "pivot") => Some(FieldValue::Vec2(r.pivot)),
        _ => None,
    }
}

/// Sets a field of a built-in component. The value should already be checked by the rules.
pub fn builtin_set(data: &mut ComponentData, name: &str, value: FieldValue) -> Result<(), String> {
    match (data, name, value) {
        (ComponentData::Transform(t), "position", FieldValue::Vec3(v)) => t.position = v,
        (ComponentData::Transform(t), "rotation", FieldValue::Vec3(v)) => t.rotation = v,
        (ComponentData::Transform(t), "scale", FieldValue::Vec3(v)) => t.scale = v,
        (ComponentData::RectTransform(r), "position", FieldValue::Vec2(v)) => r.position = v,
        (ComponentData::RectTransform(r), "size", FieldValue::Vec2(v)) => r.size = v,
        (ComponentData::RectTransform(r), "anchor_min", FieldValue::Vec2(v)) => r.anchor_min = v,
        (ComponentData::RectTransform(r), "anchor_max", FieldValue::Vec2(v)) => r.anchor_max = v,
        (ComponentData::RectTransform(r), "pivot", FieldValue::Vec2(v)) => r.pivot = v,
        _ => return Err(format!("this component has no field \"{name}\" of that kind")),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::rules::range_of;
    use crate::math::{Vec2, Vec3};
    use crate::scene::{RectTransformData, TransformData};

    #[test]
    fn every_described_field_can_be_read() {
        for data in [
            ComponentData::Transform(TransformData::default()),
            ComponentData::RectTransform(RectTransformData::default()),
        ] {
            for descriptor in builtin_schema(&data).unwrap() {
                assert!(builtin_get(&data, &descriptor.name).is_some(), "{} can't be read", descriptor.name);
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
}
