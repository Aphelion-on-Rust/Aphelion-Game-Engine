//! The colliders of a scene, placed in the world.
//!
//! Scale: the size of a collider follows the scale of its object, like in most engines.
//! A box is scaled per axis. A sphere uses the biggest scale. A capsule, cylinder or prism uses the bigger
//! of X and Z for its radius and Y for its height.

use crate::components::ColliderShape;
use crate::identity::AssetUuid;
use crate::scene::transform::world_matrix;
use crate::scene::{query, ComponentData, SceneData};
use glam::{Quat, Vec3};

/// A collider with everything already worked out for the world.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldCollider {
    pub object: AssetUuid,
    pub shape: ColliderShape,
    /// The middle of the shape in the world.
    pub position: Vec3,
    /// How the shape is turned. Scale is not in here, it is already in the sizes below.
    pub rotation: Quat,
    /// The sides of a box (box only).
    pub size: Vec3,
    pub radius: f32,
    /// The whole height of a capsule, cylinder or prism.
    pub height: f32,
    pub sides: u32,
    pub trigger: bool,
    pub friction: f32,
    pub bounciness: f32,
}

impl WorldCollider {
    /// Half the length of the straight middle part along the Y axis. For a capsule the two round ends
    /// are not counted, for a cylinder or prism it is half the height.
    pub fn half_straight(&self) -> f32 {
        match self.shape {
            ColliderShape::Capsule => (self.height * 0.5 - self.radius).max(0.0),
            _ => self.height * 0.5,
        }
    }
}

/// Every collider of the active objects of a scene, in the world. Objects scaled to nothing are skipped.
pub fn extract_colliders(scene: &SceneData) -> Vec<WorldCollider> {
    let mut colliders = Vec::new();

    for object in &scene.objects {
        if !query::is_active_in_hierarchy(&scene.objects, &object.uuid) {
            continue;
        }

        let Some(world) = world_matrix(&scene.objects, &object.uuid) else {
            continue;
        };
        let (scale, rotation, _) = world.to_scale_rotation_translation();
        let scale = scale.abs();

        if scale.min_element() <= 1e-6 {
            continue;
        }

        for entry in &object.components {
            let ComponentData::Collider3D(collider) = &entry.data else {
                continue;
            };

            let center = Vec3::new(collider.center.x, collider.center.y, collider.center.z);
            let (size, radius, height) = match collider.shape {
                ColliderShape::Box => (
                    Vec3::new(collider.size.x * scale.x, collider.size.y * scale.y, collider.size.z * scale.z),
                    0.0,
                    0.0,
                ),
                ColliderShape::Sphere => (Vec3::ZERO, collider.radius * scale.max_element(), 0.0),
                _ => (Vec3::ZERO, collider.radius * scale.x.max(scale.z), collider.height * scale.y),
            };

            colliders.push(WorldCollider {
                object: object.uuid,
                shape: collider.shape,
                position: world.transform_point3(center),
                rotation,
                size,
                radius,
                height,
                sides: collider.sides.clamp(3, 32) as u32,
                trigger: collider.trigger,
                friction: collider.friction,
                bounciness: collider.bounciness,
            });
        }
    }

    colliders
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::IdRegistryModel;
    use crate::math::Vec3 as V3;
    use crate::mivc::Model;
    use crate::scene::{ObjectPreset, SceneController, SceneModel};

    fn world() -> (SceneModel, IdRegistryModel) {
        let mut ids = IdRegistryModel::default();
        let scene = SceneController.create_scene(&mut ids, "Test");

        (scene, ids)
    }

    fn place(scene: &mut SceneModel, object: AssetUuid, position: V3, rotation: V3, scale: V3) {
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

    fn set_collider(scene: &mut SceneModel, object: AssetUuid, change: impl FnOnce(&mut crate::components::Collider3DData)) {
        let component = query::find_object(&scene.data().objects, &object).unwrap().components[2].id;

        SceneController
            .update_component(scene, &object, component, |data| {
                if let ComponentData::Collider3D(collider) = data {
                    change(collider);
                }
            })
            .unwrap();
    }

    #[test]
    fn a_box_is_scaled_per_axis_and_a_sphere_by_the_biggest_scale() {
        let (mut scene, mut ids) = world();
        let cube = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();
        let ball = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsSphere, None).unwrap();

        place(&mut scene, cube, V3::new(1.0, 2.0, 3.0), V3::ZERO, V3::new(2.0, 1.0, 3.0));
        place(&mut scene, ball, V3::ZERO, V3::ZERO, V3::new(1.0, 4.0, 2.0));

        let colliders = extract_colliders(scene.data());

        assert_eq!(colliders.len(), 2);
        assert_eq!(colliders[0].size, Vec3::new(2.0, 1.0, 3.0));
        assert!((colliders[0].position - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-5);
        assert!((colliders[1].radius - 2.0).abs() < 1e-5); // 0.5 * 4
    }

    #[test]
    fn a_capsule_takes_its_radius_from_x_and_z_and_its_height_from_y() {
        let (mut scene, mut ids) = world();
        let object = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();

        place(&mut scene, object, V3::ZERO, V3::ZERO, V3::new(2.0, 3.0, 1.0));
        set_collider(&mut scene, object, |collider| collider.shape = ColliderShape::Capsule);

        let collider = &extract_colliders(scene.data())[0];

        assert!((collider.radius - 1.0).abs() < 1e-5); // 0.5 * max(2, 1)
        assert!((collider.height - 6.0).abs() < 1e-5); // 2 * 3
        assert!((collider.half_straight() - 2.0).abs() < 1e-5); // 3 - radius 1
    }

    #[test]
    fn the_center_is_turned_with_the_object() {
        let (mut scene, mut ids) = world();
        let object = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();

        place(&mut scene, object, V3::ZERO, V3::new(0.0, 90.0, 0.0), V3::ONE);
        set_collider(&mut scene, object, |collider| collider.center = V3::new(1.0, 0.0, 0.0));

        let collider = &extract_colliders(scene.data())[0];

        // Turned 90 degrees around Y, "right" becomes "backward".
        assert!((collider.position - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-4);
    }

    #[test]
    fn inactive_objects_and_objects_scaled_to_nothing_have_no_colliders() {
        let (mut scene, mut ids) = world();
        let off = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();
        let flat = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();
        let kept = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();

        SceneController.set_active(&mut scene, &off, false).unwrap();
        place(&mut scene, flat, V3::ZERO, V3::ZERO, V3::new(1.0, 0.0, 1.0));

        let colliders = extract_colliders(scene.data());

        assert_eq!(colliders.len(), 1);
        assert_eq!(colliders[0].object, kept);
    }
}
