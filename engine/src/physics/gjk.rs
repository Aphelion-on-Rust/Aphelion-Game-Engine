//! Do two convex shapes overlap, and if so how deep and in which direction?
//!
//! GJK answers "overlap or not" by building a small shape (a simplex) from support points of the
//! "Minkowski difference" A - B, which contains the origin exactly when A and B overlap.
//! EPA then grows that shape outward until it finds the side of A - B that is closest to the origin.
//! That side is the shortest way to push the two shapes apart.

use super::convex::Convex;
use glam::Vec3;

/// How two overlapping shapes can be pushed apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Penetration {
    /// Points from shape A toward shape B, length 1. Move A by `-normal * depth` (or B by `+normal * depth`).
    pub normal: Vec3,
    pub depth: f32,
}

/// A point of the Minkowski difference A - B.
fn minkowski(a: &Convex, b: &Convex, direction: Vec3) -> Vec3 {
    a.support(direction) - b.support(-direction)
}

fn same_way(a: Vec3, b: Vec3) -> bool {
    a.dot(b) > 0.0
}

// The simplex is kept newest point first. Each function reduces it to the part closest to the origin
// and sets the next direction to look in. They return true when the origin is inside the simplex.

fn line(simplex: &mut Vec<Vec3>, direction: &mut Vec3) -> bool {
    let (a, b) = (simplex[0], simplex[1]);
    let ab = b - a;
    let to_origin = -a;

    if same_way(ab, to_origin) {
        *direction = ab.cross(to_origin).cross(ab);
    } else {
        *simplex = vec![a];
        *direction = to_origin;
    }

    false
}

fn triangle(simplex: &mut Vec<Vec3>, direction: &mut Vec3) -> bool {
    let (a, b, c) = (simplex[0], simplex[1], simplex[2]);
    let (ab, ac) = (b - a, c - a);
    let to_origin = -a;
    let normal = ab.cross(ac);

    if same_way(normal.cross(ac), to_origin) {
        if same_way(ac, to_origin) {
            *simplex = vec![a, c];
            *direction = ac.cross(to_origin).cross(ac);
        } else {
            *simplex = vec![a, b];
            return line(simplex, direction);
        }
    } else if same_way(ab.cross(normal), to_origin) {
        *simplex = vec![a, b];
        return line(simplex, direction);
    } else if same_way(normal, to_origin) {
        *direction = normal;
    } else {
        *simplex = vec![a, c, b];
        *direction = -normal;
    }

    false
}

fn tetrahedron(simplex: &mut Vec<Vec3>, direction: &mut Vec3) -> bool {
    let (a, b, c, d) = (simplex[0], simplex[1], simplex[2], simplex[3]);
    let (ab, ac, ad) = (b - a, c - a, d - a);
    let to_origin = -a;

    if same_way(ab.cross(ac), to_origin) {
        *simplex = vec![a, b, c];
        return triangle(simplex, direction);
    }

    if same_way(ac.cross(ad), to_origin) {
        *simplex = vec![a, c, d];
        return triangle(simplex, direction);
    }

    if same_way(ad.cross(ab), to_origin) {
        *simplex = vec![a, d, b];
        return triangle(simplex, direction);
    }

    true
}

fn next_simplex(simplex: &mut Vec<Vec3>, direction: &mut Vec3) -> bool {
    match simplex.len() {
        2 => line(simplex, direction),
        3 => triangle(simplex, direction),
        4 => tetrahedron(simplex, direction),
        _ => false,
    }
}

/// The simplex around the origin if the shapes overlap, None if they don't (touching only does not count).
fn gjk(a: &Convex, b: &Convex) -> Option<Vec<Vec3>> {
    let mut direction = b.center() - a.center();

    if direction.length_squared() < 1e-10 {
        direction = Vec3::X;
    }

    let first = minkowski(a, b, direction);
    let mut simplex = vec![first];

    direction = -first;

    for _ in 0..64 {
        if direction.length_squared() < 1e-12 {
            // The origin is on the simplex.
            return Some(simplex);
        }

        let point = minkowski(a, b, direction);

        // The newest point did not get past the origin: no overlap.
        if point.dot(direction) <= 0.0 {
            return None;
        }

        simplex.insert(0, point);

        if next_simplex(&mut simplex, &mut direction) {
            return Some(simplex);
        }
    }

    None
}

pub fn overlaps(a: &Convex, b: &Convex) -> bool {
    gjk(a, b).is_some()
}

/// A side of the growing shape in EPA. `normal` points away from the inside.
struct Face {
    corners: [usize; 3],
    normal: Vec3,
    distance: f32,
}

/// Makes a face with its normal pointing away from `inside`. None if the three points are in a line.
fn make_face(points: &[Vec3], inside: Vec3, mut corners: [usize; 3]) -> Option<Face> {
    let (a, b, c) = (points[corners[0]], points[corners[1]], points[corners[2]]);
    let mut normal = (b - a).cross(c - a);

    if normal.length_squared() < 1e-12 {
        return None;
    }

    normal = normal.normalize();

    if normal.dot(a - inside) < 0.0 {
        corners.swap(1, 2);
        normal = -normal;
    }

    Some(Face { corners, normal, distance: normal.dot(a).max(0.0) })
}

/// A guess for when EPA can't run: push apart along the line between the middles.
fn fallback(a: &Convex, b: &Convex) -> Penetration {
    let mut normal = b.center() - a.center();

    if normal.length_squared() < 1e-10 {
        normal = Vec3::Y;
    }

    let normal = normal.normalize();
    let depth = (a.support(normal).dot(normal) - b.support(-normal).dot(normal)).max(0.0);

    Penetration { normal, depth }
}

/// How the shapes overlap, or None if they don't.
pub fn penetration(a: &Convex, b: &Convex) -> Option<Penetration> {
    let simplex = gjk(a, b)?;

    if simplex.len() < 4 {
        return Some(fallback(a, b));
    }

    let mut points = simplex;
    let inside = points.iter().copied().sum::<Vec3>() / 4.0;
    let mut faces: Vec<Face> = Vec::new();

    for corners in [[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]] {
        match make_face(&points, inside, corners) {
            Some(face) => faces.push(face),
            None => return Some(fallback(a, b)),
        }
    }

    for _ in 0..100 {
        let (normal, distance) = {
            let closest = faces.iter().min_by(|x, y| x.distance.total_cmp(&y.distance))?;

            (closest.normal, closest.distance)
        };
        let support = minkowski(a, b, normal);

        // The side can't be pushed out any further: it is on the surface of A - B.
        if support.dot(normal) - distance < 1e-3 {
            return Some(Penetration { normal, depth: distance });
        }

        let index = points.len();

        points.push(support);

        // Remove the faces that can see the new point and join the hole to it.
        let mut hole: Vec<(usize, usize)> = Vec::new();
        let mut kept = Vec::new();

        for face in faces.drain(..) {
            if face.normal.dot(support - points[face.corners[0]]) > 1e-6 {
                let [c0, c1, c2] = face.corners;

                for (from, to) in [(c0, c1), (c1, c2), (c2, c0)] {
                    // An edge shared with another removed face is inside the hole, not on its rim.
                    match hole.iter().position(|&(x, y)| x == to && y == from) {
                        Some(shared) => {
                            hole.remove(shared);
                        }
                        None => hole.push((from, to)),
                    }
                }
            } else {
                kept.push(face);
            }
        }

        faces = kept;

        for (from, to) in hole {
            if let Some(face) = make_face(&points, inside, [from, to, index]) {
                faces.push(face);
            }
        }

        if faces.is_empty() {
            return Some(fallback(a, b));
        }
    }

    // Did not settle: use the closest side found so far.
    let closest = faces.iter().min_by(|x, y| x.distance.total_cmp(&y.distance))?;

    Some(Penetration { normal: closest.normal, depth: closest.distance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ColliderShape;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::physics::WorldCollider;
    use glam::Quat;

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

    fn at(shape: ColliderShape, position: Vec3) -> Convex {
        Convex::new(&collider(shape), position)
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.02
    }

    const SHAPES: [ColliderShape; 5] =
        [ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule, ColliderShape::Cylinder, ColliderShape::Prism];

    #[test]
    fn two_overlapping_balls_are_pushed_apart_along_the_line_between_them() {
        let (a, b) = (at(ColliderShape::Sphere, Vec3::ZERO), at(ColliderShape::Sphere, Vec3::new(0.8, 0.0, 0.0)));
        let found = penetration(&a, &b).unwrap();

        assert!(close(found.depth, 0.2), "depth {}", found.depth);
        assert!(found.normal.dot(Vec3::X) > 0.99, "normal {:?}", found.normal);
    }

    #[test]
    fn two_overlapping_boxes_are_pushed_apart_along_the_shortest_way() {
        let (a, b) = (at(ColliderShape::Box, Vec3::ZERO), at(ColliderShape::Box, Vec3::new(0.8, 0.1, 0.0)));
        let found = penetration(&a, &b).unwrap();

        assert!(close(found.depth, 0.2), "depth {}", found.depth);
        assert!(found.normal.dot(Vec3::X) > 0.99, "normal {:?}", found.normal);

        // The other way round the normal flips.
        let flipped = penetration(&b, &a).unwrap();

        assert!(flipped.normal.dot(Vec3::NEG_X) > 0.99);
    }

    #[test]
    fn a_capsule_standing_on_a_box_pushes_up() {
        let floor = Convex::new(&WorldCollider { size: Vec3::new(2.0, 1.0, 2.0), ..collider(ColliderShape::Box) }, Vec3::ZERO);
        // The capsule is 2 tall with radius 0.5, so its middle at 1.4 puts its bottom 0.1 into the box top (0.5).
        let capsule = at(ColliderShape::Capsule, Vec3::new(0.0, 1.4, 0.0));
        let found = penetration(&floor, &capsule).unwrap();

        assert!(close(found.depth, 0.1), "depth {}", found.depth);
        assert!(found.normal.dot(Vec3::Y) > 0.99, "normal {:?}", found.normal);
    }

    #[test]
    fn a_ball_resting_on_a_box_pushes_up() {
        let floor = Convex::new(&WorldCollider { size: Vec3::new(10.0, 1.0, 10.0), ..collider(ColliderShape::Box) }, Vec3::new(0.0, -0.5, 0.0));
        let ball = at(ColliderShape::Sphere, Vec3::new(0.3, 0.45, 0.2));
        let found = penetration(&floor, &ball).unwrap();

        assert!(close(found.depth, 0.05), "depth {}", found.depth);
        assert!(found.normal.dot(Vec3::Y) > 0.99);
    }

    #[test]
    fn shapes_that_do_not_touch_do_not_overlap_and_every_pair_of_shapes_works() {
        for first in SHAPES {
            for second in SHAPES {
                let near = at(second, Vec3::new(0.3, 0.2, 0.1));
                let far = at(second, Vec3::new(10.0, 0.0, 0.0));
                let a = at(first, Vec3::ZERO);

                assert!(overlaps(&a, &near), "{first:?} and {second:?} should overlap");
                assert!(!overlaps(&a, &far), "{first:?} and {second:?} should be apart");

                let found = penetration(&a, &near).unwrap_or_else(|| panic!("{first:?} and {second:?} gave no penetration"));

                assert!(found.depth > 0.0 && found.depth < 3.0, "{first:?} and {second:?}: depth {}", found.depth);
                assert!((found.normal.length() - 1.0).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn shapes_that_are_exactly_on_top_of_each_other_still_give_an_answer() {
        let (a, b) = (at(ColliderShape::Sphere, Vec3::ZERO), at(ColliderShape::Sphere, Vec3::ZERO));
        let found = penetration(&a, &b).unwrap();

        assert!(found.depth > 0.0);
        assert!((found.normal.length() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn a_turned_box_is_found_by_its_support_points() {
        let mut turned = collider(ColliderShape::Box);

        turned.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);

        let shape = Convex::new(&turned, Vec3::ZERO);

        // The corner of a unit box turned 45 degrees points straight along X, 0.7071 from the middle.
        assert!((shape.support(Vec3::X).x - 0.5_f32.sqrt()).abs() < 1e-4);
    }
}
