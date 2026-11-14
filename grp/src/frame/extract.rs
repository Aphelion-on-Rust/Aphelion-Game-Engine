use super::interface::{RenderCamera, RenderFrame, RenderLight, RenderMesh, RenderSprite};
use super::transform::world_matrix;
use flux_engine::components::{Projection, ViewCameraData};
use flux_engine::identity::AssetUuid;
use flux_engine::math::Rgba;
use flux_engine::scene::{query, ComponentData, GameObjectData, SceneData};
use glam::{Mat4, Vec3};

fn rgba(color: Rgba) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

/// The matrix that squashes the camera's view into clip space.
pub fn projection_matrix(camera: &ViewCameraData, aspect: f32) -> Mat4 {
    match camera.projection {
        Projection::Perspective => Mat4::perspective_lh(camera.fov_degrees.to_radians(), aspect, camera.near, camera.far),
        Projection::Orthographic => {
            let half_height = camera.ortho_size;
            let half_width = half_height * aspect;

            Mat4::orthographic_lh(-half_width, half_width, -half_height, half_height, camera.near, camera.far)
        }
    }
}

/// A camera for an object that sits at `world` (the matrix of its object).
pub fn camera_from_world(object: Option<AssetUuid>, world: Mat4, camera: &ViewCameraData, aspect: f32) -> RenderCamera {
    RenderCamera {
        object,
        view: world.inverse(),
        projection: projection_matrix(camera, aspect),
        position: world.w_axis.truncate(),
        clear_color: rgba(camera.clear_color),
        near: camera.near,
        far: camera.far,
    }
}

/// The camera the Game view should use: the active ViewCamera with the highest priority
/// (the first one wins a tie). None if the scene has no camera.
pub fn find_game_camera(scene: &SceneData, aspect: f32) -> Option<RenderCamera> {
    let mut best: Option<(i32, &GameObjectData, &ViewCameraData)> = None;

    for object in &scene.objects {
        if !query::is_active_in_hierarchy(&scene.objects, &object.uuid) {
            continue;
        }

        for entry in &object.components {
            if let ComponentData::ViewCamera(camera) = &entry.data {
                if best.map_or(true, |(priority, _, _)| camera.priority > priority) {
                    best = Some((camera.priority, object, camera));
                }
            }
        }
    }

    let (_, object, camera) = best?;
    let world = world_matrix(&scene.objects, &object.uuid)?;

    Some(camera_from_world(Some(object.uuid), world, camera, aspect))
}

/// A camera for a scene that has none: a little above the middle, looking a bit down.
pub fn fallback_camera(aspect: f32) -> RenderCamera {
    let camera = ViewCameraData::default();
    let eye = Vec3::new(0.0, 2.0, -8.0);
    let forward = Vec3::new(0.0, -0.2, 1.0).normalize();

    RenderCamera {
        object: None,
        view: Mat4::look_to_lh(eye, forward, Vec3::Y),
        projection: projection_matrix(&camera, aspect),
        position: eye,
        clear_color: rgba(camera.clear_color),
        near: camera.near,
        far: camera.far,
    }
}

/// Collects everything to draw from a scene, seen from `camera`.
/// Inactive objects (and objects inside inactive ones) and parts marked not visible are left out.
pub fn collect(scene: &SceneData, camera: RenderCamera) -> RenderFrame {
    let objects = &scene.objects;
    let mut frame = RenderFrame { camera, meshes: Vec::new(), sprites: Vec::new(), lights: Vec::new() };

    for object in objects {
        if !query::is_active_in_hierarchy(objects, &object.uuid) {
            continue;
        }

        let Some(world) = world_matrix(objects, &object.uuid) else {
            continue;
        };

        for entry in &object.components {
            match &entry.data {
                ComponentData::MeshDraw(mesh) if mesh.visible => frame.meshes.push(RenderMesh {
                    object: object.uuid,
                    mesh: mesh.mesh,
                    model: world,
                    color: rgba(mesh.color),
                    unlit: mesh.unlit,
                    cast_shadows: mesh.cast_shadows,
                    receive_shadows: mesh.receive_shadows,
                }),
                ComponentData::SpriteDraw(sprite) if sprite.visible => frame.sprites.push(RenderSprite {
                    object: object.uuid,
                    model: world * Mat4::from_scale(Vec3::new(sprite.size.x, sprite.size.y, 1.0)),
                    color: rgba(sprite.color),
                    flip_x: sprite.flip_x,
                    flip_y: sprite.flip_y,
                    order: sprite.order,
                }),
                ComponentData::LightSource(light) => frame.lights.push(RenderLight {
                    object: object.uuid,
                    kind: light.kind,
                    color: [light.color.r, light.color.g, light.color.b],
                    intensity: light.intensity,
                    position: world.w_axis.truncate(),
                    direction: world.transform_vector3(Vec3::Z).normalize_or_zero(),
                    range: light.range,
                    spot_angle_degrees: light.spot_angle_degrees,
                    shadows: light.shadows,
                }),
                _ => {}
            }
        }
    }

    // A stable sort: sprites with the same order stay in scene order.
    frame.sprites.sort_by_key(|sprite| sprite.order);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_engine::identity::IdRegistryModel;
    use flux_engine::math::Vec3 as V3;
    use flux_engine::scene::{ObjectPreset, SceneController, SceneModel};
    use glam::Vec4;

    struct World {
        scene: SceneModel,
        ids: IdRegistryModel,
    }

    fn world() -> World {
        let mut ids = IdRegistryModel::default();
        let scene = SceneController.create_scene(&mut ids, "Test");

        World { scene, ids }
    }

    fn add(world: &mut World, preset: ObjectPreset, parent: Option<AssetUuid>) -> AssetUuid {
        SceneController.create_from_preset(&mut world.scene, &mut world.ids, preset, parent).unwrap()
    }

    fn place(world: &mut World, object: AssetUuid, position: V3, rotation: V3, scale: V3) {
        let component = query::find_object(&data(world).objects, &object).unwrap().components[0].id;

        SceneController
            .update_component(&mut world.scene, &object, component, |data| {
                if let ComponentData::Transform(transform) = data {
                    transform.position = position;
                    transform.rotation = rotation;
                    transform.scale = scale;
                }
            })
            .unwrap();
    }

    fn edit(world: &mut World, object: AssetUuid, index: usize, change: impl FnOnce(&mut ComponentData)) {
        let component = query::find_object(&data(world).objects, &object).unwrap().components[index].id;

        SceneController.update_component(&mut world.scene, &object, component, change).unwrap();
    }

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-4
    }

    fn data(world: &World) -> &SceneData {
        use flux_engine::mivc::Model;

        world.scene.data()
    }

    #[test]
    fn a_child_is_placed_inside_its_parents_position_and_scale() {
        let mut world = world();
        let parent = add(&mut world, ObjectPreset::Empty, None);
        let child = add(&mut world, ObjectPreset::Empty, Some(parent));

        place(&mut world, parent, V3::new(10.0, 0.0, 0.0), V3::ZERO, V3::new(2.0, 2.0, 2.0));
        place(&mut world, child, V3::new(1.0, 0.0, 0.0), V3::ZERO, V3::ONE);

        let matrix = world_matrix(&data(&world).objects, &child).unwrap();

        assert!(close(matrix.transform_point3(Vec3::ZERO), Vec3::new(12.0, 0.0, 0.0)));
    }

    #[test]
    fn rotating_90_degrees_around_y_turns_forward_into_right() {
        let mut world = world();
        let object = add(&mut world, ObjectPreset::Empty, None);

        place(&mut world, object, V3::ZERO, V3::new(0.0, 90.0, 0.0), V3::ONE);

        let matrix = world_matrix(&data(&world).objects, &object).unwrap();

        assert!(close(matrix.transform_point3(Vec3::new(0.0, 0.0, 1.0)), Vec3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn inactive_objects_their_children_and_invisible_parts_are_not_drawn() {
        let mut world = world();
        let shown = add(&mut world, ObjectPreset::Cube, None);
        let hidden_parent = add(&mut world, ObjectPreset::Cube, None);
        add(&mut world, ObjectPreset::Sphere, Some(hidden_parent));
        let invisible = add(&mut world, ObjectPreset::Cube, None);

        SceneController.set_active(&mut world.scene, &hidden_parent, false).unwrap();
        edit(&mut world, invisible, 1, |data| {
            if let ComponentData::MeshDraw(mesh) = data {
                mesh.visible = false;
            }
        });

        let frame = collect(data(&world), fallback_camera(1.0));

        assert_eq!(frame.meshes.len(), 1);
        assert_eq!(frame.meshes[0].object, shown);
    }

    #[test]
    fn the_camera_with_the_highest_priority_is_the_game_camera() {
        let mut world = world();

        assert!(find_game_camera(data(&world), 1.0).is_none());

        let first = add(&mut world, ObjectPreset::Camera, None);
        let second = add(&mut world, ObjectPreset::Camera, None);

        assert_eq!(find_game_camera(data(&world), 1.0).unwrap().object, Some(first)); // a tie: the first one

        edit(&mut world, second, 1, |data| {
            if let ComponentData::ViewCamera(camera) = data {
                camera.priority = 5;
            }
        });

        assert_eq!(find_game_camera(data(&world), 1.0).unwrap().object, Some(second));

        SceneController.set_active(&mut world.scene, &second, false).unwrap();
        assert_eq!(find_game_camera(data(&world), 1.0).unwrap().object, Some(first));
    }

    #[test]
    fn a_perspective_camera_sees_what_is_in_front_and_not_what_is_behind() {
        let mut world = world();
        let camera = add(&mut world, ObjectPreset::Camera, None);

        place(&mut world, camera, V3::ZERO, V3::ZERO, V3::ONE);

        let camera = find_game_camera(data(&world), 1.0).unwrap();
        let clip = camera.projection * camera.view;
        let ahead = clip.project_point3(Vec3::new(0.0, 0.0, 5.0));

        assert!(ahead.x.abs() < 1e-4 && ahead.y.abs() < 1e-4);
        assert!(ahead.z > 0.0 && ahead.z < 1.0);
        assert!((clip * Vec4::new(0.0, 0.0, -5.0, 1.0)).w < 0.0);
    }

    #[test]
    fn an_orthographic_camera_is_as_wide_as_its_size_times_the_aspect() {
        let camera = ViewCameraData { projection: Projection::Orthographic, ortho_size: 2.0, ..Default::default() };
        let projection = projection_matrix(&camera, 2.0);

        assert!((projection.project_point3(Vec3::new(4.0, 0.0, 5.0)).x - 1.0).abs() < 1e-4);
        assert!((projection.project_point3(Vec3::new(0.0, 2.0, 5.0)).y - 1.0).abs() < 1e-4);
    }

    #[test]
    fn a_light_shines_along_its_forward_direction_from_its_world_position() {
        let mut world = world();
        let parent = add(&mut world, ObjectPreset::Empty, None);
        let light = add(&mut world, ObjectPreset::DirectionalLight, Some(parent));

        place(&mut world, parent, V3::new(0.0, 4.0, 0.0), V3::ZERO, V3::ONE);
        place(&mut world, light, V3::new(1.0, 0.0, 0.0), V3::new(90.0, 0.0, 0.0), V3::ONE);

        let frame = collect(data(&world), fallback_camera(1.0));

        assert_eq!(frame.lights.len(), 1);
        assert!(close(frame.lights[0].direction, Vec3::new(0.0, -1.0, 0.0)));
        assert!(close(frame.lights[0].position, Vec3::new(1.0, 4.0, 0.0)));
    }

    #[test]
    fn sprites_come_out_sorted_by_order_and_carry_their_size() {
        let mut world = world();
        let orders = [5, 1, 3];
        let mut sprites = Vec::new();

        for order in orders {
            let sprite = add(&mut world, ObjectPreset::Sprite, None);

            edit(&mut world, sprite, 1, |data| {
                if let ComponentData::SpriteDraw(draw) = data {
                    draw.order = order;
                    draw.size = flux_engine::math::Vec2::new(2.0, 3.0);
                }
            });
            sprites.push(sprite);
        }

        let frame = collect(data(&world), fallback_camera(1.0));
        let sorted: Vec<i32> = frame.sprites.iter().map(|sprite| sprite.order).collect();

        assert_eq!(sorted, vec![1, 3, 5]);
        assert!(close(frame.sprites[0].model.transform_vector3(Vec3::X), Vec3::new(2.0, 0.0, 0.0)));
        assert!(close(frame.sprites[0].model.transform_vector3(Vec3::Y), Vec3::new(0.0, 3.0, 0.0)));
    }

    #[test]
    fn the_fallback_camera_can_be_inverted_and_looks_at_the_middle() {
        let camera = fallback_camera(1.5);
        let clip = camera.projection * camera.view;
        let middle = clip.project_point3(Vec3::ZERO);

        assert!(middle.x.abs() < 0.2);
        assert!(middle.z > 0.0 && middle.z < 1.0);
        assert!(camera.view.determinant().abs() > 0.0);
        assert!(camera.object.is_none());
    }
}
