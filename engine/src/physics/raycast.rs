//! Shooting a ray at the colliders: what does it hit first?
//!
//! Each shape is tested in its own space (the ray is moved there with the collider's turn and position),
//! so the shapes are simple: centered on the origin, standing along the Y axis.
//! Only a ray that comes from OUTSIDE a shape hits it. A ray that starts inside a shape does not hit it.

use super::world::WorldCollider;
use crate::components::ColliderShape;
use crate::identity::AssetUuid;
use glam::Vec3;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub object: AssetUuid,
    /// How far along the ray.
    pub distance: f32,
    pub point: Vec3,
    /// Points away from the surface, length 1.
    pub normal: Vec3,
}

/// How far along the ray, and the surface normal.
type Hit = (f32, Vec3);

fn nearest(hits: Vec<Hit>) -> Option<Hit> {
    hits.into_iter().min_by(|a, b| a.0.total_cmp(&b.0))
}

/// A solid made of flat sides. Each side is (normal, offset): a point is inside if
/// `point . normal <= offset` for every side.
fn ray_convex(origin: Vec3, direction: Vec3, planes: &[(Vec3, f32)]) -> Option<Hit> {
    let mut enter = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    let mut normal = Vec3::ZERO;

    for &(side, offset) in planes {
        let toward = direction.dot(side);
        let outside_by = origin.dot(side) - offset;

        if toward.abs() < 1e-9 {
            // Going along this side: it only matters if the ray is outside it.
            if outside_by > 0.0 {
                return None;
            }

            continue;
        }

        let t = -outside_by / toward;

        if toward < 0.0 {
            if t > enter {
                enter = t;
                normal = side;
            }
        } else if t < exit {
            exit = t;
        }

        if enter > exit {
            return None;
        }
    }

    // Starting inside means entering "in the past".
    if enter < 0.0 || !enter.is_finite() {
        return None;
    }

    Some((enter, normal))
}

fn ray_box(origin: Vec3, direction: Vec3, size: Vec3) -> Option<Hit> {
    let half = size * 0.5;

    ray_convex(
        origin,
        direction,
        &[(Vec3::X, half.x), (Vec3::NEG_X, half.x), (Vec3::Y, half.y), (Vec3::NEG_Y, half.y), (Vec3::Z, half.z), (Vec3::NEG_Z, half.z)],
    )
}

/// A prism is a top, a bottom and one flat side for each side of the base polygon.
fn ray_prism(origin: Vec3, direction: Vec3, sides: u32, radius: f32, height: f32) -> Option<Hit> {
    let mut planes = vec![(Vec3::Y, height * 0.5), (Vec3::NEG_Y, height * 0.5)];
    let count = sides.max(3) as f32;

    // The corners of the base are at 2 PI k / n, so the flat sides face the angles in between.
    let apothem = radius * (PI / count).cos();

    for k in 0..sides.max(3) {
        let angle = 2.0 * PI * (k as f32 + 0.5) / count;

        planes.push((Vec3::new(angle.cos(), 0.0, angle.sin()), apothem));
    }

    ray_convex(origin, direction, &planes)
}

fn ray_sphere(origin: Vec3, direction: Vec3, radius: f32) -> Option<Hit> {
    let b = origin.dot(direction);
    let c = origin.dot(origin) - radius * radius;
    let discriminant = b * b - c;

    // c < 0 means the ray starts inside the ball.
    if discriminant < 0.0 || c < 0.0 {
        return None;
    }

    let t = -b - discriminant.sqrt();

    (t >= 0.0).then(|| (t, (origin + direction * t).normalize_or_zero()))
}

/// The curved side of a cylinder standing along Y, for the part with `|y| <= half`.
fn ray_cylinder_side(origin: Vec3, direction: Vec3, radius: f32, half: f32) -> Option<Hit> {
    let a = direction.x * direction.x + direction.z * direction.z;

    if a < 1e-12 {
        return None;
    }

    let b = origin.x * direction.x + origin.z * direction.z;
    let c = origin.x * origin.x + origin.z * origin.z - radius * radius;
    let discriminant = b * b - a * c;

    if discriminant < 0.0 {
        return None;
    }

    let t = (-b - discriminant.sqrt()) / a;
    let point = origin + direction * t;

    (t >= 0.0 && point.y.abs() <= half).then(|| (t, Vec3::new(point.x, 0.0, point.z).normalize_or_zero()))
}

fn ray_cylinder(origin: Vec3, direction: Vec3, radius: f32, half: f32) -> Option<Hit> {
    if origin.y.abs() <= half && origin.x * origin.x + origin.z * origin.z <= radius * radius {
        return None;
    }

    let mut hits: Vec<Hit> = ray_cylinder_side(origin, direction, radius, half).into_iter().collect();

    for sign in [-1.0_f32, 1.0] {
        if direction.y.abs() > 1e-9 {
            let t = (sign * half - origin.y) / direction.y;
            let point = origin + direction * t;

            if t >= 0.0 && point.x * point.x + point.z * point.z <= radius * radius {
                hits.push((t, Vec3::new(0.0, sign, 0.0)));
            }
        }
    }

    nearest(hits)
}

/// A capsule is a cylinder with a half ball on each end. `half` is half the straight part.
fn ray_capsule(origin: Vec3, direction: Vec3, radius: f32, half: f32) -> Option<Hit> {
    // Starting inside: closer to the middle line than the radius.
    let on_line = Vec3::new(0.0, origin.y.clamp(-half, half), 0.0);

    if (origin - on_line).length_squared() <= radius * radius {
        return None;
    }

    let mut hits: Vec<Hit> = ray_cylinder_side(origin, direction, radius, half).into_iter().collect();

    for sign in [-1.0_f32, 1.0] {
        let center = Vec3::new(0.0, sign * half, 0.0);

        if let Some((t, normal)) = ray_sphere(origin - center, direction, radius) {
            // Only the outer half of each ball counts.
            if (origin.y + direction.y * t - sign * half) * sign >= 0.0 {
                hits.push((t, normal));
            }
        }
    }

    nearest(hits)
}

/// The first collider the ray hits within `max_distance`. Triggers are skipped unless asked for.
/// `direction` does not have to be length 1.
pub fn raycast(
    colliders: &[WorldCollider],
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    include_triggers: bool,
) -> Option<RayHit> {
    let direction = direction.normalize_or_zero();

    if direction == Vec3::ZERO {
        return None;
    }

    let mut best: Option<RayHit> = None;

    for collider in colliders {
        if collider.trigger && !include_triggers {
            continue;
        }

        // Into the collider's own space: its middle is the origin and it is not turned.
        let turn_back = collider.rotation.inverse();
        let local_origin = turn_back * (origin - collider.position);
        let local_direction = turn_back * direction;

        let hit = match collider.shape {
            ColliderShape::Box => ray_box(local_origin, local_direction, collider.size),
            ColliderShape::Sphere => ray_sphere(local_origin, local_direction, collider.radius),
            ColliderShape::Cylinder => ray_cylinder(local_origin, local_direction, collider.radius, collider.height * 0.5),
            ColliderShape::Capsule => ray_capsule(local_origin, local_direction, collider.radius, collider.half_straight()),
            ColliderShape::Prism => ray_prism(local_origin, local_direction, collider.sides, collider.radius, collider.height),
        };

        if let Some((distance, normal)) = hit {
            if distance <= max_distance && best.map_or(true, |current| distance < current.distance) {
                best = Some(RayHit {
                    object: collider.object,
                    distance,
                    point: origin + direction * distance,
                    normal: (collider.rotation * normal).normalize_or_zero(),
                });
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdController, IdRegistryModel};
    use glam::Quat;

    const SHAPES: [ColliderShape; 5] =
        [ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule, ColliderShape::Cylinder, ColliderShape::Prism];

    fn collider(shape: ColliderShape) -> WorldCollider {
        WorldCollider {
            object: IdController.new_uuid(&mut IdRegistryModel::default()),
            shape,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            size: Vec3::ONE,
            radius: 0.5,
            height: 2.0,
            sides: 6,
            trigger: false,
            friction: 0.5,
            bounciness: 0.0,
        }
    }

    fn shoot(shape: ColliderShape, origin: Vec3, direction: Vec3) -> Option<RayHit> {
        raycast(&[collider(shape)], origin, direction, 100.0, false)
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn every_shape_is_hit_from_the_front_at_the_right_distance() {
        let expected = [
            (ColliderShape::Box, 4.5),
            (ColliderShape::Sphere, 4.5),
            (ColliderShape::Capsule, 4.5),
            (ColliderShape::Cylinder, 4.5),
            // A hexagon with corner radius 0.5 is 0.5 * cos(30 degrees) = 0.433 deep along this direction.
            (ColliderShape::Prism, 5.0 - 0.5 * (PI / 6.0).cos()),
        ];

        for (shape, distance) in expected {
            let hit = shoot(shape, Vec3::new(0.0, 0.0, -5.0), Vec3::Z).unwrap_or_else(|| panic!("{shape:?} was missed"));

            assert!(close(hit.distance, distance), "{shape:?}: {}", hit.distance);
            assert!(close(hit.point.z, -5.0 + distance));
            assert!(close(hit.normal.z, -1.0), "{shape:?} normal {:?}", hit.normal);
        }
    }

    #[test]
    fn every_shape_is_hit_from_above_on_its_top() {
        let tops = [
            (ColliderShape::Box, 4.5),
            (ColliderShape::Sphere, 4.5),
            (ColliderShape::Capsule, 4.0),
            (ColliderShape::Cylinder, 4.0),
            (ColliderShape::Prism, 4.0),
        ];

        for (shape, distance) in tops {
            let hit = shoot(shape, Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y).unwrap();

            assert!(close(hit.distance, distance), "{shape:?}: {}", hit.distance);
            assert!(close(hit.normal.y, 1.0), "{shape:?} normal {:?}", hit.normal);
        }
    }

    #[test]
    fn a_ray_that_passes_beside_a_shape_misses_it() {
        for shape in SHAPES {
            assert!(shoot(shape, Vec3::new(0.0, 3.0, -5.0), Vec3::Z).is_none(), "{shape:?} was hit");
            assert!(shoot(shape, Vec3::new(0.0, 0.0, -5.0), Vec3::NEG_Z).is_none(), "{shape:?} was hit from behind");
        }
    }

    #[test]
    fn a_ray_that_starts_inside_a_shape_does_not_hit_it() {
        for shape in SHAPES {
            assert!(shoot(shape, Vec3::new(0.0, 0.1, 0.0), Vec3::Z).is_none(), "{shape:?} was hit from inside");
        }
    }

    #[test]
    fn a_hit_beyond_the_maximum_distance_does_not_count() {
        let shapes = [collider(ColliderShape::Box)];

        assert!(raycast(&shapes, Vec3::new(0.0, 0.0, -5.0), Vec3::Z, 4.0, false).is_none());
        assert!(raycast(&shapes, Vec3::new(0.0, 0.0, -5.0), Vec3::Z, 5.0, false).is_some());
    }

    #[test]
    fn the_nearest_collider_wins_and_the_direction_does_not_need_length_one() {
        let mut near = collider(ColliderShape::Sphere);
        let far = collider(ColliderShape::Box);

        near.position = Vec3::new(0.0, 0.0, -2.0);

        let hit = raycast(&[far.clone(), near.clone()], Vec3::new(0.0, 0.0, -6.0), Vec3::new(0.0, 0.0, 7.0), 100.0, false).unwrap();

        assert_eq!(hit.object, near.object);
        assert!(close(hit.distance, 3.5));
        assert!(raycast(&[far], Vec3::new(0.0, 0.0, -6.0), Vec3::ZERO, 100.0, false).is_none());
    }

    #[test]
    fn triggers_are_ignored_unless_asked_for() {
        let mut area = collider(ColliderShape::Box);

        area.trigger = true;

        assert!(raycast(&[area.clone()], Vec3::new(0.0, 0.0, -5.0), Vec3::Z, 100.0, false).is_none());
        assert!(raycast(&[area], Vec3::new(0.0, 0.0, -5.0), Vec3::Z, 100.0, true).is_some());
    }

    #[test]
    fn a_turned_box_is_hit_on_its_corner() {
        let mut turned = collider(ColliderShape::Box);

        turned.rotation = Quat::from_rotation_y(PI / 4.0);

        let hit = raycast(&[turned], Vec3::new(0.0, 0.0, -5.0), Vec3::Z, 100.0, false).unwrap();

        // The corner of a unit box turned 45 degrees is 0.7071 from the middle.
        assert!(close(hit.distance, 5.0 - 0.5_f32.sqrt()));
    }
}
