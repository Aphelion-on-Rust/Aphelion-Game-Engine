use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    /// Moved by forces and gravity.
    Dynamic,
    /// Moved only by code, but pushes dynamic bodies.
    Kinematic,
    /// Never moves.
    Static,
}

named_enum!(BodyKind { Dynamic, Kinematic, Static });

/// Makes an object move like a physical thing. (It does the work of a rigidbody.)
/// Colliders on the same object give it its shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body3DData {
    pub kind: BodyKind,
    pub mass: f32,
    /// 1 is normal gravity, 0 floats, 2 falls twice as fast.
    pub gravity_scale: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub lock_rotation: bool,
}

impl Default for Body3DData {
    fn default() -> Self {
        Self { kind: BodyKind::Dynamic, mass: 1.0, gravity_scale: 1.0, linear_damping: 0.0, angular_damping: 0.05, lock_rotation: false }
    }
}
