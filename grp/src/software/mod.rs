//! A renderer that runs on the CPU. It draws meshes (with lighting and a depth buffer) and sprites
//! into an image. Slow compared to a GPU but it works everywhere, and its pictures can be checked in tests.
//!
//! Meshes: back faces are not drawn, the nearest surface wins, lit meshes get ambient light plus
//! every light (no shadows yet). Sprites are drawn after the meshes in order, flat colored, with a
//! depth test against the meshes. Lit colors are used as they are (no gamma correction).

pub mod mesh;
pub mod raster;
pub mod shade;
pub mod texture;

use crate::backend::RenderBackend;
use crate::frame::{RenderCamera, RenderFrame, RenderMesh, RenderSprite, TextureImage};
use glam::{Mat3, Mat4, Vec2, Vec3};
use mesh::primitive;
use raster::{draw_triangle, ClipVertex, DrawMode, Target};
use shade::lit_color;
use texture::sample;

/// A finished picture: red, green, blue, alpha bytes, row by row from the top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let start = (y * self.width + x) * 4;

        [self.rgba[start], self.rgba[start + 1], self.rgba[start + 2], self.rgba[start + 3]]
    }
}

fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// The renderer that draws on the CPU.
#[derive(Default)]
pub struct SoftwareRenderer;

impl RenderBackend for SoftwareRenderer {
    fn render(&mut self, frame: &RenderFrame, width: usize, height: usize) -> Image {
        render(frame, width, height)
    }
}

/// Draws a frame into a picture of the given size.
pub fn render(frame: &RenderFrame, width: usize, height: usize) -> Image {
    let width = width.max(1);
    let height = height.max(1);
    let mut target = Target::new(width, height, frame.camera.clear_color);
    let view_projection = frame.camera.projection * frame.camera.view;
    let forward = frame.camera.view.inverse().transform_vector3(Vec3::Z);

    for mesh in &frame.meshes {
        draw_mesh(&mut target, frame, view_projection, forward, mesh, frame.selected.contains(&mesh.object));
    }

    for sprite in &frame.sprites {
        let texture = sprite.texture.and_then(|id| frame.textures.get(&id)).map(|texture| texture.as_ref());

        draw_sprite(&mut target, view_projection, sprite, frame.selected.contains(&sprite.object), texture);
    }

    let mut rgba = Vec::with_capacity(width * height * 4);

    for pixel in &target.color {
        rgba.extend([to_byte(pixel[0]), to_byte(pixel[1]), to_byte(pixel[2]), 255]);
    }

    Image { width, height, rgba }
}

/// An orthographic camera looks the same direction everywhere, a perspective one looks at each point.
fn is_orthographic(camera: &RenderCamera) -> bool {
    camera.projection.w_axis.w > 0.5
}

/// The color of something that is selected: mixed 35% toward orange.
fn highlight(color: [f32; 4]) -> [f32; 4] {
    let orange = [1.0, 0.6, 0.1];
    let mix = |value: f32, target: f32| value * 0.65 + target * 0.35;

    [mix(color[0], orange[0]), mix(color[1], orange[1]), mix(color[2], orange[2]), color[3]]
}

fn draw_mesh(target: &mut Target, frame: &RenderFrame, view_projection: Mat4, forward: Vec3, mesh: &RenderMesh, selected: bool) {
    let base_color = if selected { highlight(mesh.color) } else { mesh.color };
    let shape = primitive(mesh.mesh);
    let normal_matrix = Mat3::from_mat4(mesh.model);

    // A mesh scaled to nothing has no surface.
    if normal_matrix.determinant().abs() < 1e-12 {
        return;
    }

    let normal_matrix = normal_matrix.inverse().transpose();
    let texture = mesh.texture.and_then(|id| frame.textures.get(&id));
    let vertices: Vec<ClipVertex> = shape
        .positions
        .iter()
        .zip(&shape.normals)
        .zip(&shape.uvs)
        .map(|((position, normal), uv)| {
            let world = mesh.model.transform_point3(*position);

            ClipVertex {
                clip: view_projection * world.extend(1.0),
                world,
                normal: (normal_matrix * *normal).normalize_or_zero(),
                uv: *uv,
            }
        })
        .collect();
    let orthographic = is_orthographic(&frame.camera);
    let mode = DrawMode { depth_write: true, depth_equal_ok: false };

    for corners in shape.indices.chunks_exact(3) {
        let triangle = [vertices[corners[0] as usize], vertices[corners[1] as usize], vertices[corners[2] as usize]];

        // Skip triangles that face away from the camera.
        let normal = triangle[0].normal + triangle[1].normal + triangle[2].normal;
        let center = (triangle[0].world + triangle[1].world + triangle[2].world) / 3.0;
        let to_camera = if orthographic { -forward } else { frame.camera.position - center };

        if normal.dot(to_camera) <= 0.0 {
            continue;
        }

        draw_triangle(target, triangle, mode, &mut |fragment| {
            // The texture (if any) is multiplied with the color. Its alpha is not used on meshes.
            let tint = match texture {
                Some(texture) => sample(
                    texture,
                    Vec2::new(fragment.uv.x * mesh.tiling[0] + mesh.offset[0], fragment.uv.y * mesh.tiling[1] + mesh.offset[1]),
                ),
                None => [1.0; 4],
            };
            let base = [base_color[0] * tint[0], base_color[1] * tint[1], base_color[2] * tint[2], 1.0];

            if mesh.unlit {
                return Some(base);
            }

            let color = lit_color(base, fragment.world, fragment.normal.normalize_or_zero(), &frame.lights);

            Some([color[0], color[1], color[2], 1.0])
        });
    }
}

fn draw_sprite(target: &mut Target, view_projection: Mat4, sprite: &RenderSprite, selected: bool, texture: Option<&TextureImage>) {
    let color = if selected { highlight(sprite.color) } else { sprite.color };
    let corners = [Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, 0.0), Vec3::new(-0.5, 0.5, 0.0)];
    let vertices: Vec<ClipVertex> = corners
        .iter()
        .map(|corner| {
            let world = sprite.model.transform_point3(*corner);
            // The bottom left corner is (0, 0) of the picture, the top right (1, 1).
            let (mut u, mut v) = (corner.x + 0.5, corner.y + 0.5);

            if sprite.flip_x {
                u = 1.0 - u;
            }

            if sprite.flip_y {
                v = 1.0 - v;
            }

            ClipVertex { clip: view_projection * world.extend(1.0), world, normal: Vec3::NEG_Z, uv: Vec2::new(u, v) }
        })
        .collect();
    let mode = DrawMode { depth_write: false, depth_equal_ok: true };

    for triangle in [[0, 1, 2], [0, 2, 3]] {
        let corners = [vertices[triangle[0]], vertices[triangle[1]], vertices[triangle[2]]];

        draw_triangle(target, corners, mode, &mut |fragment| {
            let tint = texture.map_or([1.0; 4], |texture| sample(texture, fragment.uv));

            Some([color[0] * tint[0], color[1] * tint[1], color[2] * tint[2], color[3] * tint[3]])
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{camera_from_world, RenderLight};
    use flux_engine::components::{LightKind, PrimitiveMesh, Projection, ViewCameraData};
    use flux_engine::identity::{AssetUuid, IdController, IdRegistryModel};
    use glam::Quat;
    use std::f32::consts::FRAC_PI_2;

    fn uuid() -> AssetUuid {
        IdController.new_uuid(&mut IdRegistryModel::default())
    }

    /// A camera at (0, 0, -5) looking at the middle.
    fn camera() -> RenderCamera {
        camera_from_world(None, Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)), &ViewCameraData::default(), 1.0)
    }

    fn mesh_at(kind: PrimitiveMesh, position: Vec3, color: [f32; 4], unlit: bool) -> RenderMesh {
        RenderMesh {
            object: uuid(),
            mesh: kind,
            model: Mat4::from_translation(position),
            color,
            unlit,
            cast_shadows: true,
            receive_shadows: true,
            texture: None,
            tiling: [1.0, 1.0],
            offset: [0.0, 0.0],
        }
    }

    fn sun(direction: Vec3) -> RenderLight {
        RenderLight {
            object: uuid(),
            kind: LightKind::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            position: Vec3::ZERO,
            direction,
            range: 10.0,
            spot_angle_degrees: 45.0,
            shadows: false,
        }
    }

    fn frame(camera: RenderCamera, meshes: Vec<RenderMesh>, lights: Vec<RenderLight>) -> RenderFrame {
        RenderFrame { camera, meshes, sprites: Vec::new(), lights, selected: Vec::new(), textures: Default::default() }
    }

    fn near(actual: [u8; 4], expected: [u8; 3]) -> bool {
        (0..3).all(|channel| (actual[channel] as i32 - expected[channel] as i32).abs() <= 2)
    }

    #[test]
    fn an_empty_scene_is_the_clear_color_and_the_picture_has_the_right_size() {
        let image = render(&frame(camera(), vec![], vec![]), 16, 8);

        assert_eq!((image.width, image.height, image.rgba.len()), (16, 8, 16 * 8 * 4));
        assert!((0..8).all(|y| (0..16).all(|x| image.pixel(x, y) == [26, 31, 41, 255])));
    }

    #[test]
    fn a_cube_is_drawn_in_the_middle_and_the_corners_stay_clear() {
        let cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true);
        let image = render(&frame(camera(), vec![cube], vec![]), 64, 64);

        assert_eq!(image.pixel(32, 32), [255, 255, 255, 255]);
        assert_eq!(image.pixel(0, 0), [26, 31, 41, 255]);
        assert_eq!(image.pixel(63, 63), [26, 31, 41, 255]);
    }

    #[test]
    fn the_nearer_object_hides_the_farther_one_whatever_the_drawing_order() {
        let near_cube = mesh_at(PrimitiveMesh::Cube, Vec3::new(0.0, 0.0, -1.0), [1.0, 0.0, 0.0, 1.0], true);
        let far_cube = mesh_at(PrimitiveMesh::Cube, Vec3::new(0.0, 0.0, 2.0), [0.0, 1.0, 0.0, 1.0], true);

        for meshes in [vec![near_cube.clone(), far_cube.clone()], vec![far_cube, near_cube]] {
            let image = render(&frame(camera(), meshes, vec![]), 64, 64);

            assert_eq!(image.pixel(32, 32), [255, 0, 0, 255]);
        }
    }

    #[test]
    fn a_lit_face_toward_the_light_is_bright_and_one_facing_away_only_has_ambient_light() {
        let cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [0.5, 0.25, 0.25, 1.0], false);

        // The camera looks along +Z, so the face it sees points toward -Z.
        let lit = render(&frame(camera(), vec![cube.clone()], vec![sun(Vec3::Z)]), 64, 64);
        let dark = render(&frame(camera(), vec![cube], vec![sun(Vec3::NEG_Z)]), 64, 64);

        assert!(near(lit.pixel(32, 32), [147, 73, 73])); // base * (0.15 + 1.0)
        assert!(near(dark.pixel(32, 32), [19, 10, 10])); // base * 0.15
    }

    #[test]
    fn an_unlit_mesh_ignores_the_lights() {
        let cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [0.5, 0.25, 0.25, 1.0], true);
        let image = render(&frame(camera(), vec![cube], vec![sun(Vec3::Z)]), 64, 64);

        assert!(near(image.pixel(32, 32), [128, 64, 64]));
    }

    #[test]
    fn the_faces_pointing_away_from_the_camera_are_not_drawn() {
        // Seen from inside a cube, none of its faces face the camera, so nothing is drawn.
        let cube = RenderMesh {
            model: Mat4::from_scale(Vec3::splat(20.0)),
            ..mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true)
        };
        let image = render(&frame(camera(), vec![cube], vec![]), 32, 32);

        assert_eq!(image.pixel(16, 16), [26, 31, 41, 255]);
    }

    #[test]
    fn a_point_light_makes_the_ground_brighter_right_under_it() {
        // A camera 5 above the ground, looking straight down.
        let looking_down = Mat4::from_rotation_translation(Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.0, 5.0, 0.0));
        let camera = camera_from_world(None, looking_down, &ViewCameraData::default(), 1.0);
        let ground = mesh_at(PrimitiveMesh::Plane, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], false);
        let lamp = RenderLight { kind: LightKind::Point, position: Vec3::new(0.0, 1.0, 0.0), ..sun(Vec3::Z) };
        let image = render(&frame(camera, vec![ground], vec![lamp]), 64, 64);

        assert!(image.pixel(32, 32)[0] > image.pixel(58, 32)[0]);
        assert!(image.pixel(58, 32)[0] > 38); // more than the ambient light alone (0.15 * 255)
    }

    #[test]
    fn a_surface_that_runs_through_the_camera_is_cut_at_the_near_plane() {
        let camera = camera_from_world(None, Mat4::IDENTITY, &ViewCameraData::default(), 1.0);
        let ground = RenderMesh {
            model: Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
            ..mesh_at(PrimitiveMesh::Plane, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true)
        };
        let image = render(&frame(camera, vec![ground], vec![]), 64, 64);

        // The ground fills the lower part of the picture, the sky the top.
        assert_eq!(image.pixel(32, 63), [255, 255, 255, 255]);
        assert_eq!(image.pixel(32, 0), [26, 31, 41, 255]);
    }

    #[test]
    fn a_sphere_a_cylinder_and_a_capsule_all_show_up() {
        for kind in [PrimitiveMesh::Sphere, PrimitiveMesh::Cylinder, PrimitiveMesh::Capsule] {
            let shape = mesh_at(kind, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true);
            let image = render(&frame(camera(), vec![shape], vec![]), 64, 64);

            assert_eq!(image.pixel(32, 32), [255, 255, 255, 255], "{kind:?} is missing");
        }
    }

    #[test]
    fn a_sprite_with_a_higher_order_is_drawn_on_top_and_sprites_respect_meshes_in_front() {
        let camera = camera_from_world(
            None,
            Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)),
            &ViewCameraData { projection: Projection::Orthographic, ortho_size: 2.0, ..Default::default() },
            1.0,
        );
        let sprite = |order: i32, color: [f32; 4]| RenderSprite {
            object: uuid(),
            model: Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            color,
            flip_x: false,
            flip_y: false,
            order,
            texture: None,
        };
        let mut picture = frame(camera.clone(), vec![], vec![]);

        picture.sprites = vec![sprite(0, [1.0, 0.0, 0.0, 1.0]), sprite(1, [0.0, 1.0, 0.0, 1.0])];

        assert_eq!(render(&picture, 32, 32).pixel(16, 16), [0, 255, 0, 255]);

        // A cube in front of the sprites hides them.
        picture.meshes = vec![mesh_at(PrimitiveMesh::Cube, Vec3::new(0.0, 0.0, -2.0), [0.0, 0.0, 1.0, 1.0], true)];

        assert_eq!(render(&picture, 32, 32).pixel(16, 16), [0, 0, 255, 255]);
    }

    #[test]
    fn the_backend_trait_draws_the_same_picture() {
        let cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true);
        let picture = frame(camera(), vec![cube], vec![]);

        assert_eq!(SoftwareRenderer.render(&picture, 16, 16), render(&picture, 16, 16));
    }

    #[test]
    fn a_selected_object_is_drawn_mixed_toward_orange() {
        let cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true);
        let mut picture = frame(camera(), vec![cube.clone()], vec![]);

        assert_eq!(render(&picture, 32, 32).pixel(16, 16), [255, 255, 255, 255]);

        picture.selected = vec![cube.object];

        // white * 0.65 + orange (1.0, 0.6, 0.1) * 0.35
        assert!(near(render(&picture, 32, 32).pixel(16, 16), [255, 219, 175]));
    }

    fn picture_texture(width: usize, height: usize, rgba: Vec<u8>) -> std::sync::Arc<TextureImage> {
        std::sync::Arc::new(TextureImage { width, height, rgba, nearest: true })
    }

    #[test]
    fn a_textured_sprite_shows_the_corners_of_its_picture_in_the_right_places() {
        let camera = camera_from_world(
            None,
            Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)),
            &ViewCameraData { projection: Projection::Orthographic, ortho_size: 2.0, ..Default::default() },
            1.0,
        );
        let id = uuid();
        let sprite = RenderSprite {
            object: uuid(),
            model: Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            color: [1.0; 4],
            flip_x: false,
            flip_y: false,
            order: 0,
            texture: Some(id),
        };
        let mut picture = frame(camera, vec![], vec![]);

        // red, green / blue, white
        picture.textures.insert(
            id,
            picture_texture(2, 2, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255]),
        );
        picture.sprites = vec![sprite.clone()];

        let image = render(&picture, 32, 32);

        assert_eq!(image.pixel(12, 12), [255, 0, 0, 255]);
        assert_eq!(image.pixel(20, 12), [0, 255, 0, 255]);
        assert_eq!(image.pixel(12, 20), [0, 0, 255, 255]);
        assert_eq!(image.pixel(20, 20), [255, 255, 255, 255]);

        // Flipped left to right.
        picture.sprites = vec![RenderSprite { flip_x: true, ..sprite }];

        let flipped = render(&picture, 32, 32);

        assert_eq!(flipped.pixel(12, 12), [0, 255, 0, 255]);
        assert_eq!(flipped.pixel(20, 12), [255, 0, 0, 255]);
    }

    #[test]
    fn a_see_through_texel_shows_what_is_behind_the_sprite() {
        let camera = camera_from_world(
            None,
            Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0)),
            &ViewCameraData { projection: Projection::Orthographic, ortho_size: 2.0, ..Default::default() },
            1.0,
        );
        let id = uuid();
        let mut picture = frame(camera, vec![], vec![]);

        picture.textures.insert(id, picture_texture(1, 1, vec![255, 255, 255, 0]));
        picture.sprites = vec![RenderSprite {
            object: uuid(),
            model: Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            color: [1.0; 4],
            flip_x: false,
            flip_y: false,
            order: 0,
            texture: Some(id),
        }];

        assert_eq!(render(&picture, 32, 32).pixel(16, 16), [26, 31, 41, 255]);
    }

    #[test]
    fn a_texture_on_a_mesh_is_multiplied_with_its_color_and_a_missing_texture_is_ignored() {
        let id = uuid();
        let mut cube = mesh_at(PrimitiveMesh::Cube, Vec3::ZERO, [1.0, 1.0, 1.0, 1.0], true);

        cube.texture = Some(id);

        let mut picture = frame(camera(), vec![cube.clone()], vec![]);

        // The texture is not loaded: the cube is just white.
        assert_eq!(render(&picture, 64, 64).pixel(32, 32), [255, 255, 255, 255]);

        picture.textures.insert(id, picture_texture(1, 1, vec![0, 0, 255, 255]));
        assert_eq!(render(&picture, 64, 64).pixel(32, 32), [0, 0, 255, 255]);

        // A red color times a blue texture is black.
        picture.meshes = vec![RenderMesh { color: [1.0, 0.0, 0.0, 1.0], ..cube }];
        assert_eq!(render(&picture, 64, 64).pixel(32, 32), [0, 0, 0, 255]);
    }
}
