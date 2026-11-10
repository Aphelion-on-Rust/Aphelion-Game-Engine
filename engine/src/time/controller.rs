use super::model::TimeModel;
use crate::error::{EngineError, EngineResult};
use crate::mivc::Controller;

pub struct TimeController;

impl Controller for TimeController {}

impl TimeController {
    /// Call once at the start of every frame with the real time since the last frame.
    pub fn begin_frame(&self, model: &mut TimeModel, real_delta_seconds: f64) {
        let data = self.write(model);
        let delta = real_delta_seconds.max(0.0).min(data.max_delta_seconds) * data.time_scale;

        data.delta_seconds = delta;
        data.elapsed_seconds += delta;
        data.accumulator += delta;
        data.frame_count += 1;
    }

    /// Returns true if one fixed step is due, and uses it up.
    /// Call it in a `while` loop until it returns false.
    pub fn take_fixed_step(&self, model: &mut TimeModel) -> bool {
        let data = self.write(model);

        if data.accumulator >= data.fixed_delta_seconds {
            data.accumulator -= data.fixed_delta_seconds;
            true
        } else {
            false
        }
    }

    /// Negative values are treated as 0 (paused).
    pub fn set_time_scale(&self, model: &mut TimeModel, scale: f64) {
        self.write(model).time_scale = scale.max(0.0);
    }

    pub fn set_fixed_step(&self, model: &mut TimeModel, seconds: f64) -> EngineResult<()> {
        if seconds <= 0.0 {
            return Err(EngineError::InvalidOperation("the fixed step must be above 0".to_string()));
        }

        self.write(model).fixed_delta_seconds = seconds;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mivc::Model;
    use crate::time::query::blend_alpha;

    #[test]
    fn frame_time_is_split_into_fixed_steps() {
        let mut model = TimeModel::default();
        TimeController.set_fixed_step(&mut model, 0.01).unwrap();
        TimeController.begin_frame(&mut model, 0.035);

        let mut steps = 0;
        while TimeController.take_fixed_step(&mut model) {
            steps += 1;
        }

        assert_eq!(steps, 3);
        assert!((blend_alpha(model.data()) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn paused_time_does_not_move() {
        let mut model = TimeModel::default();
        TimeController.set_time_scale(&mut model, 0.0);
        TimeController.begin_frame(&mut model, 0.1);

        assert_eq!(model.data().delta_seconds, 0.0);
        assert_eq!(model.data().elapsed_seconds, 0.0);
    }

    #[test]
    fn a_huge_lag_spike_is_clamped() {
        let mut model = TimeModel::default();
        TimeController.begin_frame(&mut model, 10.0);

        assert_eq!(model.data().delta_seconds, 0.25);
    }
}
