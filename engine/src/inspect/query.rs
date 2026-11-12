//! Looking things up for the Inspector: which fields a component has, how to label them,
//! which are visible, and how they are laid out (headers, spaces, foldout groups).

use super::access;
use super::builtin::{builtin_get, builtin_schema};
use super::interface::{
    FieldAttribute, FieldDescriptor, FieldPath, FieldValue, PathSegment, SchemaLibraryData, ScriptSchemaData,
};
use crate::identity::AssetUuid;
use crate::scene::ComponentData;
use std::borrow::Cow;

pub fn find_schema<'a>(library: &'a SchemaLibraryData, script: &AssetUuid) -> Option<&'a ScriptSchemaData> {
    library.scripts.iter().find(|schema| schema.script == *script)
}

/// The fields of a component: the built-in description, or the script's schema.
/// None for a script that has no schema (yet).
pub fn schema_for<'a>(data: &ComponentData, library: &'a SchemaLibraryData) -> Option<Cow<'a, [FieldDescriptor]>> {
    match data {
        ComponentData::Script(script) => {
            find_schema(library, &script.script).map(|schema| Cow::Borrowed(schema.fields.as_slice()))
        }
        other => builtin_schema(other).map(Cow::Owned),
    }
}

/// The current value of a field of a component, by path (a script's fields can be nested).
pub fn read_field(data: &ComponentData, path: &FieldPath) -> Option<FieldValue> {
    match data {
        ComponentData::Script(script) => access::get_in(&script.fields, path).cloned(),
        other => match path.segments() {
            [PathSegment::Field(name)] => builtin_get(other, name),
            _ => None,
        },
    }
}

/// `jumpHeight` -> "Jump Height", `max_speed` -> "Max Speed".
pub fn humanize(name: &str) -> String {
    let mut text = String::new();
    let mut previous: Option<char> = None;

    for letter in name.chars() {
        if letter == '_' {
            if !text.is_empty() && !text.ends_with(' ') {
                text.push(' ');
            }

            previous = Some(' ');
            continue;
        }

        if letter.is_uppercase() && previous.map_or(false, |before| before.is_lowercase() || before.is_ascii_digit()) {
            text.push(' ');
        }

        if text.is_empty() || text.ends_with(' ') {
            text.extend(letter.to_uppercase());
        } else {
            text.push(letter);
        }

        previous = Some(letter);
    }

    text.trim_end().to_string()
}

/// The name shown for a field: its Label attribute, or the field name made readable.
pub fn label(descriptor: &FieldDescriptor) -> String {
    descriptor
        .attributes
        .iter()
        .find_map(|attribute| match attribute {
            FieldAttribute::Label { text } => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_else(|| humanize(&descriptor.name))
}

pub fn tooltip(descriptor: &FieldDescriptor) -> Option<&str> {
    descriptor.attributes.iter().find_map(|attribute| match attribute {
        FieldAttribute::Tooltip { text } => Some(text.as_str()),
        _ => None,
    })
}

pub fn group_of(descriptor: &FieldDescriptor) -> Option<&str> {
    descriptor.attributes.iter().find_map(|attribute| match attribute {
        FieldAttribute::Group { name } => Some(name.as_str()),
        _ => None,
    })
}

/// False for a hidden field, or when its ShowIf / HideIf condition says so.
/// `lookup` gives the current value of another field of the same object.
pub fn is_visible(descriptor: &FieldDescriptor, lookup: &dyn Fn(&str) -> Option<FieldValue>) -> bool {
    for attribute in &descriptor.attributes {
        match attribute {
            FieldAttribute::Hide => return false,
            FieldAttribute::ShowIf { field, equals } if lookup(field).as_ref() != Some(equals) => return false,
            FieldAttribute::HideIf { field, equals } if lookup(field).as_ref() == Some(equals) => return false,
            _ => {}
        }
    }

    true
}

/// One thing to draw, in order.
#[derive(Debug, PartialEq)]
pub enum LayoutItem<'a> {
    Header(&'a str),
    Space(f32),
    Field(&'a FieldDescriptor),
    /// A foldout holding all the fields that name this group.
    Group { name: &'a str, items: Vec<LayoutItem<'a>> },
}

/// Headers and spaces that come before a field.
fn decorations(descriptor: &FieldDescriptor) -> Vec<LayoutItem<'_>> {
    descriptor
        .attributes
        .iter()
        .filter_map(|attribute| match attribute {
            FieldAttribute::Header { text } => Some(LayoutItem::Header(text.as_str())),
            FieldAttribute::Space { height } => Some(LayoutItem::Space(*height)),
            _ => None,
        })
        .collect()
}

enum Slot<'a> {
    Plain(LayoutItem<'a>),
    Group(usize),
}

/// The order things are drawn in. Hidden fields are left out. Fields of the same group are
/// collected into one foldout, which sits where the group's first field is.
pub fn layout<'a>(descriptors: &'a [FieldDescriptor], lookup: &dyn Fn(&str) -> Option<FieldValue>) -> Vec<LayoutItem<'a>> {
    let mut slots: Vec<Slot<'a>> = Vec::new();
    let mut groups: Vec<(&'a str, Vec<LayoutItem<'a>>)> = Vec::new();

    for descriptor in descriptors {
        if !is_visible(descriptor, lookup) {
            continue;
        }

        let mut chunk = decorations(descriptor);

        chunk.push(LayoutItem::Field(descriptor));

        match group_of(descriptor) {
            Some(name) => {
                let index = match groups.iter().position(|(existing, _)| *existing == name) {
                    Some(index) => index,
                    None => {
                        groups.push((name, Vec::new()));
                        slots.push(Slot::Group(groups.len() - 1));
                        groups.len() - 1
                    }
                };

                groups[index].1.extend(chunk);
            }
            None => slots.extend(chunk.into_iter().map(Slot::Plain)),
        }
    }

    slots
        .into_iter()
        .map(|slot| match slot {
            Slot::Plain(item) => item,
            Slot::Group(index) => {
                let name = groups[index].0;

                LayoutItem::Group { name, items: std::mem::take(&mut groups[index].1) }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::interface::FieldKind;

    fn field(name: &str, attributes: Vec<FieldAttribute>) -> FieldDescriptor {
        FieldDescriptor { name: name.to_string(), kind: FieldKind::Bool, default: None, attributes }
    }

    fn names(items: &[LayoutItem]) -> Vec<String> {
        items
            .iter()
            .map(|item| match item {
                LayoutItem::Header(text) => format!("# {text}"),
                LayoutItem::Space(_) => "_".to_string(),
                LayoutItem::Field(descriptor) => descriptor.name.clone(),
                LayoutItem::Group { name, items } => format!("[{name}: {}]", names(items).join(" ")),
            })
            .collect()
    }

    #[test]
    fn field_names_become_readable_labels() {
        assert_eq!(humanize("jumpHeight"), "Jump Height");
        assert_eq!(humanize("max_speed"), "Max Speed");
        assert_eq!(humanize("speed"), "Speed");
        assert_eq!(humanize("HP"), "HP");
        assert_eq!(humanize("anchor_min"), "Anchor Min");
    }

    #[test]
    fn a_label_attribute_wins_over_the_field_name() {
        let plain = field("jump_height", vec![]);
        let named = field("jump_height", vec![FieldAttribute::Label { text: "Jump".to_string() }]);

        assert_eq!(label(&plain), "Jump Height");
        assert_eq!(label(&named), "Jump");
    }

    #[test]
    fn show_if_hide_if_and_hide_decide_what_is_visible() {
        let lookup = |name: &str| (name == "can_fly").then(|| FieldValue::Bool(true));
        let show = field("a", vec![FieldAttribute::ShowIf { field: "can_fly".to_string(), equals: FieldValue::Bool(true) }]);
        let show_not = field("b", vec![FieldAttribute::ShowIf { field: "can_fly".to_string(), equals: FieldValue::Bool(false) }]);
        let hide_if = field("c", vec![FieldAttribute::HideIf { field: "can_fly".to_string(), equals: FieldValue::Bool(true) }]);
        let hidden = field("d", vec![FieldAttribute::Hide]);
        let unknown = field("e", vec![FieldAttribute::ShowIf { field: "nothing".to_string(), equals: FieldValue::Bool(true) }]);

        assert!(is_visible(&show, &lookup));
        assert!(!is_visible(&show_not, &lookup));
        assert!(!is_visible(&hide_if, &lookup));
        assert!(!is_visible(&hidden, &lookup));
        assert!(!is_visible(&unknown, &lookup));
    }

    #[test]
    fn the_layout_has_headers_spaces_and_foldouts_in_order() {
        let fields = vec![
            field("speed", vec![FieldAttribute::Header { text: "Movement".to_string() }]),
            field("hp", vec![FieldAttribute::Group { name: "Stats".to_string() }]),
            field("jump", vec![FieldAttribute::Space { height: 8.0 }]),
            field("armor", vec![FieldAttribute::Group { name: "Stats".to_string() }]),
            field("secret", vec![FieldAttribute::Hide]),
            field("fly", vec![FieldAttribute::ShowIf { field: "can_fly".to_string(), equals: FieldValue::Bool(true) }]),
        ];

        let flying = |name: &str| (name == "can_fly").then(|| FieldValue::Bool(true));
        let grounded = |_: &str| -> Option<FieldValue> { None };

        assert_eq!(names(&layout(&fields, &flying)), vec!["# Movement", "speed", "[Stats: hp armor]", "_", "jump", "fly"]);
        assert_eq!(names(&layout(&fields, &grounded)), vec!["# Movement", "speed", "[Stats: hp armor]", "_", "jump"]);
    }
}
