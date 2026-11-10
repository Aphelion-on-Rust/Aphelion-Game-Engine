//! Which number inside which component a number field edits.

use flux_engine::identity::{AssetUuid, ComponentId};
use flux_engine::scene::ComponentData;

/// One number inside a component. The `usize` is the axis: 0 = x, 1 = y, 2 = z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentField {
    Position(usize),
    Rotation(usize),
    Scale(usize),
    AnchorMin(usize),
    AnchorMax(usize),
    Pivot(usize),
    RectPosition(usize),
    Size(usize),
}

/// A number field in the inspector: this object, this component, this number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberTarget {
    pub object: AssetUuid,
    pub component: ComponentId,
    pub field: ComponentField,
}

/// Reads the number. None if the field does not belong to this kind of component.
pub fn read_field(data: &ComponentData, field: ComponentField) -> Option<f64> {
    let value = match (data, field) {
        (ComponentData::Transform(t), ComponentField::Position(axis)) => t.position.get(axis),
        (ComponentData::Transform(t), ComponentField::Rotation(axis)) => t.rotation.get(axis),
        (ComponentData::Transform(t), ComponentField::Scale(axis)) => t.scale.get(axis),
        (ComponentData::RectTransform(r), ComponentField::AnchorMin(axis)) => r.anchor_min.get(axis),
        (ComponentData::RectTransform(r), ComponentField::AnchorMax(axis)) => r.anchor_max.get(axis),
        (ComponentData::RectTransform(r), ComponentField::Pivot(axis)) => r.pivot.get(axis),
        (ComponentData::RectTransform(r), ComponentField::RectPosition(axis)) => r.position.get(axis),
        (ComponentData::RectTransform(r), ComponentField::Size(axis)) => r.size.get(axis),
        _ => return None,
    };

    Some(value as f64)
}

/// Writes the number. Does nothing if the field does not belong to this kind of component.
pub fn write_field(data: &mut ComponentData, field: ComponentField, value: f64) {
    let value = value as f32;

    match (data, field) {
        (ComponentData::Transform(t), ComponentField::Position(axis)) => t.position.set(axis, value),
        (ComponentData::Transform(t), ComponentField::Rotation(axis)) => t.rotation.set(axis, value),
        (ComponentData::Transform(t), ComponentField::Scale(axis)) => t.scale.set(axis, value),
        (ComponentData::RectTransform(r), ComponentField::AnchorMin(axis)) => r.anchor_min.set(axis, value),
        (ComponentData::RectTransform(r), ComponentField::AnchorMax(axis)) => r.anchor_max.set(axis, value),
        (ComponentData::RectTransform(r), ComponentField::Pivot(axis)) => r.pivot.set(axis, value),
        (ComponentData::RectTransform(r), ComponentField::RectPosition(axis)) => r.position.set(axis, value),
        (ComponentData::RectTransform(r), ComponentField::Size(axis)) => r.size.set(axis, value),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_engine::scene::{RectTransformData, TransformData};

    #[test]
    fn a_transform_field_can_be_written_and_read_back() {
        let mut data = ComponentData::Transform(TransformData::default());

        write_field(&mut data, ComponentField::Rotation(1), 45.0);

        assert_eq!(read_field(&data, ComponentField::Rotation(1)), Some(45.0));
        assert_eq!(read_field(&data, ComponentField::Rotation(0)), Some(0.0));
    }

    #[test]
    fn a_field_of_the_wrong_component_is_ignored() {
        let mut data = ComponentData::RectTransform(RectTransformData::default());

        write_field(&mut data, ComponentField::Position(0), 9.0);

        assert_eq!(read_field(&data, ComponentField::Position(0)), None);
        assert_eq!(read_field(&data, ComponentField::Size(0)), Some(100.0));
    }
}
