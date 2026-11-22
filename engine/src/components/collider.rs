use crate::math::Vec3;
use serde::{Deserialize, Serialize};

/// The shape of a collider. Sizes are in the fields of `Collider3DData`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// A box with sides `size`.
    Box,
    /// A ball with `radius`.
    Sphere,
    /// A cylinder with round ends: `radius` and `height` (the height counts the round ends).
    Capsule,
    /// A cylinder with flat ends: `radius` and `height`.
    Cylinder,
    /// A prism with a regular polygon as its base: `sides`, `radius` of the base and `height`.
    Prism,
}

named_enum!(ColliderShape { Box, Sphere, Capsule, Cylinder, Prism });

/// Something solid. Other things can hit it, or (as a trigger) pass through it and just notice.
/// The shape is centered at `center` (in the object's own space) and stands along the Y axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collider3DData {
    pub shape: ColliderShape,
    pub center: Vec3,
    /// The sides of a box.
    pub size: Vec3,
    pub radius: f32,
    /// The height of a capsule, cylinder or prism.
    pub height: f32,
    /// The number of sides of a prism (3 to 32).
    pub sides: i32,
    /// A trigger is not solid: things pass through it and it sends OnTrigger events.
    pub trigger: bool,
    /// 0 is slippery, 1 is very grippy.
    pub friction: f32,
    /// 0 does not bounce, 1 bounces back as fast as it came.
    pub bounciness: f32,
}

impl Default for Collider3DData {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box,
            center: Vec3::ZERO,
            size: Vec3::ONE,
            radius: 0.5,
            height: 2.0,
            sides: 6,
            trigger: false,
            friction: 0.5,
            bounciness: 0.0,
        }
    }
}
