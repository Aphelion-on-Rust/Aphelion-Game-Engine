use flux_engine::identity::AssetUuid;
use flux_engine::scene::{query, ComponentData, GameObjectData};
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

/// The matrix of an object in the world: its own matrix after all its parents'.
/// None if the object does not exist.
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
