//! Ready-made objects for the "Create" menu: a cube, a camera, a light ...

use super::interface::{ComponentData, TransformData};
use crate::components::{LightData, LightKind, MeshDrawData, PrimitiveMesh, SpriteDrawData, ViewCameraData};
use crate::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectPreset {
    Empty,
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Capsule,
    Camera,
    DirectionalLight,
    PointLight,
    SpotLight,
    Sprite,
}

impl ObjectPreset {
    pub const ALL: [ObjectPreset; 11] = [
        ObjectPreset::Empty,
        ObjectPreset::Cube,
        ObjectPreset::Sphere,
        ObjectPreset::Plane,
        ObjectPreset::Cylinder,
        ObjectPreset::Capsule,
        ObjectPreset::Camera,
        ObjectPreset::DirectionalLight,
        ObjectPreset::PointLight,
        ObjectPreset::SpotLight,
        ObjectPreset::Sprite,
    ];

    /// The name of the new object.
    pub fn name(self) -> &'static str {
        match self {
            ObjectPreset::Empty => "GameObject",
            ObjectPreset::Cube => "Cube",
            ObjectPreset::Sphere => "Sphere",
            ObjectPreset::Plane => "Plane",
            ObjectPreset::Cylinder => "Cylinder",
            ObjectPreset::Capsule => "Capsule",
            ObjectPreset::Camera => "Camera",
            ObjectPreset::DirectionalLight => "Directional Light",
            ObjectPreset::PointLight => "Point Light",
            ObjectPreset::SpotLight => "Spot Light",
            ObjectPreset::Sprite => "Sprite",
        }
    }

    /// Where the new object starts. Lights and cameras are placed so they are useful right away.
    pub fn transform(self) -> TransformData {
        let mut transform = TransformData::default();

        match self {
            ObjectPreset::Camera => transform.position = Vec3::new(0.0, 1.0, -10.0),
            ObjectPreset::DirectionalLight => transform.rotation = Vec3::new(50.0, -30.0, 0.0),
            ObjectPreset::PointLight => transform.position = Vec3::new(0.0, 3.0, 0.0),
            ObjectPreset::SpotLight => {
                transform.position = Vec3::new(0.0, 5.0, 0.0);
                transform.rotation = Vec3::new(90.0, 0.0, 0.0);
            }
            _ => {}
        }

        transform
    }

    /// The components the object gets besides its Transform.
    pub fn components(self) -> Vec<ComponentData> {
        let mesh = |mesh| vec![ComponentData::MeshDraw(MeshDrawData { mesh, ..Default::default() })];
        let light = |kind| vec![ComponentData::LightSource(LightData { kind, ..Default::default() })];

        match self {
            ObjectPreset::Empty => Vec::new(),
            ObjectPreset::Cube => mesh(PrimitiveMesh::Cube),
            ObjectPreset::Sphere => mesh(PrimitiveMesh::Sphere),
            ObjectPreset::Plane => mesh(PrimitiveMesh::Plane),
            ObjectPreset::Cylinder => mesh(PrimitiveMesh::Cylinder),
            ObjectPreset::Capsule => mesh(PrimitiveMesh::Capsule),
            ObjectPreset::Camera => vec![ComponentData::ViewCamera(ViewCameraData::default())],
            ObjectPreset::DirectionalLight => light(LightKind::Directional),
            ObjectPreset::PointLight => light(LightKind::Point),
            ObjectPreset::SpotLight => light(LightKind::Spot),
            ObjectPreset::Sprite => vec![ComponentData::SpriteDraw(SpriteDrawData::default())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_has_a_name_and_the_right_components() {
        for preset in ObjectPreset::ALL {
            assert!(!preset.name().is_empty());
        }

        let kinds = |preset: ObjectPreset| -> Vec<&'static str> {
            preset.components().iter().map(|component| component.type_name()).collect()
        };

        assert!(kinds(ObjectPreset::Empty).is_empty());
        assert_eq!(kinds(ObjectPreset::Sphere), vec!["MeshDraw"]);
        assert_eq!(kinds(ObjectPreset::Camera), vec!["ViewCamera"]);
        assert_eq!(kinds(ObjectPreset::SpotLight), vec!["LightSource"]);
        assert_eq!(kinds(ObjectPreset::Sprite), vec!["SpriteDraw"]);
    }

    #[test]
    fn a_directional_light_is_tilted_and_a_camera_is_pulled_back() {
        assert_eq!(ObjectPreset::DirectionalLight.transform().rotation, Vec3::new(50.0, -30.0, 0.0));
        assert_eq!(ObjectPreset::Camera.transform().position.z, -10.0);
        assert_eq!(ObjectPreset::Cube.transform(), TransformData::default());
    }
}
