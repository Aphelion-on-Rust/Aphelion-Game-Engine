//! Where an object is in the world: its own matrix after all its parents'.
//!
//! The coordinate system is left-handed with Y up and Z forward. Rotation is Euler angles in degrees,
//! applied in the order Z, then X, then Y (the same convention as Unity).

use super::interface::{ComponentData, GameObjectData};
use super::query;
use crate::identity::AssetUuid;
use glam::{EulerRot, Mat4, Quat, Vec3};

/// The matrix of an object relative to its parent. An object with no Transform is not moved.
/// (A RectTransform only moves the object by its position for now, UI drawing comes later.)
pub fn local_matrix(object: &GameObjectData) -> Mat4 {
    for entry in &object.components {
        match &entry.data {
            ComponentData::Transform(transform) => {
                // Z first, then X, then Y.
                let rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    transform.rotation.y.to_radians(),
                    transform.rotation.x.to_radians(),
                    transform.rotation.z.to_radians(),
                );

                return Mat4::from_scale_rotation_translation(
                    Vec3::new(transform.scale.x, transform.scale.y, transform.scale.z),
                    rotation,
                    Vec3::new(transform.position.x, transform.position.y, transform.position.z),
                );
            }
            ComponentData::RectTransform(rect) => {
                return Mat4::from_translation(Vec3::new(rect.position.x, rect.position.y, 0.0));
            }
            _ => {}
        }
    }

    Mat4::IDENTITY
}

/// The matrix of an object in the world. None if the object does not exist.
pub fn world_matrix(objects: &[GameObjectData], uuid: &AssetUuid) -> Option<Mat4> {
    let mut chain = vec![query::find_object(objects, uuid)?];

    while let Some(parent) = chain.last().and_then(|object| object.parent).and_then(|parent| query::find_object(objects, &parent)) {
        chain.push(parent);

        // A broken file with a parent loop must not hang the editor.
        if chain.len() > objects.len() {
            break;
        }
    }

    Some(chain.iter().rev().fold(Mat4::IDENTITY, |world, object| world * local_matrix(object)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::IdRegistryModel;
    use crate::scene::{SceneController, SceneModel};

    fn place(scene: &mut SceneModel, object: AssetUuid, position: crate::math::Vec3, rotation: crate::math::Vec3, scale: crate::math::Vec3) {
        use crate::mivc::Model;

        let component = query::find_object(&scene.data().objects, &object).unwrap().components[0].id;

        SceneController
            .update_component(scene, &object, component, |data| {
                if let ComponentData::Transform(transform) = data {
                    transform.position = position;
                    transform.rotation = rotation;
                    transform.scale = scale;
                }
            })
            .unwrap();
    }

    #[test]
    fn a_child_is_placed_inside_its_parent_and_the_rotation_order_is_z_x_y() {
        use crate::math::Vec3 as V3;
        use crate::mivc::Model;

        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let parent = SceneController.create_object(&mut scene, &mut ids, "Parent", None).unwrap();
        let child = SceneController.create_object(&mut scene, &mut ids, "Child", Some(parent)).unwrap();

        place(&mut scene, parent, V3::new(10.0, 0.0, 0.0), V3::new(0.0, 90.0, 0.0), V3::new(2.0, 2.0, 2.0));
        place(&mut scene, child, V3::new(0.0, 0.0, 1.0), V3::ZERO, V3::ONE);

        let matrix = world_matrix(&scene.data().objects, &child).unwrap();
        let point = matrix.transform_point3(Vec3::ZERO);

        // The child is 1 forward (scaled to 2), the parent turned 90 degrees around Y: forward becomes right.
        assert!((point - Vec3::new(12.0, 0.0, 0.0)).length() < 1e-4);
    }
}
