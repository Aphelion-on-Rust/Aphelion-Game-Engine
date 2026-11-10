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
