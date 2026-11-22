//! Ready-made objects for the "Create" menu: a cube, a camera, a light ...

use super::interface::{ComponentData, TransformData};
use crate::components::{
    Body3DData, BodyKind, Collider3DData, ColliderShape, LightData, LightKind, MeshDrawData, PrimitiveMesh, SpriteDrawData,
    ViewCameraData,
};
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
    /// A cube that falls and can be hit (mesh, collider and dynamic body).
    PhysicsCube,
    PhysicsSphere,
    /// A flat solid floor that never moves.
    Floor,
}

impl ObjectPreset {
    pub const ALL: [ObjectPreset; 14] = [
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
        ObjectPreset::PhysicsCube,
        ObjectPreset::PhysicsSphere,
        ObjectPreset::Floor,
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
            ObjectPreset::PhysicsCube => "Physics Cube",
            ObjectPreset::PhysicsSphere => "Physics Sphere",
            ObjectPreset::Floor => "Floor",
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
            ObjectPreset::PhysicsCube => transform.position = Vec3::new(0.0, 5.0, 0.0),
            ObjectPreset::PhysicsSphere => transform.position = Vec3::new(1.0, 6.5, 0.0),
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
            ObjectPreset::PhysicsCube => vec![
                ComponentData::MeshDraw(MeshDrawData { mesh: PrimitiveMesh::Cube, ..Default::default() }),
                ComponentData::Collider3D(Collider3DData::default()),
                ComponentData::Body3D(Body3DData::default()),
            ],
            ObjectPreset::PhysicsSphere => vec![
                ComponentData::MeshDraw(MeshDrawData { mesh: PrimitiveMesh::Sphere, ..Default::default() }),
                ComponentData::Collider3D(Collider3DData { shape: ColliderShape::Sphere, ..Default::default() }),
                ComponentData::Body3D(Body3DData::default()),
            ],
            ObjectPreset::Floor => vec![
                ComponentData::MeshDraw(MeshDrawData { mesh: PrimitiveMesh::Plane, ..Default::default() }),
                // The plane has no thickness, so the collider is a thin box just below it.
                ComponentData::Collider3D(Collider3DData {
                    size: Vec3::new(10.0, 0.2, 10.0),
                    center: Vec3::new(0.0, -0.1, 0.0),
                    ..Default::default()
                }),
                ComponentData::Body3D(Body3DData { kind: BodyKind::Static, ..Default::default() }),
            ],
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

    #[test]
    fn the_physics_presets_have_a_mesh_a_collider_and_a_body() {
        let kinds = |preset: ObjectPreset| -> Vec<&'static str> {
            preset.components().iter().map(|component| component.type_name()).collect()
        };

        assert_eq!(kinds(ObjectPreset::PhysicsCube), vec!["MeshDraw", "Collider3D", "Body3D"]);
        assert_eq!(kinds(ObjectPreset::PhysicsSphere), vec!["MeshDraw", "Collider3D", "Body3D"]);
        assert_eq!(kinds(ObjectPreset::Floor), vec!["MeshDraw", "Collider3D", "Body3D"]);

        match &ObjectPreset::Floor.components()[2] {
            ComponentData::Body3D(body) => assert_eq!(body.kind, BodyKind::Static),
            other => panic!("expected a body, got {other:?}"),
        }
    }
}
