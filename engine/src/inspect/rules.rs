//! What a value may be. These are the rules behind [Range], [Min], enum options, read-only
//! and the kind of a field.

use super::interface::{FieldAttribute, FieldDescriptor, FieldKind, FieldPath, FieldValue, PathSegment};
use crate::math::{Vec2, Vec3};

pub fn range_of(attributes: &[FieldAttribute]) -> Option<(f64, f64)> {
    attributes.iter().find_map(|attribute| match attribute {
        FieldAttribute::Range { min, max } => Some((*min, *max)),
        _ => None,
    })
}

pub fn min_of(attributes: &[FieldAttribute]) -> Option<f64> {
    attributes.iter().find_map(|attribute| match attribute {
        FieldAttribute::Min { value } => Some(*value),
        _ => None,
    })
}

pub fn is_read_only(attributes: &[FieldAttribute]) -> bool {
    attributes.iter().any(|attribute| matches!(attribute, FieldAttribute::ReadOnly))
}

/// Keeps a number inside [Range] and above [Min].
pub fn clamp(value: f64, attributes: &[FieldAttribute]) -> f64 {
    let mut value = value;

    if let Some((a, b)) = range_of(attributes) {
        value = value.clamp(a.min(b), a.max(b));
    }

    if let Some(min) = min_of(attributes) {
        value = value.max(min);
    }

    value
}

fn clamp_f32(value: f32, attributes: &[FieldAttribute]) -> f32 {
    clamp(value as f64, attributes) as f32
}

pub fn kind_name(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::Bool => "bool",
        FieldKind::Int => "int",
        FieldKind::Float => "float",
        FieldKind::Text => "text",
        FieldKind::Vec2 => "Vec2",
        FieldKind::Vec3 => "Vec3",
        FieldKind::Color => "color",
        FieldKind::Enum { .. } => "enum",
        FieldKind::ObjectRef => "object reference",
        FieldKind::List { .. } => "list",
        FieldKind::Struct { .. } => "struct",
    }
}

fn value_name(value: &FieldValue) -> &'static str {
    match value {
        FieldValue::Bool(_) => "a bool",
        FieldValue::Int(_) => "an int",
        FieldValue::Float(_) => "a float",
        FieldValue::Text(_) => "text",
        FieldValue::Vec2(_) => "a Vec2",
        FieldValue::Vec3(_) => "a Vec3",
        FieldValue::Color(_) => "a color",
        FieldValue::Enum(_) => "an enum value",
        FieldValue::ObjectRef(_) => "an object reference",
        FieldValue::List(_) => "a list",
        FieldValue::Struct(_) => "a struct",
    }
}

/// Checks a value against the kind of its field and fixes what can be fixed:
/// numbers are kept inside [Range] / [Min] (an int is rounded), an int is accepted for a float.
/// Returns the value to store, or the reason it is refused.
pub fn validate_value(kind: &FieldKind, attributes: &[FieldAttribute], value: FieldValue) -> Result<FieldValue, String> {
    match (kind, value) {
        (FieldKind::Bool, FieldValue::Bool(flag)) => Ok(FieldValue::Bool(flag)),
        (FieldKind::Int, FieldValue::Int(number)) => Ok(FieldValue::Int(clamp(number as f64, attributes).round() as i64)),
        (FieldKind::Float, FieldValue::Float(number)) => {
            if number.is_nan() {
                Err("that is not a number".to_string())
            } else {
                Ok(FieldValue::Float(clamp(number, attributes)))
            }
        }
        (FieldKind::Float, FieldValue::Int(number)) => Ok(FieldValue::Float(clamp(number as f64, attributes))),
        (FieldKind::Text, FieldValue::Text(text)) => Ok(FieldValue::Text(text)),
        (FieldKind::Vec2, FieldValue::Vec2(v)) => {
            Ok(FieldValue::Vec2(Vec2::new(clamp_f32(v.x, attributes), clamp_f32(v.y, attributes))))
        }
        (FieldKind::Vec3, FieldValue::Vec3(v)) => Ok(FieldValue::Vec3(Vec3::new(
            clamp_f32(v.x, attributes),
            clamp_f32(v.y, attributes),
            clamp_f32(v.z, attributes),
        ))),
        (FieldKind::Color, FieldValue::Color(color)) => Ok(FieldValue::Color(color)),
        (FieldKind::Enum { options }, FieldValue::Enum(name)) => {
            if options.contains(&name) {
                Ok(FieldValue::Enum(name))
            } else {
                Err(format!("\"{name}\" is not one of: {}", options.join(", ")))
            }
        }
        (FieldKind::ObjectRef, FieldValue::ObjectRef(target)) => Ok(FieldValue::ObjectRef(target)),
        (FieldKind::List { element }, FieldValue::List(items)) => items
            .into_iter()
            .map(|item| validate_value(element, attributes, item))
            .collect::<Result<Vec<_>, _>>()
            .map(FieldValue::List),
        (FieldKind::Struct { fields }, FieldValue::Struct(entries)) => {
            let mut checked = Vec::with_capacity(entries.len());

            for mut entry in entries {
                let descriptor = fields
                    .iter()
                    .find(|descriptor| descriptor.name == entry.name)
                    .ok_or_else(|| format!("there is no field called \"{}\"", entry.name))?;

                entry.value = validate_value(&descriptor.kind, &descriptor.attributes, entry.value)?;
                checked.push(entry);
            }

            Ok(FieldValue::Struct(checked))
        }
        (kind, value) => Err(format!("a {} field can't hold {}", kind_name(kind), value_name(&value))),
    }
}

/// What a path leads to: the kind of the value, the attributes that apply to it,
/// whether it (or anything around it) is read-only, and the default of its descriptor.
pub struct Resolved<'a> {
    pub kind: &'a FieldKind,
    pub attributes: &'a [FieldAttribute],
    pub read_only: bool,
    pub default: Option<&'a FieldValue>,
}

fn from_descriptor<'a>(descriptor: &'a FieldDescriptor, parent_read_only: bool) -> Resolved<'a> {
    Resolved {
        kind: &descriptor.kind,
        attributes: &descriptor.attributes,
        read_only: parent_read_only || is_read_only(&descriptor.attributes),
        default: descriptor.default.as_ref(),
    }
}

/// Follows a path through the field descriptions. The elements of a list use the attributes
/// of the list field, so [Range] on a list of floats keeps every element in range.
pub fn resolve<'a>(descriptors: &'a [FieldDescriptor], path: &FieldPath) -> Result<Resolved<'a>, String> {
    let mut segments = path.segments().iter();

    let Some(PathSegment::Field(name)) = segments.next() else {
        return Err("a field path must start with a field name".to_string());
    };

    let top = descriptors
        .iter()
        .find(|descriptor| descriptor.name == *name)
        .ok_or_else(|| format!("there is no field called \"{name}\""))?;
    let mut current = from_descriptor(top, false);

    for segment in segments {
        current = match (current.kind, segment) {
            (FieldKind::Struct { fields }, PathSegment::Field(name)) => {
                let child = fields
                    .iter()
                    .find(|descriptor| descriptor.name == *name)
                    .ok_or_else(|| format!("there is no field called \"{name}\""))?;

                from_descriptor(child, current.read_only)
            }
            (FieldKind::List { element }, PathSegment::Index(_)) => Resolved {
                kind: element,
                attributes: current.attributes,
                read_only: current.read_only,
                default: None,
            },
            _ => return Err("that path does not lead to a field".to_string()),
        };
    }

    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::interface::FieldEntry;

    fn range(min: f64, max: f64) -> Vec<FieldAttribute> {
        vec![FieldAttribute::Range { min, max }]
    }

    fn descriptor(name: &str, kind: FieldKind, attributes: Vec<FieldAttribute>) -> FieldDescriptor {
        FieldDescriptor { name: name.to_string(), kind, default: None, attributes }
    }

    #[test]
    fn range_and_min_keep_numbers_inside() {
        assert_eq!(clamp(15.0, &range(0.0, 10.0)), 10.0);
        assert_eq!(clamp(-5.0, &range(0.0, 10.0)), 0.0);
        assert_eq!(clamp(-5.0, &[FieldAttribute::Min { value: 0.0 }]), 0.0);
        assert_eq!(clamp(7.0, &[]), 7.0);
        // A range written the wrong way round still works.
        assert_eq!(clamp(15.0, &range(10.0, 0.0)), 10.0);
    }

    #[test]
    fn ints_are_rounded_and_kept_in_range_and_an_int_fits_a_float() {
        assert_eq!(validate_value(&FieldKind::Int, &range(0.0, 100.0), FieldValue::Int(250)), Ok(FieldValue::Int(100)));
        assert_eq!(validate_value(&FieldKind::Float, &[], FieldValue::Int(3)), Ok(FieldValue::Float(3.0)));
        assert!(validate_value(&FieldKind::Float, &[], FieldValue::Float(f64::NAN)).is_err());
    }

    #[test]
    fn vectors_are_clamped_per_axis() {
        let checked = validate_value(&FieldKind::Vec2, &range(0.0, 1.0), FieldValue::Vec2(Vec2::new(2.0, -1.0)));

        assert_eq!(checked, Ok(FieldValue::Vec2(Vec2::new(1.0, 0.0))));
    }

    #[test]
    fn the_wrong_kind_is_refused_with_a_clear_reason() {
        let error = validate_value(&FieldKind::Bool, &[], FieldValue::Int(1)).unwrap_err();

        assert_eq!(error, "a bool field can't hold an int");
    }

    #[test]
    fn an_enum_only_takes_its_options() {
        let kind = FieldKind::Enum { options: vec!["Walk".to_string(), "Run".to_string()] };

        assert!(validate_value(&kind, &[], FieldValue::Enum("Run".to_string())).is_ok());
        assert!(validate_value(&kind, &[], FieldValue::Enum("Fly".to_string())).is_err());
    }

    #[test]
    fn lists_and_structs_are_checked_all_the_way_down() {
        let list = FieldKind::List { element: Box::new(FieldKind::Float) };
        let checked = validate_value(&list, &range(0.0, 1.0), FieldValue::List(vec![FieldValue::Float(5.0), FieldValue::Float(0.5)]));

        assert_eq!(checked, Ok(FieldValue::List(vec![FieldValue::Float(1.0), FieldValue::Float(0.5)])));
        assert!(validate_value(&list, &[], FieldValue::List(vec![FieldValue::Bool(true)])).is_err());

        let stats = FieldKind::Struct { fields: vec![descriptor("health", FieldKind::Int, range(0.0, 100.0))] };
        let entry = |value| FieldValue::Struct(vec![FieldEntry { name: "health".to_string(), value }]);

        assert_eq!(validate_value(&stats, &[], entry(FieldValue::Int(500))), Ok(entry(FieldValue::Int(100))));

        let unknown = FieldValue::Struct(vec![FieldEntry { name: "mana".to_string(), value: FieldValue::Int(1) }]);

        assert!(validate_value(&stats, &[], unknown).is_err());
    }

    #[test]
    fn a_path_leads_through_structs_and_lists_and_carries_read_only_down() {
        let fields = vec![
            descriptor(
                "stats",
                FieldKind::Struct { fields: vec![descriptor("health", FieldKind::Int, range(0.0, 100.0))] },
                vec![FieldAttribute::ReadOnly],
            ),
            descriptor("waypoints", FieldKind::List { element: Box::new(FieldKind::Vec3) }, range(0.0, 5.0)),
        ];

        let health = resolve(&fields, &FieldPath::field("stats").child_field("health")).unwrap();
        assert_eq!(health.kind, &FieldKind::Int);
        assert!(health.read_only); // the struct is read-only, so everything in it is

        let element = resolve(&fields, &FieldPath::field("waypoints").child_index(3)).unwrap();
        assert_eq!(element.kind, &FieldKind::Vec3);
        assert_eq!(range_of(element.attributes), Some((0.0, 5.0))); // the list's range applies to elements
        assert!(!element.read_only);

        assert!(resolve(&fields, &FieldPath::field("missing")).is_err());
        assert!(resolve(&fields, &FieldPath::field("waypoints").child_field("x")).is_err());
        assert!(resolve(&fields, &FieldPath::default()).is_err());
    }
}
