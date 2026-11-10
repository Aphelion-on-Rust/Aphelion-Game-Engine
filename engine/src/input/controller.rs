use super::interface::{KeyCode, MouseButton};
use super::model::InputModel;
use crate::math::Vec2;
use crate::mivc::Controller;

/// Turns raw events from the window into input state.
pub struct InputController;

impl Controller for InputController {}

impl InputController {
    pub fn key_event(&self, model: &mut InputModel, key: KeyCode, is_pressed: bool) {
        let data = self.write(model);

        if is_pressed {
            // insert returns false when the key was already down (OS key repeat),
            // so "pressed" only fires once per real press.
            if data.keys_down.insert(key) {
                data.keys_pressed.insert(key);
            }
        } else if data.keys_down.remove(&key) {
            data.keys_released.insert(key);
        }
    }

    pub fn mouse_button_event(&self, model: &mut InputModel, button: MouseButton, is_pressed: bool) {
        let data = self.write(model);

        if is_pressed {
            if data.mouse_down.insert(button) {
                data.mouse_pressed.insert(button);
            }
        } else if data.mouse_down.remove(&button) {
            data.mouse_released.insert(button);
        }
    }

    pub fn mouse_moved(&self, model: &mut InputModel, position: Vec2) {
        let data = self.write(model);

        data.mouse_delta = Vec2::new(
            data.mouse_delta.x + position.x - data.mouse_position.x,
            data.mouse_delta.y + position.y - data.mouse_position.y,
        );
        data.mouse_position = position;
    }

    pub fn mouse_scrolled(&self, model: &mut InputModel, amount: f32) {
        self.write(model).scroll_delta += amount;
    }

    /// Call at the END of every frame. Clears the one-frame values, keeps what is held.
    pub fn end_frame(&self, model: &mut InputModel) {
        let data = self.write(model);

        data.keys_pressed.clear();
        data.keys_released.clear();
        data.mouse_pressed.clear();
        data.mouse_released.clear();
        data.mouse_delta = Vec2::ZERO;
        data.scroll_delta = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::query::*;
    use crate::mivc::Model;

    #[test]
    fn key_repeat_only_counts_as_one_press() {
        let mut model = InputModel::default();

        InputController.key_event(&mut model, KeyCode::Space, true);
        InputController.end_frame(&mut model);
        InputController.key_event(&mut model, KeyCode::Space, true); // OS repeat

        assert!(is_key_down(model.data(), KeyCode::Space));
        assert!(!was_key_pressed(model.data(), KeyCode::Space));
    }

    #[test]
    fn end_frame_clears_pressed_but_keeps_held() {
        let mut model = InputModel::default();

        InputController.key_event(&mut model, KeyCode::W, true);
        assert!(was_key_pressed(model.data(), KeyCode::W));

        InputController.end_frame(&mut model);
        assert!(!was_key_pressed(model.data(), KeyCode::W));
        assert!(is_key_down(model.data(), KeyCode::W));

        InputController.key_event(&mut model, KeyCode::W, false);
        assert!(was_key_released(model.data(), KeyCode::W));
        assert!(!is_key_down(model.data(), KeyCode::W));
    }

    #[test]
    fn mouse_movement_adds_up_within_a_frame() {
        let mut model = InputModel::default();

        InputController.mouse_moved(&mut model, Vec2::new(10.0, 0.0));
        InputController.mouse_moved(&mut model, Vec2::new(15.0, 5.0));

        assert_eq!(model.data().mouse_delta, Vec2::new(15.0, 5.0));

        InputController.end_frame(&mut model);
        assert_eq!(model.data().mouse_delta, Vec2::ZERO);
        assert_eq!(mouse_position(model.data()), Vec2::new(15.0, 5.0));
    }
}
