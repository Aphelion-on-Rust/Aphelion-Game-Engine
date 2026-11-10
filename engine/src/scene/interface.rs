use crate::identity::{AssetUuid, ComponentId};
use crate::math::{Vec2, Vec3};
use crate::mivc::Interface;
use serde::{Deserialize, Serialize};

/// Position, rotation and scale of a 3D object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformData {
    pub position: Vec3,
    /// Euler angles in degrees.
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for TransformData {
    fn default() -> Self {
        Self { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }
    }
}

/// Layout of a UI element: anchors, pivot, position and size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RectTransformData {
    pub anchor_min: Vec2,
    pub anchor_max: Vec2,
    pub pivot: Vec2,
    pub position: Vec2,
    pub size: Vec2,
}

impl Default for RectTransformData {
    fn default() -> Self {
        Self {
            anchor_min: Vec2::new(0.5, 0.5),
            anchor_max: Vec2::new(0.5, 0.5),
            pivot: Vec2::new(0.5, 0.5),
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 100.0),
        }
    }
}

/// A script attached to an object. `script` is the uuid of the script asset.
/// It never changes when the script file is edited.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptData {
    pub script: AssetUuid,
    /// A disabled script gets no Update, FixedUpdate, mouse events ...
    pub enabled: bool,
}

/// The data of one component. Step 3 adds the rest of the catalog here
/// (physics, rendering, audio, UI ...). In YAML it looks like:
///
/// ```yaml
/// type: Transform
/// position: {x: 0, y: 0, z: 0}
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComponentData {
    Transform(TransformData),
    RectTransform(RectTransformData),
    Script(ScriptData),
}

impl ComponentData {
    /// Transform and RectTransform control placement. An object has exactly one of them.
    pub fn is_placement(&self) -> bool {
        matches!(self, ComponentData::Transform(_) | ComponentData::RectTransform(_))
    }
}

/// A component plus its fixed id (starts at 10000).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentEntry {
    pub id: ComponentId,
    #[serde(flatten)]
    pub data: ComponentData,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameObjectData {
    pub uuid: AssetUuid,
    pub name: String,
    pub active: bool,
    pub parent: Option<AssetUuid>,
    pub children: Vec<AssetUuid>,
    pub components: Vec<ComponentEntry>,
    /// Set on the root of an object that was created from a prefab.
    #[serde(default)]
    pub prefab_source: Option<AssetUuid>,
}

/// A saved scene. Objects with no parent are the roots of the hierarchy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneData {
    pub uuid: AssetUuid,
    pub name: String,
    pub objects: Vec<GameObjectData>,
}

impl Interface for SceneData {
    const FORMAT: &'static str = "scene";
    const VERSION: u32 = 1;
}
