//! Which number inside which field a number box edits.

use flux_engine::identity::{AssetUuid, ComponentId};
use flux_engine::inspect::{FieldPath, FieldValue};
use flux_engine::math::{Vec2, Vec3};

/// Which number of a field. A float or int has one (`Whole`), a Vec2 has X and Y, a Vec3 X, Y and Z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Whole,
    X,
    Y,
    Z,
}

/// A number box in the Inspector: this object, this component, this field, this part of it.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberTarget {
    pub object: AssetUuid,
    pub component: ComponentId,
    pub path: FieldPath,
    pub part: Part,
}

/// The number a part of a value currently has. None if the value has no such part.
pub fn number_of(value: &FieldValue, part: Part) -> Option<f64> {
    match (value, part) {
        (FieldValue::Float(number), Part::Whole) => Some(*number),
        (FieldValue::Int(number), Part::Whole) => Some(*number as f64),
        (FieldValue::Vec2(v), Part::X) => Some(v.x as f64),
        (FieldValue::Vec2(v), Part::Y) => Some(v.y as f64),
        (FieldValue::Vec3(v), Part::X) => Some(v.x as f64),
        (FieldValue::Vec3(v), Part::Y) => Some(v.y as f64),
        (FieldValue::Vec3(v), Part::Z) => Some(v.z as f64),
        _ => None,
    }
}

/// The value of the whole field after one of its numbers changed. None if the value has no such part.
pub fn with_number(current: &FieldValue, part: Part, number: f64) -> Option<FieldValue> {
    let n = number as f32;

    match (current, part) {
        (FieldValue::Float(_), Part::Whole) => Some(FieldValue::Float(number)),
        (FieldValue::Int(_), Part::Whole) => Some(FieldValue::Int(number.round() as i64)),
        (FieldValue::Vec2(v), Part::X) => Some(FieldValue::Vec2(Vec2::new(n, v.y))),
        (FieldValue::Vec2(v), Part::Y) => Some(FieldValue::Vec2(Vec2::new(v.x, n))),
        (FieldValue::Vec3(v), Part::X) => Some(FieldValue::Vec3(Vec3::new(n, v.y, v.z))),
        (FieldValue::Vec3(v), Part::Y) => Some(FieldValue::Vec3(Vec3::new(v.x, n, v.z))),
        (FieldValue::Vec3(v), Part::Z) => Some(FieldValue::Vec3(Vec3::new(v.x, v.y, n))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_number_of_a_vector_changes_and_the_others_stay() {
        let vector = FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(with_number(&vector, Part::Y, 9.0), Some(FieldValue::Vec3(Vec3::new(1.0, 9.0, 3.0))));
        assert_eq!(with_number(&FieldValue::Vec2(Vec2::new(1.0, 2.0)), Part::X, 5.0), Some(FieldValue::Vec2(Vec2::new(5.0, 2.0))));
    }

    #[test]
    fn floats_and_ints_have_one_number_and_ints_are_rounded() {
        assert_eq!(with_number(&FieldValue::Float(1.0), Part::Whole, 2.5), Some(FieldValue::Float(2.5)));
        assert_eq!(with_number(&FieldValue::Int(1), Part::Whole, 2.6), Some(FieldValue::Int(3)));
    }

    #[test]
    fn a_part_that_does_not_exist_is_refused() {
        assert_eq!(with_number(&FieldValue::Float(1.0), Part::X, 2.0), None);
        assert_eq!(with_number(&FieldValue::Vec2(Vec2::ZERO), Part::Z, 2.0), None);
        assert_eq!(with_number(&FieldValue::Bool(true), Part::Whole, 2.0), None);
    }

    #[test]
    fn the_current_number_of_a_part_can_be_read() {
        assert_eq!(number_of(&FieldValue::Vec3(Vec3::new(1.0, 2.0, 3.0)), Part::Z), Some(3.0));
        assert_eq!(number_of(&FieldValue::Int(4), Part::Whole), Some(4.0));
        assert_eq!(number_of(&FieldValue::Text(String::new()), Part::Whole), None);
    }
}
