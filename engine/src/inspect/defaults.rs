use super::interface::{FieldDescriptor, FieldEntry, FieldKind, FieldValue};
use crate::math::{Rgba, Vec2, Vec3};

/// The value a field of this kind starts with.
pub fn default_value(kind: &FieldKind) -> FieldValue {
    match kind {
        FieldKind::Bool => FieldValue::Bool(false),
        FieldKind::Int => FieldValue::Int(0),
        FieldKind::Float => FieldValue::Float(0.0),
        FieldKind::Text => FieldValue::Text(String::new()),
        FieldKind::Vec2 => FieldValue::Vec2(Vec2::ZERO),
        FieldKind::Vec3 => FieldValue::Vec3(Vec3::ZERO),
        FieldKind::Color => FieldValue::Color(Rgba::WHITE),
        FieldKind::Enum { options } => FieldValue::Enum(options.first().cloned().unwrap_or_default()),
        FieldKind::ObjectRef => FieldValue::ObjectRef(None),
        FieldKind::List { .. } => FieldValue::List(Vec::new()),
        FieldKind::Struct { fields } => FieldValue::Struct(
            fields
                .iter()
                .map(|descriptor| FieldEntry { name: descriptor.name.clone(), value: descriptor_default(descriptor) })
                .collect(),
        ),
    }
}

/// The default of a field: the one written in its description, or the kind's own.
pub fn descriptor_default(descriptor: &FieldDescriptor) -> FieldValue {
    descriptor.default.clone().unwrap_or_else(|| default_value(&descriptor.kind))
}

/// Adds a default value for every field of the schema that has none yet. Existing values stay,
/// and values of fields that are no longer in the schema are left alone (nothing is thrown away).
/// Returns how many fields were added.
pub fn sync_fields(entries: &mut Vec<FieldEntry>, schema: &[FieldDescriptor]) -> usize {
    let mut added = 0;

    for descriptor in schema {
        if !entries.iter().any(|entry| entry.name == descriptor.name) {
            entries.push(FieldEntry { name: descriptor.name.clone(), value: descriptor_default(descriptor) });
            added += 1;
        }
    }

    added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(name: &str, kind: FieldKind, default: Option<FieldValue>) -> FieldDescriptor {
        FieldDescriptor { name: name.to_string(), kind, default, attributes: Vec::new() }
    }

    #[test]
    fn every_kind_has_a_sensible_default() {
        assert_eq!(default_value(&FieldKind::Float), FieldValue::Float(0.0));
        assert_eq!(default_value(&FieldKind::Color), FieldValue::Color(Rgba::WHITE));
        assert_eq!(
            default_value(&FieldKind::Enum { options: vec!["Walk".to_string(), "Run".to_string()] }),
            FieldValue::Enum("Walk".to_string())
        );
        assert_eq!(default_value(&FieldKind::List { element: Box::new(FieldKind::Int) }), FieldValue::List(Vec::new()));
    }

    #[test]
    fn a_struct_starts_with_the_defaults_of_its_fields_and_a_written_default_wins() {
        let kind = FieldKind::Struct {
            fields: vec![
                descriptor("health", FieldKind::Int, Some(FieldValue::Int(100))),
                descriptor("armor", FieldKind::Float, None),
            ],
        };

        assert_eq!(
            default_value(&kind),
            FieldValue::Struct(vec![
                FieldEntry { name: "health".to_string(), value: FieldValue::Int(100) },
                FieldEntry { name: "armor".to_string(), value: FieldValue::Float(0.0) },
            ])
        );
    }

    #[test]
    fn syncing_adds_missing_fields_and_keeps_everything_else() {
        let schema = vec![
            descriptor("speed", FieldKind::Float, Some(FieldValue::Float(5.0))),
            descriptor("can_fly", FieldKind::Bool, None),
        ];
        let mut entries = vec![
            FieldEntry { name: "speed".to_string(), value: FieldValue::Float(9.0) },
            FieldEntry { name: "old_field".to_string(), value: FieldValue::Int(1) },
        ];

        assert_eq!(sync_fields(&mut entries, &schema), 1);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].value, FieldValue::Float(9.0)); // kept
        assert_eq!(entries[1].name, "old_field"); // not thrown away
        assert_eq!(entries[2].name, "can_fly");
        assert_eq!(sync_fields(&mut entries, &schema), 0);
    }
}
