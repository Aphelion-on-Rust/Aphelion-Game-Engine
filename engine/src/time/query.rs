use super::interface::TimeData;

/// How far we are between two fixed steps, from 0.0 to 1.0.
/// Renderers use it to draw smoothly between physics steps.
pub fn blend_alpha(data: &TimeData) -> f64 {
    if data.fixed_delta_seconds <= 0.0 {
        return 0.0;
    }

    (data.accumulator / data.fixed_delta_seconds).clamp(0.0, 1.0)
}
