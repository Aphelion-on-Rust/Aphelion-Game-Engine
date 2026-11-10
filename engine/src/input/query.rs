use super::interface::{InputData, KeyCode, MouseButton};
use crate::math::Vec2;

pub fn is_key_down(data: &InputData, key: KeyCode) -> bool {
    data.keys_down.contains(&key)
}

pub fn was_key_pressed(data: &InputData, key: KeyCode) -> bool {
    data.keys_pressed.contains(&key)
}

pub fn was_key_released(data: &InputData, key: KeyCode) -> bool {
    data.keys_released.contains(&key)
}

pub fn is_mouse_down(data: &InputData, button: MouseButton) -> bool {
    data.mouse_down.contains(&button)
}

pub fn was_mouse_pressed(data: &InputData, button: MouseButton) -> bool {
    data.mouse_pressed.contains(&button)
}

pub fn was_mouse_released(data: &InputData, button: MouseButton) -> bool {
    data.mouse_released.contains(&button)
}

pub fn mouse_position(data: &InputData) -> Vec2 {
    data.mouse_position
}
