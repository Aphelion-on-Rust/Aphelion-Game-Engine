//! The built-in shapes as triangles. Normals point outward.
//! Sizes: Cube 1 x 1 x 1, Sphere diameter 1, Plane 10 x 10 (facing up), Cylinder radius 0.5 and
//! height 2, Capsule radius 0.5 and height 2. All are centered on the origin.

use flux_engine::components::PrimitiveMesh;
use glam::{Vec2, Vec3};
use std::f32::consts::PI;
use std::sync::OnceLock;

pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Where each point is on the texture, from (0, 0) at the bottom left to (1, 1) at the top right.
    pub uvs: Vec<Vec2>,
    /// Three indices per triangle. The winding does not matter, culling uses the normals.
    pub indices: Vec<u32>,
}

impl Mesh {
    fn empty() -> Self {
        Self { positions: Vec::new(), normals: Vec::new(), uvs: Vec::new(), indices: Vec::new() }
    }

    /// Adds four corners and the two triangles between them.
    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3) {
        let first = self.positions.len() as u32;

        for corner in corners {
            self.positions.push(corner);
            self.normals.push(normal);
        }

        self.uvs.extend([Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)]);

        self.indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }
}

fn cube() -> Mesh {
    let mut mesh = Mesh::empty();

    // (normal, two directions along the face)
    let faces = [
        (Vec3::X, Vec3::Y, Vec3::Z),
        (Vec3::NEG_X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::Z, Vec3::X),
        (Vec3::NEG_Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (Vec3::NEG_Z, Vec3::Y, Vec3::X),
    ];

    for (normal, u, v) in faces {
        let center = normal * 0.5;

        mesh.quad(
            [center - u * 0.5 - v * 0.5, center + u * 0.5 - v * 0.5, center + u * 0.5 + v * 0.5, center - u * 0.5 + v * 0.5],
            normal,
        );
    }

    mesh
}

fn plane() -> Mesh {
    let mut mesh = Mesh::empty();

    mesh.quad(
        [Vec3::new(-5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, 5.0), Vec3::new(-5.0, 0.0, 5.0)],
        Vec3::Y,
    );
    mesh
}

/// A surface of rotation around Y, built from rings. Each ring is `(angle from the top, height offset)`.
/// A sphere uses rings from 0 to PI. A capsule uses two half spheres moved apart.
fn lathe(rings: &[(f32, f32)], radius: f32, segments: usize) -> Mesh {
    let mut mesh = Mesh::empty();

    for (ring_index, &(theta, offset)) in rings.iter().enumerate() {
        for segment in 0..=segments {
            let phi = 2.0 * PI * segment as f32 / segments as f32;
            let normal = Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());

            mesh.positions.push(normal * radius + Vec3::new(0.0, offset, 0.0));
            mesh.normals.push(normal);
            mesh.uvs.push(Vec2::new(segment as f32 / segments as f32, ring_index as f32 / (rings.len() - 1) as f32));
        }
    }

    let stride = (segments + 1) as u32;

    for ring in 0..rings.len() as u32 - 1 {
        for segment in 0..segments as u32 {
            let a = ring * stride + segment;
            let b = a + 1;
            let c = a + stride;
            let d = c + 1;

            mesh.indices.extend([a, c, b, b, c, d]);
        }
    }

    mesh
}

fn sphere() -> Mesh {
    let rings: Vec<(f32, f32)> = (0..=16).map(|i| (PI * i as f32 / 16.0, 0.0)).collect();

    lathe(&rings, 0.5, 24)
}

fn capsule() -> Mesh {
    let mut rings: Vec<(f32, f32)> = (0..=8).map(|i| (0.5 * PI * i as f32 / 8.0, 0.5)).collect();

    rings.extend((0..=8).map(|i| (0.5 * PI + 0.5 * PI * i as f32 / 8.0, -0.5)));
    lathe(&rings, 0.5, 24)
}

fn cylinder() -> Mesh {
    let mut mesh = Mesh::empty();
    let segments = 24;
    let point = |segment: usize, y: f32| {
        let phi = 2.0 * PI * segment as f32 / segments as f32;

        (Vec3::new(0.5 * phi.cos(), y, 0.5 * phi.sin()), Vec3::new(phi.cos(), 0.0, phi.sin()))
    };

    // The side.
    for segment in 0..segments {
        let (bottom_a, normal_a) = point(segment, -1.0);
        let (top_a, _) = point(segment, 1.0);
        let (bottom_b, normal_b) = point(segment + 1, -1.0);
        let (top_b, _) = point(segment + 1, 1.0);
        let first = mesh.positions.len() as u32;

        mesh.positions.extend([bottom_a, bottom_b, top_b, top_a]);
        mesh.normals.extend([normal_a, normal_b, normal_b, normal_a]);

        let (left, right) = (segment as f32 / segments as f32, (segment + 1) as f32 / segments as f32);

        mesh.uvs.extend([Vec2::new(left, 0.0), Vec2::new(right, 0.0), Vec2::new(right, 1.0), Vec2::new(left, 1.0)]);
        mesh.indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    // The two caps, as fans around a center point.
    for (y, normal) in [(1.0, Vec3::Y), (-1.0, Vec3::NEG_Y)] {
        let center = mesh.positions.len() as u32;

        mesh.positions.push(Vec3::new(0.0, y, 0.0));
        mesh.normals.push(normal);
        mesh.uvs.push(Vec2::new(0.5, 0.5));

        for segment in 0..=segments {
            let edge = point(segment, y).0;

            mesh.positions.push(edge);
            mesh.normals.push(normal);
            mesh.uvs.push(Vec2::new(0.5 + edge.x, 0.5 + edge.z));
        }

        for segment in 0..segments as u32 {
            mesh.indices.extend([center, center + 1 + segment, center + 2 + segment]);
        }
    }

    mesh
}

/// The mesh of a built-in shape. They are made once.
pub fn primitive(kind: PrimitiveMesh) -> &'static Mesh {
    static MESHES: OnceLock<Vec<Mesh>> = OnceLock::new();

    let meshes = MESHES.get_or_init(|| vec![cube(), sphere(), plane(), cylinder(), capsule()]);

    match kind {
        PrimitiveMesh::Cube => &meshes[0],
        PrimitiveMesh::Sphere => &meshes[1],
        PrimitiveMesh::Plane => &meshes[2],
        PrimitiveMesh::Cylinder => &meshes[3],
        PrimitiveMesh::Capsule => &meshes[4],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [PrimitiveMesh; 5] =
        [PrimitiveMesh::Cube, PrimitiveMesh::Sphere, PrimitiveMesh::Plane, PrimitiveMesh::Cylinder, PrimitiveMesh::Capsule];

    #[test]
    fn every_shape_is_a_valid_triangle_list_with_unit_normals() {
        for kind in ALL {
            let mesh = primitive(kind);

            assert!(!mesh.indices.is_empty(), "{kind:?} has no triangles");
            assert_eq!(mesh.indices.len() % 3, 0);
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len(), mesh.uvs.len(), "{kind:?} has no uv for some points");
            assert!(mesh.uvs.iter().all(|uv| (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y)), "{kind:?} has a uv outside the texture");
            assert!(mesh.indices.iter().all(|&index| (index as usize) < mesh.positions.len()));
            assert!(mesh.normals.iter().all(|normal| (normal.length() - 1.0).abs() < 1e-4), "{kind:?} has a bad normal");
        }
    }

    #[test]
    fn the_shapes_have_the_documented_sizes() {
        let largest = |kind: PrimitiveMesh, f: fn(&Vec3) -> f32| primitive(kind).positions.iter().map(f).fold(0.0_f32, f32::max);

        assert!((largest(PrimitiveMesh::Cube, |p| p.x.abs().max(p.y.abs()).max(p.z.abs())) - 0.5).abs() < 1e-5);
        assert!((largest(PrimitiveMesh::Sphere, |p| p.length()) - 0.5).abs() < 1e-4);
        assert!((largest(PrimitiveMesh::Plane, |p| p.x.abs()) - 5.0).abs() < 1e-5);
        assert!((largest(PrimitiveMesh::Cylinder, |p| p.y.abs()) - 1.0).abs() < 1e-5);
        assert!((largest(PrimitiveMesh::Capsule, |p| p.y.abs()) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn the_normals_of_a_cube_point_away_from_its_center() {
        let mesh = primitive(PrimitiveMesh::Cube);

        for (position, normal) in mesh.positions.iter().zip(&mesh.normals) {
            assert!(position.dot(*normal) > 0.0);
        }
    }
}
