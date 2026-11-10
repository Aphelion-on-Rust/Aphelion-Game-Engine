use super::controller::LifecycleController;
use super::interface::{LifecycleEvent, PressedOn};
use super::model::{LifecycleModel, MouseTrackerModel};
use crate::identity::AssetUuid;
use crate::input::InputData;
use crate::mivc::{Controller, Model};

/// Turns "what is under the cursor" plus the mouse input into OnMouse... events.
///
/// Finding the object under the cursor (picking) needs colliders, which come in the
/// physics step. Until then the caller passes `hovered` in.
pub struct MouseEventController;

impl Controller for MouseEventController {}

impl MouseEventController {
    /// Call once per frame. Queues events, they are delivered by `flush_object_events`.
    pub fn update(
        &self,
        tracker: &mut MouseTrackerModel,
        lifecycle: &mut LifecycleModel,
        input: &InputData,
        hovered: Option<AssetUuid>,
    ) {
        let queue = |lifecycle: &mut LifecycleModel, object: AssetUuid, event: LifecycleEvent| {
            LifecycleController.queue_object_event(lifecycle, object, event);
        };

        // Enter / Exit when the cursor moves from one object to another.
        let previous = tracker.data().hovered;

        if previous != hovered {
            if let Some(old) = previous {
                queue(lifecycle, old, LifecycleEvent::OnMouseExit);
            }

            if let Some(new) = hovered {
                queue(lifecycle, new, LifecycleEvent::OnMouseEnter);
            }
        }

        if let Some(object) = hovered {
            queue(lifecycle, object, LifecycleEvent::OnMouseOver);
        }

        let data = self.write(tracker);
        data.hovered = hovered;

        // Buttons pressed this frame: remember where, and send OnMouseDown.
        for button in &input.mouse_pressed {
            if let Some(object) = hovered {
                data.pressed_on.retain(|entry| entry.button != *button);
                data.pressed_on.push(PressedOn { button: *button, object });
                queue(lifecycle, object, LifecycleEvent::OnMouseDown { button: *button });
            }
        }

        // Buttons still held (not the ones pressed this very frame): OnMouseDrag.
        for entry in &data.pressed_on {
            let is_held = input.mouse_down.contains(&entry.button);
            let just_pressed = input.mouse_pressed.contains(&entry.button);

            if is_held && !just_pressed {
                queue(lifecycle, entry.object, LifecycleEvent::OnMouseDrag { button: entry.button });
            }
        }

        // Buttons released this frame: OnMouseUp on the object where it was pressed,
        // and OnMouseClick if the cursor is still over that same object.
        for button in &input.mouse_released {
            if let Some(position) = data.pressed_on.iter().position(|entry| entry.button == *button) {
                let entry = data.pressed_on.remove(position);

                queue(lifecycle, entry.object, LifecycleEvent::OnMouseUp { button: *button });

                if hovered == Some(entry.object) {
                    queue(lifecycle, entry.object, LifecycleEvent::OnMouseClick { button: *button });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::input::{InputController, InputModel, MouseButton};

    fn queued_names(lifecycle: &LifecycleModel) -> Vec<&'static str> {
        lifecycle.data().pending_events.iter().map(|item| item.event.name()).collect()
    }

    fn clear(lifecycle: &mut LifecycleModel) {
        LifecycleController.flush_object_events(lifecycle);
    }

    #[test]
    fn a_full_click_on_an_object() {
        let target = IdController.new_uuid(&mut IdRegistryModel::default());
        let mut tracker = MouseTrackerModel::default();
        let mut lifecycle = LifecycleModel::default();
        let mut input = InputModel::default();

        // Cursor arrives.
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), Some(target));
        assert_eq!(queued_names(&lifecycle), vec!["OnMouseEnter", "OnMouseOver"]);
        clear(&mut lifecycle);

        // Button goes down.
        InputController.mouse_button_event(&mut input, MouseButton::Left, true);
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), Some(target));
        assert_eq!(queued_names(&lifecycle), vec!["OnMouseOver", "OnMouseDown"]);
        clear(&mut lifecycle);
        InputController.end_frame(&mut input);

        // Button held.
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), Some(target));
        assert_eq!(queued_names(&lifecycle), vec!["OnMouseOver", "OnMouseDrag"]);
        clear(&mut lifecycle);

        // Button released over the same object.
        InputController.mouse_button_event(&mut input, MouseButton::Left, false);
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), Some(target));
        assert_eq!(queued_names(&lifecycle), vec!["OnMouseOver", "OnMouseUp", "OnMouseClick"]);
        clear(&mut lifecycle);
        InputController.end_frame(&mut input);

        // Cursor leaves.
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), None);
        assert_eq!(queued_names(&lifecycle), vec!["OnMouseExit"]);
    }

    #[test]
    fn releasing_somewhere_else_is_not_a_click() {
        let target = IdController.new_uuid(&mut IdRegistryModel::default());
        let mut tracker = MouseTrackerModel::default();
        let mut lifecycle = LifecycleModel::default();
        let mut input = InputModel::default();

        InputController.mouse_button_event(&mut input, MouseButton::Left, true);
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), Some(target));
        clear(&mut lifecycle);
        InputController.end_frame(&mut input);

        InputController.mouse_button_event(&mut input, MouseButton::Left, false);
        MouseEventController.update(&mut tracker, &mut lifecycle, input.data(), None);

        let names = queued_names(&lifecycle);
        assert!(names.contains(&"OnMouseUp"));
        assert!(!names.contains(&"OnMouseClick"));
    }
}
