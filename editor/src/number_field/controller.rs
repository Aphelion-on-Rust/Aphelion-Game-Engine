use super::interface::{DragPhase, DragSpeed, NumberFieldConfig};
use super::model::NumberFieldModel;
use flux_engine::error::{EngineError, EngineResult};
use flux_engine::mivc::{Controller, Model};

pub struct NumberFieldController;

impl Controller for NumberFieldController {}

fn not_dragging() -> EngineError {
    EngineError::InvalidOperation("the field is not being dragged".to_string())
}

impl NumberFieldController {
    pub fn hover_enter(&self, model: &mut NumberFieldModel) {
        let data = self.write(model);

        if data.phase == DragPhase::Idle {
            data.phase = DragPhase::Hovering;
        }
    }

    /// Ignored while dragging: the pointer is hidden and may leave the field.
    pub fn hover_exit(&self, model: &mut NumberFieldModel) {
        let data = self.write(model);

        if data.phase == DragPhase::Hovering {
            data.phase = DragPhase::Idle;
        }
    }

    /// Mouse went down on the field. The pointer is hidden from now on.
    pub fn begin_drag(
        &self,
        model: &mut NumberFieldModel,
        mouse_x: f64,
        current_value: f64,
        config: NumberFieldConfig,
    ) -> EngineResult<()> {
        if model.data().phase == DragPhase::Dragging {
            return Err(EngineError::InvalidOperation("the field is already being dragged".to_string()));
        }

        let data = self.write(model);

        data.phase = DragPhase::Dragging;
        data.config = config;
        data.start_value = current_value;
        data.start_mouse_x = mouse_x;
        data.value = current_value;
        Ok(())
    }

    /// Mouse moved. Returns the new value.
    /// `mouse_x` is the total x position, not the movement since last frame.
    pub fn drag_to(&self, model: &mut NumberFieldModel, mouse_x: f64, speed: DragSpeed) -> EngineResult<f64> {
        if model.data().phase != DragPhase::Dragging {
            return Err(not_dragging());
        }

        let data = self.write(model);
        let config = &data.config;
        let moved = mouse_x - data.start_mouse_x;
        let mut value = data.start_value + moved * config.sensitivity * speed.multiplier();

        if let Some(step) = config.step.filter(|step| *step > 0.0) {
            value = (value / step).round() * step;
        }

        if config.is_integer {
            value = value.round();
        }

        if let Some(min) = config.min {
            value = value.max(min);
        }

        if let Some(max) = config.max {
            value = value.min(max);
        }

        data.value = value;
        Ok(value)
    }

    /// Mouse released. The pointer comes back. Returns the final value.
    pub fn end_drag(&self, model: &mut NumberFieldModel) -> EngineResult<f64> {
        if model.data().phase != DragPhase::Dragging {
            return Err(not_dragging());
        }

        let data = self.write(model);

        data.phase = DragPhase::Hovering;
        Ok(data.value)
    }

    /// Escape pressed. Goes back to the value from before the drag.
    pub fn cancel_drag(&self, model: &mut NumberFieldModel) -> EngineResult<f64> {
        if model.data().phase != DragPhase::Dragging {
            return Err(not_dragging());
        }

        let data = self.write(model);

        data.value = data.start_value;
        data.phase = DragPhase::Hovering;
        Ok(data.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number_field::interface::CursorIcon;
    use crate::number_field::query::{cursor_icon, is_pointer_hidden};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn dragging(config: NumberFieldConfig, start_value: f64) -> NumberFieldModel {
        let mut model = NumberFieldModel::default();

        NumberFieldController.hover_enter(&mut model);
        NumberFieldController.begin_drag(&mut model, 100.0, start_value, config).unwrap();
        model
    }

    #[test]
    fn cursor_follows_the_phase() {
        let mut model = NumberFieldModel::default();
        assert_eq!(cursor_icon(model.data()), CursorIcon::Default);

        NumberFieldController.hover_enter(&mut model);
        assert_eq!(cursor_icon(model.data()), CursorIcon::HorizontalResize);

        NumberFieldController.begin_drag(&mut model, 0.0, 1.0, NumberFieldConfig::default()).unwrap();
        assert_eq!(cursor_icon(model.data()), CursorIcon::Hidden);
        assert!(is_pointer_hidden(model.data()));

        NumberFieldController.end_drag(&mut model).unwrap();
        assert!(!is_pointer_hidden(model.data()));
    }

    #[test]
    fn dragging_right_adds_and_left_subtracts() {
        let mut model = dragging(NumberFieldConfig::default(), 5.0);

        // 100 pixels at 0.1 per pixel = +10.
        assert!(close(NumberFieldController.drag_to(&mut model, 200.0, DragSpeed::Normal).unwrap(), 15.0));
        assert!(close(NumberFieldController.drag_to(&mut model, 0.0, DragSpeed::Normal).unwrap(), -5.0));
    }

    #[test]
    fn shift_is_fine_and_ctrl_is_coarse() {
        let mut model = dragging(NumberFieldConfig::default(), 0.0);

        assert!(close(NumberFieldController.drag_to(&mut model, 200.0, DragSpeed::Fine).unwrap(), 1.0));
        assert!(close(NumberFieldController.drag_to(&mut model, 200.0, DragSpeed::Coarse).unwrap(), 100.0));
        assert_eq!(DragSpeed::from_keys(true, false), DragSpeed::Fine);
        assert_eq!(DragSpeed::from_keys(true, true), DragSpeed::Coarse);
    }

    #[test]
    fn range_limits_are_respected() {
        let config = NumberFieldConfig { min: Some(0.0), max: Some(10.0), ..NumberFieldConfig::default() };
        let mut model = dragging(config, 5.0);

        assert!(close(NumberFieldController.drag_to(&mut model, 10_000.0, DragSpeed::Normal).unwrap(), 10.0));
        assert!(close(NumberFieldController.drag_to(&mut model, -10_000.0, DragSpeed::Normal).unwrap(), 0.0));
    }

    #[test]
    fn integers_and_steps_snap() {
        let integer = NumberFieldConfig { is_integer: true, ..NumberFieldConfig::default() };
        let mut model = dragging(integer, 0.0);
        assert!(close(NumberFieldController.drag_to(&mut model, 117.0, DragSpeed::Normal).unwrap(), 2.0));

        let stepped = NumberFieldConfig { step: Some(0.5), ..NumberFieldConfig::default() };
        let mut model = dragging(stepped, 0.0);
        assert!(close(NumberFieldController.drag_to(&mut model, 117.0, DragSpeed::Normal).unwrap(), 1.5));
    }

    #[test]
    fn escape_restores_the_old_value() {
        let mut model = dragging(NumberFieldConfig::default(), 3.0);

        NumberFieldController.drag_to(&mut model, 500.0, DragSpeed::Normal).unwrap();

        assert!(close(NumberFieldController.cancel_drag(&mut model).unwrap(), 3.0));
    }

    #[test]
    fn leaving_the_field_does_not_stop_a_drag() {
        let mut model = dragging(NumberFieldConfig::default(), 0.0);

        NumberFieldController.hover_exit(&mut model);

        assert!(is_pointer_hidden(model.data()));
        assert!(NumberFieldController.drag_to(&mut NumberFieldModel::default(), 1.0, DragSpeed::Normal).is_err());
    }
}
