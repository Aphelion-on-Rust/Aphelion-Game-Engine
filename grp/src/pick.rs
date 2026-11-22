//! Finding out what is under the mouse: a ray from the camera through a point of the picture,
//! tested against the meshes and sprites of a `RenderFrame`.

use crate::frame::{RenderCamera, RenderFrame};
use crate::software::mesh::primitive;
use flux_engine::identity::AssetUuid;
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    /// Length 1.
    pub direction: Vec3,
}

/// The ray through a point of the picture. `ndc_x` and `ndc_y` go from -1 to 1: x to the right,
/// y UP (the middle of the picture is 0, 0). Works for perspective and orthographic cameras.
pub fn ray_from_ndc(camera: &RenderCamera, ndc_x: f32, ndc_y: f32) -> Ray {
    let inverse = (camera.projection * camera.view).inverse();
    let near = inverse.project_point3(Vec3::new(ndc_x, ndc_y, 0.0));
    let far = inverse.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));

    Ray { origin: near, direction: (far - near).normalize_or_zero() }
}

/// How far along the ray a triangle is hit (both sides count), or None.
fn ray_triangle(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let edge1 = b - a;
    let edge2 = c - a;
    let p = direction.cross(edge2);
    let determinant = edge1.dot(p);

    if determinant.abs() < 1e-9 {
        return None;
    }

    let inverse = 1.0 / determinant;
    let from_a = origin - a;
    let u = from_a.dot(p) * inverse;

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = from_a.cross(edge1);
    let v = direction.dot(q) * inverse;

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = edge2.dot(q) * inverse;

    (distance > 1e-5).then_some(distance)
}

/// The object under a point of the picture: the nearest mesh or sprite the ray hits.
pub fn pick(frame: &RenderFrame, ndc_x: f32, ndc_y: f32) -> Option<AssetUuid> {
    let ray = ray_from_ndc(&frame.camera, ndc_x, ndc_y);

    if ray.direction == Vec3::ZERO {
        return None;
    }

    let mut best: Option<(f32, AssetUuid)> = None;
    let mut consider = |distance: f32, object: AssetUuid| {
        if best.map_or(true, |(nearest, _)| distance < nearest) {
            best = Some((distance, object));
        }
    };

    for mesh in &frame.meshes {
        let inverse = mesh.model.inverse();

        if !inverse.is_finite() {
            continue;
        }

        // Test in the mesh's own space. The direction is not made length 1 again, so the distance
        // along it is the same as in the world.
        let origin = inverse.transform_point3(ray.origin);
        let direction = inverse.transform_vector3(ray.direction);
        let shape = primitive(mesh.mesh);

        for corners in shape.indices.chunks_exact(3) {
            let (a, b, c) = (shape.positions[corners[0] as usize], shape.positions[corners[1] as usize], shape.positions[corners[2] as usize]);

            if let Some(distance) = ray_triangle(origin, direction, a, b, c) {
                consider(distance, mesh.object);
            }
        }
    }

    for sprite in &frame.sprites {
        let inverse = sprite.model.inverse();

        if !inverse.is_finite() {
            continue;
        }

        // A sprite is the square from -0.5 to 0.5 in its own XY plane.
        let origin = inverse.transform_point3(ray.origin);
        let direction = inverse.transform_vector3(ray.direction);

        if direction.z.abs() < 1e-9 {
            continue;
        }

        let distance = -origin.z / direction.z;
        let (x, y) = (origin.x + direction.x * distance, origin.y + direction.y * distance);

        if distance > 1e-5 && x.abs() <= 0.5 && y.abs() <= 0.5 {
            consider(distance, sprite.object);
        }
    }

    best.map(|(_, object)| object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{camera_from_world, RenderMesh, RenderSprite};
    use flux_engine::components::{PrimitiveMesh, Projection, ViewCameraData};
    use flux_engine::identity::{IdController, IdRegistryModel};
    use glam::Mat4;

    fn uuid() -> AssetUuid {
        IdController.new_uuid(&mut IdRegistryModel::default())
    }

    fn camera() -> RenderCamera {
        camera_from_world(None, Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)), &ViewCameraData::default(), 1.0)
    }

    fn mesh(kind: PrimitiveMesh, model: Mat4) -> RenderMesh {
        RenderMesh {
            object: uuid(),
            mesh: kind,
            model,
            color: [1.0; 4],
            unlit: true,
            cast_shadows: true,
            receive_shadows: true,
            texture: None,
            tiling: [1.0, 1.0],
            offset: [0.0, 0.0],
        }
    }

    fn frame(camera: RenderCamera, meshes: Vec<RenderMesh>, sprites: Vec<RenderSprite>) -> RenderFrame {
        RenderFrame { camera, meshes, sprites, lights: Vec::new(), selected: Vec::new(), textures: Default::default() }
    }

    #[test]
    fn the_ray_through_the_middle_of_the_picture_goes_straight_ahead() {
        let ray = ray_from_ndc(&camera(), 0.0, 0.0);

        assert!((ray.direction - Vec3::Z).length() < 1e-4);
        assert!(ray.origin.z > -5.0 && ray.origin.z < -4.0); // starts at the near plane
    }

    #[test]
    fn the_object_in_the_middle_is_picked_and_the_empty_corner_is_not() {
        let cube = mesh(PrimitiveMesh::Cube, Mat4::IDENTITY);
        let target = cube.object;
        let picture = frame(camera(), vec![cube], vec![]);

        assert_eq!(pick(&picture, 0.0, 0.0), Some(target));
        assert_eq!(pick(&picture, 0.95, 0.95), None);
    }

    #[test]
    fn the_nearest_of_two_objects_is_picked() {
        let near = mesh(PrimitiveMesh::Cube, Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0)));
        let far = mesh(PrimitiveMesh::Sphere, Mat4::from_translation(Vec3::new(0.0, 0.0, 2.0)));
        let wanted = near.object;

        assert_eq!(pick(&frame(camera(), vec![far.clone(), near.clone()], vec![]), 0.0, 0.0), Some(wanted));
        assert_eq!(pick(&frame(camera(), vec![near, far], vec![]), 0.0, 0.0), Some(wanted));
    }

    #[test]
    fn a_scaled_and_moved_object_is_picked_where_it_is() {
        let wide = mesh(PrimitiveMesh::Cube, Mat4::from_translation(Vec3::new(1.5, 0.0, 0.0)) * Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0)));
        let target = wide.object;
        let picture = frame(camera(), vec![wide], vec![]);
        let ndc_of = |point: Vec3| {
            let ndc = (picture.camera.projection * picture.camera.view).project_point3(point);

            (ndc.x, ndc.y)
        };
        let (right_x, right_y) = ndc_of(Vec3::new(2.3, 0.0, -0.5));
        let (left_x, left_y) = ndc_of(Vec3::new(-1.0, 0.0, 0.0));

        assert_eq!(pick(&picture, right_x, right_y), Some(target)); // inside the wide cube
        assert_eq!(pick(&picture, left_x, left_y), None); // left of it, nothing there
    }

    #[test]
    fn a_sprite_is_picked_inside_its_rectangle_and_a_mesh_in_front_wins() {
        let sprite = RenderSprite {
            object: uuid(),
            model: Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            color: [1.0; 4],
            flip_x: false,
            flip_y: false,
            order: 0,
            texture: None,
        };
        let sprite_id = sprite.object;
        let mut picture = frame(camera(), vec![], vec![sprite]);

        assert_eq!(pick(&picture, 0.0, 0.0), Some(sprite_id));

        let blocker = mesh(PrimitiveMesh::Cube, Mat4::from_translation(Vec3::new(0.0, 0.0, -2.0)));
        let blocker_id = blocker.object;

        picture.meshes = vec![blocker];
        assert_eq!(pick(&picture, 0.0, 0.0), Some(blocker_id));
    }

    #[test]
    fn something_behind_the_camera_is_not_picked() {
        let behind = mesh(PrimitiveMesh::Cube, Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0)));

        assert_eq!(pick(&frame(camera(), vec![behind], vec![]), 0.0, 0.0), None);
    }

    #[test]
    fn picking_works_with_an_orthographic_camera_too() {
        let lens = ViewCameraData { projection: Projection::Orthographic, ortho_size: 2.0, ..Default::default() };
        let camera = camera_from_world(None, Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)), &lens, 1.0);
        let cube = mesh(PrimitiveMesh::Cube, Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));
        let target = cube.object;
        let picture = frame(camera, vec![cube], vec![]);

        // The cube spans x from 0.5 to 1.5, which is 0.25 to 0.75 of the half width (2).
        assert_eq!(pick(&picture, 0.5, 0.0), Some(target));
        assert_eq!(pick(&picture, -0.5, 0.0), None);
    }
}
