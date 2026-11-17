//! Drawing triangles into a picture: clipping, depth test, and perspective-correct interpolation.

use glam::{Vec3, Vec4};

/// A corner of a triangle, after the camera has been applied.
#[derive(Clone, Copy)]
pub struct ClipVertex {
    /// Position in clip space (before the perspective divide).
    pub clip: Vec4,
    pub world: Vec3,
    pub normal: Vec3,
}

/// The picture being drawn: a color and a depth value for every pixel.
pub struct Target {
    pub width: usize,
    pub height: usize,
    /// Red, green, blue, alpha from 0 to 1.
    pub color: Vec<[f32; 4]>,
    /// 0 is at the near plane, 1 at the far plane. Infinity means nothing drawn yet.
    pub depth: Vec<f32>,
}

impl Target {
    pub fn new(width: usize, height: usize, clear: [f32; 4]) -> Self {
        Self { width, height, color: vec![clear; width * height], depth: vec![f32::INFINITY; width * height] }
    }
}

/// What the shader gets for one pixel of a triangle.
pub struct Fragment {
    pub world: Vec3,
    pub normal: Vec3,
}

#[derive(Clone, Copy)]
pub struct DrawMode {
    /// Remember the depth of what was drawn (meshes do, sprites don't).
    pub depth_write: bool,
    /// Draw where the depth is equal too (sprites at the same depth, drawn in order).
    pub depth_equal_ok: bool,
}

fn lerp(a: ClipVertex, b: ClipVertex, t: f32) -> ClipVertex {
    ClipVertex {
        clip: a.clip + (b.clip - a.clip) * t,
        world: a.world + (b.world - a.world) * t,
        normal: a.normal + (b.normal - a.normal) * t,
    }
}

/// Cuts off the part of a triangle that is in front of the near plane (or behind the camera).
/// Returns a polygon with 0, 3 or 4 corners.
fn clip_near(triangle: [ClipVertex; 3]) -> Vec<ClipVertex> {
    let mut polygon = Vec::with_capacity(4);

    for index in 0..3 {
        let current = triangle[index];
        let next = triangle[(index + 1) % 3];
        let current_inside = current.clip.z >= 0.0;
        let next_inside = next.clip.z >= 0.0;

        if current_inside {
            polygon.push(current);
        }

        if current_inside != next_inside {
            let t = current.clip.z / (current.clip.z - next.clip.z);

            polygon.push(lerp(current, next, t));
        }
    }

    polygon
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

/// Draws one triangle. `shade` gives the color (with alpha) of a pixel, or None to leave it.
pub fn draw_triangle(
    target: &mut Target,
    triangle: [ClipVertex; 3],
    mode: DrawMode,
    shade: &mut dyn FnMut(&Fragment) -> Option<[f32; 4]>,
) {
    let polygon = clip_near(triangle);

    for index in 1..polygon.len().saturating_sub(1) {
        fill(target, [polygon[0], polygon[index], polygon[index + 1]], mode, shade);
    }
}

fn fill(target: &mut Target, vertices: [ClipVertex; 3], mode: DrawMode, shade: &mut dyn FnMut(&Fragment) -> Option<[f32; 4]>) {
    let (width, height) = (target.width, target.height);
    let mut screen = [Vec3::ZERO; 3];
    let mut inverse_w = [0.0_f32; 3];

    for (index, vertex) in vertices.iter().enumerate() {
        if vertex.clip.w <= 1e-9 {
            return;
        }

        let ndc = vertex.clip.truncate() / vertex.clip.w;

        // y is flipped: the top of the picture is y = 0.
        screen[index] = Vec3::new((ndc.x * 0.5 + 0.5) * width as f32, (1.0 - (ndc.y * 0.5 + 0.5)) * height as f32, ndc.z);
        inverse_w[index] = 1.0 / vertex.clip.w;
    }

    let area = edge(screen[0].x, screen[0].y, screen[1].x, screen[1].y, screen[2].x, screen[2].y);

    if area.abs() < 1e-12 {
        return;
    }

    let min_x = screen.iter().map(|s| s.x).fold(f32::INFINITY, f32::min).floor().max(0.0) as usize;
    let max_x = screen.iter().map(|s| s.x).fold(f32::NEG_INFINITY, f32::max).ceil().min(width as f32 - 1.0);
    let min_y = screen.iter().map(|s| s.y).fold(f32::INFINITY, f32::min).floor().max(0.0) as usize;
    let max_y = screen.iter().map(|s| s.y).fold(f32::NEG_INFINITY, f32::max).ceil().min(height as f32 - 1.0);

    if max_x < 0.0 || max_y < 0.0 {
        return;
    }

    for y in min_y..=(max_y as usize) {
        for x in min_x..=(max_x as usize) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);

            // How much of each corner this pixel is made of (they add up to 1 inside the triangle).
            let w0 = edge(screen[1].x, screen[1].y, screen[2].x, screen[2].y, px, py) / area;
            let w1 = edge(screen[2].x, screen[2].y, screen[0].x, screen[0].y, px, py) / area;
            let w2 = edge(screen[0].x, screen[0].y, screen[1].x, screen[1].y, px, py) / area;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }

            let depth = w0 * screen[0].z + w1 * screen[1].z + w2 * screen[2].z;

            if !(0.0..=1.0).contains(&depth) {
                continue;
            }

            let index = y * width + x;
            let closer = if mode.depth_equal_ok { depth <= target.depth[index] } else { depth < target.depth[index] };

            if !closer {
                continue;
            }

            // Perspective-correct: things far away change more slowly across the screen.
            let (a0, a1, a2) = (w0 * inverse_w[0], w1 * inverse_w[1], w2 * inverse_w[2]);
            let sum = a0 + a1 + a2;

            if sum <= 0.0 {
                continue;
            }

            let fragment = Fragment {
                world: (vertices[0].world * a0 + vertices[1].world * a1 + vertices[2].world * a2) / sum,
                normal: (vertices[0].normal * a0 + vertices[1].normal * a1 + vertices[2].normal * a2) / sum,
            };

            if let Some(color) = shade(&fragment) {
                let behind = target.color[index];
                let alpha = color[3].clamp(0.0, 1.0);

                target.color[index] = [
                    color[0] * alpha + behind[0] * (1.0 - alpha),
                    color[1] * alpha + behind[1] * (1.0 - alpha),
                    color[2] * alpha + behind[2] * (1.0 - alpha),
                    1.0,
                ];

                if mode.depth_write {
                    target.depth[index] = depth;
                }
            }
        }
    }
}
