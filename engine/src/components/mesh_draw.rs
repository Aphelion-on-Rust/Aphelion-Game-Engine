use crate::math::Rgba;
use serde::{Deserialize, Serialize};

/// The shapes that are built in. Mesh files come with the asset step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveMesh {
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Capsule,
}

named_enum!(PrimitiveMesh { Cube, Sphere, Plane, Cylinder, Capsule });

/// Draws a mesh at the object's position. (It does the work of a mesh filter plus a mesh renderer.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshDrawData {
    pub mesh: PrimitiveMesh,
    pub color: Rgba,
    /// Unlit: the color is drawn as it is, lights are ignored.
    pub unlit: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub visible: bool,
}

impl Default for MeshDrawData {
    fn default() -> Self {
        Self {
            mesh: PrimitiveMesh::Cube,
            color: Rgba::WHITE,
            unlit: false,
            cast_shadows: true,
            receive_shadows: true,
            visible: true,
        }
    }
}
