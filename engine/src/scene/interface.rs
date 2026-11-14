use crate::components::{LightData, MeshDrawData, SpriteDrawData, ViewCameraData};
use crate::identity::{AssetUuid, ComponentId};
use crate::inspect::FieldEntry;
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
    /// The values of the script's serialized fields (what the Inspector edits).
    /// Which fields exist comes from the script's schema, see the `inspect` module.
    #[serde(default)]
    pub fields: Vec<FieldEntry>,
}

impl ScriptData {
    /// A script with no field values yet.
    pub fn new(script: AssetUuid, enabled: bool) -> Self {
        Self { script, enabled, fields: Vec::new() }
    }
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
    ViewCamera(ViewCameraData),
    LightSource(LightData),
    MeshDraw(MeshDrawData),
    SpriteDraw(SpriteDrawData),
}

impl ComponentData {
    /// The components a user can add from the Inspector. A Transform is always there already,
    /// and a script needs a script asset.
    pub const ADDABLE: &'static [&'static str] = &["ViewCamera", "LightSource", "MeshDraw", "SpriteDraw"];

    /// A new component of the named kind with its default values.
    pub fn new_of_kind(kind: &str) -> Option<ComponentData> {
        match kind {
            "ViewCamera" => Some(ComponentData::ViewCamera(ViewCameraData::default())),
            "LightSource" => Some(ComponentData::LightSource(LightData::default())),
            "MeshDraw" => Some(ComponentData::MeshDraw(MeshDrawData::default())),
            "SpriteDraw" => Some(ComponentData::SpriteDraw(SpriteDrawData::default())),
            _ => None,
        }
    }

    /// The name of the kind, as used in YAML and in the Inspector.
    pub fn type_name(&self) -> &'static str {
        match self {
            ComponentData::Transform(_) => "Transform",
            ComponentData::RectTransform(_) => "RectTransform",
            ComponentData::Script(_) => "Script",
            ComponentData::ViewCamera(_) => "ViewCamera",
            ComponentData::LightSource(_) => "LightSource",
            ComponentData::MeshDraw(_) => "MeshDraw",
            ComponentData::SpriteDraw(_) => "SpriteDraw",
        }
    }

    /// Puts every value back to its default. A script is left alone (its defaults come from its schema).
    pub fn reset(&mut self) {
        match self {
            ComponentData::Transform(data) => *data = TransformData::default(),
            ComponentData::RectTransform(data) => *data = RectTransformData::default(),
            ComponentData::ViewCamera(data) => *data = ViewCameraData::default(),
            ComponentData::LightSource(data) => *data = LightData::default(),
            ComponentData::MeshDraw(data) => *data = MeshDrawData::default(),
            ComponentData::SpriteDraw(data) => *data = SpriteDrawData::default(),
            ComponentData::Script(_) => {}
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_addable_kind_can_be_made_and_names_itself() {
        for kind in ComponentData::ADDABLE {
            assert_eq!(ComponentData::new_of_kind(kind).unwrap().type_name(), *kind);
        }

        assert!(ComponentData::new_of_kind("Transform").is_none());
        assert!(ComponentData::new_of_kind("Nonsense").is_none());
    }

    #[test]
    fn reset_brings_back_the_defaults() {
        let mut camera = ViewCameraData::default();

        camera.fov_degrees = 90.0;

        let mut data = ComponentData::ViewCamera(camera);

        data.reset();

        assert_eq!(data, ComponentData::ViewCamera(ViewCameraData::default()));
    }
}
