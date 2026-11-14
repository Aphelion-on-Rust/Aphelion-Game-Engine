use crate::math::{Rgba, Vec2};
use serde::{Deserialize, Serialize};

/// Draws a flat picture (a sprite) facing the camera's Z axis. Sprite images come with the asset step,
/// until then it is a colored rectangle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteDrawData {
    pub color: Rgba,
    pub flip_x: bool,
    pub flip_y: bool,
    /// Sprites with a higher order are drawn on top of lower ones.
    pub order: i32,
    /// Width and height in world units.
    pub size: Vec2,
    pub visible: bool,
}

impl Default for SpriteDrawData {
    fn default() -> Self {
        Self { color: Rgba::WHITE, flip_x: false, flip_y: false, order: 0, size: Vec2::new(1.0, 1.0), visible: true }
    }
}
