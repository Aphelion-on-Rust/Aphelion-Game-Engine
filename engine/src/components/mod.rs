//! Components that make things appear (a camera, lights, meshes, sprites) and things solid (colliders, bodies).
//!
//! They have their own names on purpose: ViewCamera, LightSource, MeshDraw, SpriteDraw.
//! Their fields are described in `inspect/builtin.rs`, so the Inspector shows them with no extra code.
//! What turns them into pictures is the `grp` crate.

/// Gives an enum `NAMES`, `name()` and `from_name()`, so it can be an Enum field in the Inspector
/// and be saved by name.
macro_rules! named_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        impl $name {
            /// The variant names, in order. They are the options of the Enum field.
            pub const NAMES: &'static [&'static str] = &[$(stringify!($variant)),+];

            pub fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant),)+
                }
            }

            pub fn from_name(text: &str) -> Option<Self> {
                $(if text == stringify!($variant) {
                    return Some($name::$variant);
                })+

                None
            }
        }
    };
}

pub mod body;
pub mod camera;
pub mod collider;
pub mod light;
pub mod mesh_draw;
pub mod sprite_draw;

pub use body::{Body3DData, BodyKind};
pub use camera::{Projection, ViewCameraData};
pub use collider::{Collider3DData, ColliderShape};
pub use light::{LightData, LightKind};
pub use mesh_draw::{MeshDrawData, PrimitiveMesh};
pub use sprite_draw::SpriteDrawData;
