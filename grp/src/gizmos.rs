//! Outlines of things that are not drawn but should be seen in the editor, as lines.
//! Colliders for now: a box, a ball (three circles), a cylinder, a capsule and a prism.

use crate::frame::RenderLine;
use flux_engine::components::ColliderShape;
use flux_engine::physics::WorldCollider;
use glam::Vec3;
use std::f32::consts::PI;

/// Round shapes are drawn with this many straight pieces per circle.
const SEGMENTS: usize = 24;

/// Collects the lines of one collider. Points are given in the collider's own space (the middle is the
/// origin) and turned into the world when a line is added.
struct Outline<'a> {
    collider: &'a WorldCollider,
    color: [f32; 4],
    lines: Vec<RenderLine>,
}

impl Outline<'_> {
    fn world(&self, local: Vec3) -> Vec3 {
        self.collider.rotation * local + self.collider.position
    }

    fn segment(&mut self, from: Vec3, to: Vec3) {
        let line = RenderLine { from: self.world(from), to: self.world(to), color: self.color };

        self.lines.push(line);
    }

    /// Points on a circle (or part of one) in the plane spanned by `u` and `v`. `count + 1` points.
    fn arc(&mut self, center: Vec3, radius: f32, (u, v): (Vec3, Vec3), from: f32, to: f32, count: usize) {
        let point = |step: usize| {
            let angle = from + (to - from) * step as f32 / count as f32;

            center + (u * angle.cos() + v * angle.sin()) * radius
        };

        for step in 0..count {
            self.segment(point(step), point(step + 1));
        }
    }

    fn circle(&mut self, center: Vec3, radius: f32, plane: (Vec3, Vec3)) {
        self.arc(center, radius, plane, 0.0, 2.0 * PI, SEGMENTS);
    }
}

/// The lines that show a collider.
pub fn collider_lines(collider: &WorldCollider, color: [f32; 4]) -> Vec<RenderLine> {
    let mut outline = Outline { collider, color, lines: Vec::new() };
    let (x, y, z) = (Vec3::X, Vec3::Y, Vec3::Z);

    match collider.shape {
        ColliderShape::Box => {
            let half = collider.size * 0.5;
            let corner = |index: usize| {
                Vec3::new(
                    if index & 1 == 0 { -half.x } else { half.x },
                    if index & 2 == 0 { -half.y } else { half.y },
                    if index & 4 == 0 { -half.z } else { half.z },
                )
            };

            // Every pair of corners that differ in exactly one axis is an edge: 12 of them.
            for index in 0..8 {
                for axis in 0..3 {
                    let other = index ^ (1 << axis);

                    if index < other {
                        outline.segment(corner(index), corner(other));
                    }
                }
            }
        }
        ColliderShape::Sphere => {
            outline.circle(Vec3::ZERO, collider.radius, (x, y));
            outline.circle(Vec3::ZERO, collider.radius, (x, z));
            outline.circle(Vec3::ZERO, collider.radius, (y, z));
        }
        ColliderShape::Cylinder | ColliderShape::Capsule => {
            let half = collider.half_straight();
            let radius = collider.radius;
            let (top, bottom) = (Vec3::new(0.0, half, 0.0), Vec3::new(0.0, -half, 0.0));

            outline.circle(top, radius, (x, z));
            outline.circle(bottom, radius, (x, z));

            for side in [x, -x, z, -z] {
                outline.segment(top + side * radius, bottom + side * radius);
            }

            if collider.shape == ColliderShape::Capsule {
                // A half circle on each end, seen from the front and from the side.
                for plane in [(x, y), (z, y)] {
                    outline.arc(top, radius, plane, 0.0, PI, SEGMENTS / 2);
                    outline.arc(bottom, radius, plane, PI, 2.0 * PI, SEGMENTS / 2);
                }
            }
        }
        ColliderShape::Prism => {
            let half = collider.height * 0.5;
            let sides = collider.sides.max(3) as usize;
            let corner = |index: usize, height: f32| {
                let angle = 2.0 * PI * (index % sides) as f32 / sides as f32;

                Vec3::new(collider.radius * angle.cos(), height, collider.radius * angle.sin())
            };

            for index in 0..sides {
                outline.segment(corner(index, half), corner(index + 1, half));
                outline.segment(corner(index, -half), corner(index + 1, -half));
                outline.segment(corner(index, half), corner(index, -half));
            }
        }
    }

    outline.lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_engine::identity::{IdController, IdRegistryModel};
    use glam::Quat;

    fn collider(shape: ColliderShape) -> WorldCollider {
        WorldCollider {
            object: IdController.new_uuid(&mut IdRegistryModel::default()),
            shape,
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            size: Vec3::new(2.0, 4.0, 6.0),
            radius: 0.5,
            height: 3.0,
            sides: 6,
            trigger: false,
            friction: 0.5,
            bounciness: 0.0,
        }
    }

    fn lines(shape: ColliderShape) -> Vec<RenderLine> {
        collider_lines(&collider(shape), [0.0, 1.0, 0.0, 1.0])
    }

    #[test]
    fn each_shape_has_the_expected_number_of_lines() {
        assert_eq!(lines(ColliderShape::Box).len(), 12);
        assert_eq!(lines(ColliderShape::Sphere).len(), 3 * SEGMENTS);
        assert_eq!(lines(ColliderShape::Cylinder).len(), 2 * SEGMENTS + 4);
        assert_eq!(lines(ColliderShape::Capsule).len(), 2 * SEGMENTS + 4 + 4 * (SEGMENTS / 2));
        assert_eq!(lines(ColliderShape::Prism).len(), 18);
    }

    #[test]
    fn a_box_is_outlined_at_its_size_and_position() {
        let lines = lines(ColliderShape::Box);
        let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));

        for line in &lines {
            for point in [line.from, line.to] {
                low = low.min(point);
                high = high.max(point);
            }
        }

        assert!((low - Vec3::new(0.0, 0.0, 0.0)).length() < 1e-5); // 1,2,3 minus half of 2,4,6
        assert!((high - Vec3::new(2.0, 4.0, 6.0)).length() < 1e-5);
    }

    #[test]
    fn round_shapes_stay_on_their_surface() {
        // Every point of a sphere outline is `radius` from the middle.
        for line in lines(ColliderShape::Sphere) {
            assert!(((line.from - Vec3::new(1.0, 2.0, 3.0)).length() - 0.5).abs() < 1e-4);
        }

        // A capsule of height 3 and radius 0.5 reaches 1.5 above and below its middle.
        let capsule = lines(ColliderShape::Capsule);
        let top = capsule.iter().flat_map(|line| [line.from.y, line.to.y]).fold(f32::MIN, f32::max);
        let bottom = capsule.iter().flat_map(|line| [line.from.y, line.to.y]).fold(f32::MAX, f32::min);

        assert!((top - 3.5).abs() < 1e-4 && (bottom - 0.5).abs() < 1e-4);
    }

    #[test]
    fn the_outline_turns_with_the_collider_and_keeps_its_color() {
        let mut turned = collider(ColliderShape::Box);

        turned.rotation = Quat::from_rotation_z(PI / 2.0);
        turned.position = Vec3::ZERO;

        let lines = collider_lines(&turned, [1.0, 0.0, 0.0, 1.0]);
        let tallest = lines.iter().flat_map(|line| [line.from.y, line.to.y]).fold(f32::MIN, f32::max);

        // The box is 2 wide, 4 tall, 6 deep. Turned a quarter around Z, the 2 side becomes the height: 1 above the middle.
        assert!((tallest - 1.0).abs() < 1e-4);
        assert!(lines.iter().all(|line| line.color == [1.0, 0.0, 0.0, 1.0]));
    }
}
