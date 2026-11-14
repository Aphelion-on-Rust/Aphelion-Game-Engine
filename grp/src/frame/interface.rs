use flux_engine::components::{LightKind, PrimitiveMesh};
use flux_engine::identity::AssetUuid;
use glam::{Mat4, Vec3};

/// The camera a frame is drawn from.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderCamera {
    /// The object with the camera. None for the built-in fallback camera.
    pub object: Option<AssetUuid>,
    /// World space to camera space.
    pub view: Mat4,
    /// Camera space to clip space (depth from 0 to 1).
    pub projection: Mat4,
    pub position: Vec3,
    pub clear_color: [f32; 4],
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMesh {
    pub object: AssetUuid,
    pub mesh: PrimitiveMesh,
    /// The mesh's own space to world space.
    pub model: Mat4,
    pub color: [f32; 4],
    pub unlit: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSprite {
    pub object: AssetUuid,
    /// Includes the size of the sprite. A 1 x 1 quad drawn with this matrix is the sprite.
    pub model: Mat4,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
    pub order: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderLight {
    pub object: AssetUuid,
    pub kind: LightKind,
    /// Red, green and blue.
    pub color: [f32; 3],
    pub intensity: f32,
    pub position: Vec3,
    /// The way the light shines (the +Z direction of its object), length 1.
    pub direction: Vec3,
    pub range: f32,
    pub spot_angle_degrees: f32,
    pub shadows: bool,
}

/// Everything needed to draw one picture of a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrame {
    pub camera: RenderCamera,
    pub meshes: Vec<RenderMesh>,
    /// Sorted: the sprite with the lowest order first, so later ones are drawn on top.
    pub sprites: Vec<RenderSprite>,
    pub lights: Vec<RenderLight>,
}
