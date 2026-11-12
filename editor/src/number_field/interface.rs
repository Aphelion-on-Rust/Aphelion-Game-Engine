use flux_engine::inspect::rules::{min_of, range_of};
use flux_engine::inspect::{FieldAttribute, FieldKind};
use flux_engine::mivc::Interface;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragPhase {
    Idle,
    Hovering,
    Dragging,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragSpeed {
    Normal,
    /// Shift held.
    Fine,
    /// Ctrl held.
    Coarse,
}

impl DragSpeed {
    pub fn multiplier(self) -> f64 {
        match self {
            DragSpeed::Normal => 1.0,
            DragSpeed::Fine => 0.1,
            DragSpeed::Coarse => 10.0,
        }
    }

    /// Ctrl wins if both are held.
    pub fn from_keys(shift: bool, ctrl: bool) -> Self {
        if ctrl {
            DragSpeed::Coarse
        } else if shift {
            DragSpeed::Fine
        } else {
            DragSpeed::Normal
        }
    }
}

/// How one field behaves. Comes from the property, for example a [Range(0, 10)] field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumberFieldConfig {
    /// How much the value changes per pixel of mouse movement.
    pub sensitivity: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Snap the value to multiples of this.
    pub step: Option<f64>,
    pub is_integer: bool,
}

impl NumberFieldConfig {
    /// How the number box of a field behaves: [Range] and [Min] give the limits, an int field
    /// counts whole numbers, and with a range the drag speed follows its size so the whole
    /// range takes about 200 pixels.
    pub fn for_field(kind: &FieldKind, attributes: &[FieldAttribute]) -> Self {
        let range = range_of(attributes).map(|(a, b)| (a.min(b), a.max(b)));
        let is_integer = matches!(kind, FieldKind::Int);
        let min = match (range.map(|(low, _)| low), min_of(attributes)) {
            (Some(low), Some(min)) => Some(low.max(min)),
            (low, min) => low.or(min),
        };
        let sensitivity = match range {
            Some((low, high)) => ((high - low) / 200.0).max(if is_integer { 0.1 } else { 0.0001 }),
            None if is_integer => 0.1,
            None => 0.05,
        };

        Self { sensitivity, min, max: range.map(|(_, high)| high), step: None, is_integer }
    }
}

impl Default for NumberFieldConfig {
    fn default() -> Self {
        Self { sensitivity: 0.1, min: None, max: None, step: None, is_integer: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumberFieldData {
    pub phase: DragPhase,
    pub config: NumberFieldConfig,
    /// The value when the drag started (Escape goes back to it).
    pub start_value: f64,
    pub start_mouse_x: f64,
    pub value: f64,
}

impl Default for NumberFieldData {
    fn default() -> Self {
        Self {
            phase: DragPhase::Idle,
            config: NumberFieldConfig::default(),
            start_value: 0.0,
            start_mouse_x: 0.0,
            value: 0.0,
        }
    }
}

/// What the window should show for the mouse pointer over this field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorIcon {
    Default,
    /// The <-> arrow.
    HorizontalResize,
    /// No pointer at all (while dragging).
    Hidden,
}

impl Interface for NumberFieldData {
    const FORMAT: &'static str = "number_field";
    const VERSION: u32 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_gives_limits_and_a_matching_drag_speed() {
        let config = NumberFieldConfig::for_field(&FieldKind::Float, &[FieldAttribute::Range { min: 0.0, max: 10.0 }]);

        assert_eq!((config.min, config.max), (Some(0.0), Some(10.0)));
        assert!((config.sensitivity - 0.05).abs() < 1e-9);
        assert!(!config.is_integer);
    }

    #[test]
    fn min_alone_sets_only_a_lower_limit_and_a_bigger_min_wins_over_the_range() {
        let only_min = NumberFieldConfig::for_field(&FieldKind::Float, &[FieldAttribute::Min { value: 0.0 }]);
        let both = NumberFieldConfig::for_field(
            &FieldKind::Float,
            &[FieldAttribute::Range { min: 0.0, max: 10.0 }, FieldAttribute::Min { value: 2.0 }],
        );

        assert_eq!((only_min.min, only_min.max), (Some(0.0), None));
        assert_eq!(both.min, Some(2.0));
    }

    #[test]
    fn ints_count_whole_numbers_and_never_drag_slower_than_a_tenth_per_pixel() {
        let config = NumberFieldConfig::for_field(&FieldKind::Int, &[FieldAttribute::Range { min: 0.0, max: 10.0 }]);

        assert!(config.is_integer);
        assert!((config.sensitivity - 0.1).abs() < 1e-9);
        assert_eq!(NumberFieldConfig::for_field(&FieldKind::Float, &[]).sensitivity, 0.05);
    }
}
