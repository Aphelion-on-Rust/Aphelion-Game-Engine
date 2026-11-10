use crate::math::Vec2;
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Up, Down, Left, Right,
    Space, Enter, Escape, Tab, Backspace, Delete,
    Shift, Ctrl, Alt,
}

/// "down" lasts as long as the button is held.
/// "pressed" and "released" are true for one frame only.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputData {
    pub keys_down: BTreeSet<KeyCode>,
    pub keys_pressed: BTreeSet<KeyCode>,
    pub keys_released: BTreeSet<KeyCode>,
    pub mouse_down: BTreeSet<MouseButton>,
    pub mouse_pressed: BTreeSet<MouseButton>,
    pub mouse_released: BTreeSet<MouseButton>,
    pub mouse_position: Vec2,
    /// How far the mouse moved this frame.
    pub mouse_delta: Vec2,
    pub scroll_delta: f32,
}

impl Interface for InputData {
    const FORMAT: &'static str = "input_state";
    const VERSION: u32 = 1;
}
