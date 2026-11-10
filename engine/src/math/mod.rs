//! Small math types used by components. Kept simple on purpose.
//! Rotation is stored as euler angles in degrees (x, y, z) to keep YAML easy to read.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Axis 0 = x, anything else = y. Lets the inspector edit one axis by number.
    pub fn get(&self, axis: usize) -> f32 {
        if axis == 0 { self.x } else { self.y }
    }

    pub fn set(&mut self, axis: usize, value: f32) {
        if axis == 0 {
            self.x = value;
        } else {
            self.y = value;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Axis 0 = x, 1 = y, anything else = z.
    pub fn get(&self, axis: usize) -> f32 {
        match axis {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    pub fn set(&mut self, axis: usize, value: f32) {
        match axis {
            0 => self.x = value,
            1 => self.y = value,
            _ => self.z = value,
        }
    }
}
