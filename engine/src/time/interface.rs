use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

/// All times are in seconds.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeData {
    /// How long the last frame took (after clamping and time scale).
    pub delta_seconds: f64,
    /// Total game time that has passed.
    pub elapsed_seconds: f64,
    pub frame_count: u64,
    /// Length of one physics step. 1/60 by default.
    pub fixed_delta_seconds: f64,
    /// Time waiting to be spent on fixed steps.
    pub accumulator: f64,
    /// 1.0 normal, 0.5 slow motion, 0.0 paused.
    pub time_scale: f64,
    /// A frame longer than this is cut short, so one huge lag spike
    /// does not cause hundreds of physics steps.
    pub max_delta_seconds: f64,
}

impl Default for TimeData {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
            frame_count: 0,
            fixed_delta_seconds: 1.0 / 60.0,
            accumulator: 0.0,
            time_scale: 1.0,
            max_delta_seconds: 0.25,
        }
    }
}

impl Interface for TimeData {
    const FORMAT: &'static str = "time";
    const VERSION: u32 = 1;
}
