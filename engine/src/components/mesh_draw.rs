use crate::identity::AssetUuid;
use crate::math::{Rgba, Vec2};
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
    /// A texture asset. Its colors are multiplied with `color`.
    #[serde(default)]
    pub texture: Option<AssetUuid>,
    /// How often the texture repeats across the shape.
    #[serde(default = "unit_scale")]
    pub tiling: Vec2,
    /// Moves the texture across the shape.
    #[serde(default)]
    pub offset: Vec2,
}

fn unit_scale() -> Vec2 {
    Vec2::new(1.0, 1.0)
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
            texture: None,
            tiling: unit_scale(),
            offset: Vec2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::scene::{ComponentData, SceneController, SceneData};
    use crate::storage::yaml::{from_yaml, to_yaml};

    #[test]
    fn a_mesh_with_a_texture_survives_yaml() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let object = SceneController.create_object(&mut scene, &mut ids, "Cube", None).unwrap();
        let texture = IdController.new_uuid(&mut ids);
        let data = ComponentData::MeshDraw(MeshDrawData {
            texture: Some(texture),
            tiling: Vec2::new(2.0, 3.0),
            offset: Vec2::new(0.25, 0.5),
            ..Default::default()
        });

        SceneController.add_component(&mut scene, &mut ids, &object, data).unwrap();

        let loaded: SceneData = from_yaml(&to_yaml(crate::mivc::Model::data(&scene)).unwrap()).unwrap();

        assert_eq!(&loaded, crate::mivc::Model::data(&scene));
    }

    #[test]
    fn a_mesh_saved_before_textures_existed_still_loads_with_defaults() {
        let mut ids = IdRegistryModel::default();
        let (scene_id, object_id) = (IdController.new_uuid(&mut ids), IdController.new_uuid(&mut ids));
        let old = format!(
            "format: scene\nversion: 1\ndata:\n  uuid: {scene_id}\n  name: Old\n  objects:\n  - uuid: {object_id}\n    name: Cube\n    active: true\n    parent: null\n    children: []\n    components:\n    - id: 10000\n      type: MeshDraw\n      mesh: Cube\n      color: {{r: 1.0, g: 1.0, b: 1.0, a: 1.0}}\n      unlit: false\n      cast_shadows: true\n      receive_shadows: true\n      visible: true\n"
        );
        let scene: SceneData = from_yaml(&old).unwrap();

        match &scene.objects[0].components[0].data {
            ComponentData::MeshDraw(mesh) => {
                assert_eq!(mesh.texture, None);
                assert_eq!(mesh.tiling, Vec2::new(1.0, 1.0));
                assert_eq!(mesh.offset, Vec2::ZERO);
            }
            other => panic!("expected a mesh, got {other:?}"),
        }
    }
}
