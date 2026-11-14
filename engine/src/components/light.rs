use crate::math::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightKind {
    /// The sun: light from one direction everywhere. Position does not matter.
    Directional,
    /// Shines in all directions from a point.
    Point,
    /// A cone of light.
    Spot,
}

named_enum!(LightKind { Directional, Point, Spot });

/// A light. It shines along the +Z direction of its object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightData {
    pub kind: LightKind,
    pub color: Rgba,
    pub intensity: f32,
    /// How far a point or spot light reaches.
    pub range: f32,
    /// The width of a spot light cone, in degrees.
    pub spot_angle_degrees: f32,
    pub shadows: bool,
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            kind: LightKind::Point,
            color: Rgba::WHITE,
            intensity: 1.0,
            range: 10.0,
            spot_angle_degrees: 45.0,
            shadows: false,
        }
    }
}
