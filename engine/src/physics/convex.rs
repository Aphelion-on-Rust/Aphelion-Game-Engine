//! A collider as a convex solid that can answer one question: "which point of you is farthest in this
//! direction?" (the support point). That is all the collision test (`gjk.rs`) needs, for every shape.

use super::world::WorldCollider;
use crate::components::ColliderShape;
use glam::{Quat, Vec3};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Convex {
    shape: ColliderShape,
    position: Vec3,
    rotation: Quat,
    size: Vec3,
    radius: f32,
    /// Half the length of the straight middle part (half the height, except for a capsule: without the round ends).
    half_straight: f32,
    sides: u32,
}

fn sign(value: f32) -> f32 {
    if value >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

impl Convex {
    /// A collider placed at `position` (its middle in the world). The collider's own position is ignored
    /// so a moving body can place its colliders without changing them.
    pub fn new(collider: &WorldCollider, position: Vec3) -> Self {
        Self {
            shape: collider.shape,
            position,
            rotation: collider.rotation,
            size: collider.size,
            radius: collider.radius,
            half_straight: collider.half_straight(),
            sides: collider.sides.max(3),
        }
    }

    pub fn center(&self) -> Vec3 {
        self.position
    }

    /// A ball around the middle that holds the whole shape. Used to skip pairs that are far apart.
    pub fn bounding_radius(&self) -> f32 {
        match self.shape {
            ColliderShape::Box => self.size.length() * 0.5,
            ColliderShape::Sphere => self.radius,
            ColliderShape::Cylinder | ColliderShape::Prism => (self.radius * self.radius + self.half_straight * self.half_straight).sqrt(),
            ColliderShape::Capsule => self.half_straight + self.radius,
        }
    }

    /// The point of the shape that is farthest along `direction` (in the world).
    pub fn support(&self, direction: Vec3) -> Vec3 {
        let d = self.rotation.inverse() * direction;
        let along_y = Vec3::new(0.0, sign(d.y) * self.half_straight, 0.0);

        let local = match self.shape {
            ColliderShape::Box => {
                let half = self.size * 0.5;

                Vec3::new(sign(d.x) * half.x, sign(d.y) * half.y, sign(d.z) * half.z)
            }
            ColliderShape::Sphere => d.normalize_or_zero() * self.radius,
            ColliderShape::Cylinder => {
                let flat = Vec3::new(d.x, 0.0, d.z).normalize_or_zero();

                along_y + flat * self.radius
            }
            ColliderShape::Capsule => along_y + d.normalize_or_zero() * self.radius,
            ColliderShape::Prism => {
                // The corner of the base polygon that points most in this direction.
                let mut best = Vec3::ZERO;
                let mut best_dot = f32::NEG_INFINITY;

                for corner in 0..self.sides {
                    let angle = 2.0 * PI * corner as f32 / self.sides as f32;
                    let point = Vec3::new(self.radius * angle.cos(), 0.0, self.radius * angle.sin());
                    let dot = point.x * d.x + point.z * d.z;

                    if dot > best_dot {
                        best_dot = dot;
                        best = point;
                    }
                }

                along_y + best
            }
        };

        self.position + self.rotation * local
    }
}
