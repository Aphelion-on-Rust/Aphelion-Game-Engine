use crate::math::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    /// Things get smaller with distance.
    Perspective,
    /// No perspective, for 2D and flat views.
    Orthographic,
}

named_enum!(Projection { Perspective, Orthographic });

/// A camera: what the Game view shows. The camera with the highest priority is used.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewCameraData {
    pub projection: Projection,
    /// How wide the view is, up and down, in degrees. Perspective only.
    pub fov_degrees: f32,
    /// Half the height of the view in world units. Orthographic only.
    pub ortho_size: f32,
    pub near: f32,
    pub far: f32,
    pub clear_color: Rgba,
    pub priority: i32,
}

impl Default for ViewCameraData {
    fn default() -> Self {
        Self {
            projection: Projection::Perspective,
            fov_degrees: 60.0,
            ortho_size: 5.0,
            near: 0.1,
            far: 1000.0,
            clear_color: Rgba::new(0.10, 0.12, 0.16, 1.0),
            priority: 0,
        }
    }
}
