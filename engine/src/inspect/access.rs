//! Reading and writing a value anywhere inside nested structs and lists.
//! The top level is a list of named entries (the fields of a script).

use super::interface::{FieldEntry, FieldPath, FieldValue, PathSegment};

fn missing_field(name: &str) -> String {
    format!("there is no field called \"{name}\"")
}

/// The value a path leads to, or None if there is none.
pub fn get_in<'a>(root: &'a [FieldEntry], path: &FieldPath) -> Option<&'a FieldValue> {
    let mut segments = path.segments().iter();
    let PathSegment::Field(name) = segments.next()? else {
        return None;
    };
    let mut current = &root.iter().find(|entry| entry.name == *name)?.value;

    for segment in segments {
        current = match (current, segment) {
            (FieldValue::Struct(entries), PathSegment::Field(name)) => {
                &entries.iter().find(|entry| entry.name == *name)?.value
            }
            (FieldValue::List(items), PathSegment::Index(index)) => items.get(*index)?,
            _ => return None,
        };
    }

    Some(current)
}

fn find_mut<'a>(current: &'a mut FieldValue, segments: &[PathSegment]) -> Option<&'a mut FieldValue> {
    let Some((first, rest)) = segments.split_first() else {
        return Some(current);
    };

    match (current, first) {
        (FieldValue::Struct(entries), PathSegment::Field(name)) => {
            find_mut(&mut entries.iter_mut().find(|entry| entry.name == *name)?.value, rest)
        }
        (FieldValue::List(items), PathSegment::Index(index)) => find_mut(items.get_mut(*index)?, rest),
        _ => None,
    }
}

/// The value a path leads to, to change it in place.
pub fn value_mut<'a>(root: &'a mut [FieldEntry], path: &FieldPath) -> Option<&'a mut FieldValue> {
    let (first, rest) = path.segments().split_first()?;
    let PathSegment::Field(name) = first else {
        return None;
    };

    find_mut(&mut root.iter_mut().find(|entry| entry.name == *name)?.value, rest)
}

/// Replaces the value a path leads to. A top-level field that has no value yet is added.
pub fn set_in(root: &mut Vec<FieldEntry>, path: &FieldPath, value: FieldValue) -> Result<(), String> {
    let Some((PathSegment::Field(name), rest)) = path.segments().split_first() else {
        return Err("a field path must start with a field name".to_string());
    };

    let position = root.iter().position(|entry| entry.name == *name);

    if rest.is_empty() {
        match position {
            Some(index) => root[index].value = value,
            None => root.push(FieldEntry { name: name.clone(), value }),
        }

        return Ok(());
    }

    let index = position.ok_or_else(|| format!("\"{name}\" has no value yet"))?;
    let no_value = || "that path does not lead to a value".to_string();
    let (last, parent_path) = rest.split_last().ok_or_else(no_value)?;
    let parent = find_mut(&mut root[index].value, parent_path).ok_or_else(no_value)?;

    match (parent, last) {
        // A struct may not have an entry for that field yet (the schema gained it): add it.
        (FieldValue::Struct(entries), PathSegment::Field(field)) => match entries.iter_mut().find(|entry| entry.name == *field) {
            Some(entry) => entry.value = value,
            None => entries.push(FieldEntry { name: field.clone(), value }),
        },
        (FieldValue::List(items), PathSegment::Index(position)) => {
            let length = items.len();

            *items.get_mut(*position).ok_or_else(|| format!("index {position} is outside the list of {length}"))? = value;
        }
        _ => return Err(no_value()),
    }

    Ok(())
}

/// Adds an element at the end of the list a path leads to.
pub fn list_push(root: &mut [FieldEntry], path: &FieldPath, element: FieldValue) -> Result<(), String> {
    match value_mut(root, path) {
        Some(FieldValue::List(items)) => {
            items.push(element);
            Ok(())
        }
        Some(_) => Err("that field is not a list".to_string()),
        None => Err(missing_field_in(path)),
    }
}

/// Removes one element of the list a path leads to.
pub fn list_remove(root: &mut [FieldEntry], path: &FieldPath, index: usize) -> Result<(), String> {
    match value_mut(root, path) {
        Some(FieldValue::List(items)) => {
            if index >= items.len() {
                return Err(format!("index {index} is outside the list of {}", items.len()));
            }

            items.remove(index);
            Ok(())
        }
        Some(_) => Err("that field is not a list".to_string()),
        None => Err(missing_field_in(path)),
    }
}

fn missing_field_in(path: &FieldPath) -> String {
    match path.segments().first() {
        Some(PathSegment::Field(name)) => missing_field(name),
        _ => "that path does not lead to a value".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, value: FieldValue) -> FieldEntry {
        FieldEntry { name: name.to_string(), value }
    }

    fn sample() -> Vec<FieldEntry> {
        vec![
            entry("speed", FieldValue::Float(3.0)),
            entry("stats", FieldValue::Struct(vec![entry("health", FieldValue::Int(50))])),
            entry("numbers", FieldValue::List(vec![FieldValue::Int(1), FieldValue::Int(2)])),
        ]
    }

    #[test]
    fn values_are_found_by_path() {
        let root = sample();

        assert_eq!(get_in(&root, &FieldPath::field("speed")), Some(&FieldValue::Float(3.0)));
        assert_eq!(get_in(&root, &FieldPath::field("stats").child_field("health")), Some(&FieldValue::Int(50)));
        assert_eq!(get_in(&root, &FieldPath::field("numbers").child_index(1)), Some(&FieldValue::Int(2)));
        assert_eq!(get_in(&root, &FieldPath::field("numbers").child_index(9)), None);
        assert_eq!(get_in(&root, &FieldPath::field("missing")), None);
    }

    #[test]
    fn values_are_replaced_in_place_at_any_depth() {
        let mut root = sample();

        set_in(&mut root, &FieldPath::field("speed"), FieldValue::Float(8.0)).unwrap();
        set_in(&mut root, &FieldPath::field("stats").child_field("health"), FieldValue::Int(75)).unwrap();
        set_in(&mut root, &FieldPath::field("numbers").child_index(0), FieldValue::Int(10)).unwrap();

        assert_eq!(get_in(&root, &FieldPath::field("speed")), Some(&FieldValue::Float(8.0)));
        assert_eq!(get_in(&root, &FieldPath::field("stats").child_field("health")), Some(&FieldValue::Int(75)));
        assert_eq!(get_in(&root, &FieldPath::field("numbers").child_index(0)), Some(&FieldValue::Int(10)));
    }

    #[test]
    fn a_new_top_level_field_is_added_but_a_deep_path_needs_a_value_to_start_from() {
        let mut root = sample();

        set_in(&mut root, &FieldPath::field("fresh"), FieldValue::Bool(true)).unwrap();
        assert_eq!(root.len(), 4);

        assert!(set_in(&mut root, &FieldPath::field("nothing").child_field("x"), FieldValue::Int(1)).is_err());
        assert!(set_in(&mut root, &FieldPath::field("numbers").child_index(9), FieldValue::Int(1)).is_err());
    }

    #[test]
    fn a_struct_gets_an_entry_for_a_field_it_does_not_have_yet() {
        let mut root = sample();
        let path = FieldPath::field("stats").child_field("armor");

        assert_eq!(get_in(&root, &path), None);

        set_in(&mut root, &path, FieldValue::Float(1.5)).unwrap();

        assert_eq!(get_in(&root, &path), Some(&FieldValue::Float(1.5)));
        assert_eq!(get_in(&root, &FieldPath::field("stats").child_field("health")), Some(&FieldValue::Int(50)));
    }

    #[test]
    fn list_elements_can_be_added_and_removed() {
        let mut root = sample();
        let path = FieldPath::field("numbers");

        list_push(&mut root, &path, FieldValue::Int(3)).unwrap();
        assert_eq!(get_in(&root, &path), Some(&FieldValue::List(vec![FieldValue::Int(1), FieldValue::Int(2), FieldValue::Int(3)])));

        list_remove(&mut root, &path, 0).unwrap();
        assert_eq!(get_in(&root, &path), Some(&FieldValue::List(vec![FieldValue::Int(2), FieldValue::Int(3)])));

        assert!(list_remove(&mut root, &path, 5).is_err());
        assert!(list_push(&mut root, &FieldPath::field("speed"), FieldValue::Int(1)).is_err());
        assert!(list_push(&mut root, &FieldPath::field("missing"), FieldValue::Int(1)).is_err());
    }
}
