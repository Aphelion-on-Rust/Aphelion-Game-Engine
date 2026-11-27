//! Moving bodies: gravity, damping, collisions with friction and bounce, and events.
//!
//! How it works, in short:
//! - Time is cut into small steps (`substeps`). In each: bodies move with their velocity, then touching
//!   shapes are pushed apart and their velocities are fixed (`iterations` times, so stacks settle).
//! - Dynamic bodies are moved by physics. Kinematic bodies move only with the velocity you give them,
//!   but dynamic bodies can't push through them. Static bodies (objects with a collider but no Body3D, or
//!   a Static body) never move.
//! - Triggers are not solid. They only report when something enters or leaves them.
//!
//! Limits of this step: bodies move but do not turn (rotation is kept as it was), and there is no sleeping or
//! fast-body protection. Every pair of bodies is tested, so very big scenes will need a faster broad phase.

use super::convex::Convex;
use super::gjk::{overlaps, penetration};
use super::raycast::{raycast, RayHit};
use super::world::{extract_colliders, WorldCollider};
use crate::components::BodyKind;
use crate::identity::AssetUuid;
use crate::scene::transform::world_matrix;
use crate::scene::{query, ComponentData, SceneData};
use glam::Vec3;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsSettings {
    pub gravity: Vec3,
    /// How many small steps one `step` is cut into.
    pub substeps: u32,
    /// How many times per small step the contacts are fixed.
    pub iterations: u32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self { gravity: Vec3::new(0.0, -9.81, 0.0), substeps: 4, iterations: 4 }
    }
}

/// How a body behaves. (Shapes come from its colliders.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyParams {
    pub kind: BodyKind,
    pub mass: f32,
    pub gravity_scale: f32,
    pub linear_damping: f32,
}

impl Default for BodyParams {
    fn default() -> Self {
        Self { kind: BodyKind::Dynamic, mass: 1.0, gravity_scale: 1.0, linear_damping: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsEventKind {
    /// Two solid bodies started touching.
    CollisionEnter,
    CollisionExit,
    /// A body went into a trigger.
    TriggerEnter,
    TriggerExit,
}

/// Something that happened between two objects during a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsEvent {
    pub kind: PhysicsEventKind,
    pub a: AssetUuid,
    pub b: AssetUuid,
}

struct SimCollider {
    /// The shape. Its position is only the starting one, the real one is body position + `offset`.
    shape: WorldCollider,
    offset: Vec3,
}

struct SimBody {
    object: AssetUuid,
    kind: BodyKind,
    /// The origin of the object in the world.
    position: Vec3,
    velocity: Vec3,
    /// 1 / mass for dynamic bodies, 0 for everything that can't be pushed.
    inverse_mass: f32,
    gravity_scale: f32,
    linear_damping: f32,
    colliders: Vec<SimCollider>,
}

/// Pairs are remembered as the indexes of the two bodies, smaller first.
type Pair = (usize, usize);

pub struct PhysicsWorld {
    settings: PhysicsSettings,
    bodies: Vec<SimBody>,
    touching: HashSet<Pair>,
    inside_triggers: HashSet<Pair>,
}

impl PhysicsWorld {
    pub fn new(settings: PhysicsSettings) -> Self {
        Self { settings, bodies: Vec::new(), touching: HashSet::new(), inside_triggers: HashSet::new() }
    }

    /// Builds the world from a scene: every active object with a Body3D or a collider. Objects with a collider
    /// but no body are static. The body's origin is the object's position in the world.
    pub fn from_scene(scene: &SceneData, settings: PhysicsSettings) -> Self {
        let mut world = Self::new(settings);
        let colliders = extract_colliders(scene);

        for object in &scene.objects {
            if !query::is_active_in_hierarchy(&scene.objects, &object.uuid) {
                continue;
            }

            let body = object.components.iter().find_map(|entry| match &entry.data {
                ComponentData::Body3D(body) => Some(body.clone()),
                _ => None,
            });
            let own: Vec<WorldCollider> = colliders.iter().filter(|collider| collider.object == object.uuid).cloned().collect();

            if body.is_none() && own.is_empty() {
                continue;
            }

            let origin = world_matrix(&scene.objects, &object.uuid).map_or(Vec3::ZERO, |matrix| matrix.w_axis.truncate());
            let params = match body {
                Some(body) => BodyParams {
                    kind: body.kind,
                    mass: body.mass,
                    gravity_scale: body.gravity_scale,
                    linear_damping: body.linear_damping,
                },
                None => BodyParams { kind: BodyKind::Static, ..Default::default() },
            };

            world.add_body(object.uuid, params, origin, own);
        }

        world
    }

    /// Adds a body. The colliders are given where they are in the world right now.
    pub fn add_body(&mut self, object: AssetUuid, params: BodyParams, position: Vec3, colliders: Vec<WorldCollider>) {
        let inverse_mass = if params.kind == BodyKind::Dynamic { 1.0 / params.mass.max(0.001) } else { 0.0 };

        self.bodies.push(SimBody {
            object,
            kind: params.kind,
            position,
            velocity: Vec3::ZERO,
            inverse_mass,
            gravity_scale: params.gravity_scale,
            linear_damping: params.linear_damping.max(0.0),
            colliders: colliders.into_iter().map(|shape| SimCollider { offset: shape.position - position, shape }).collect(),
        });
    }

    fn body(&self, object: &AssetUuid) -> Option<&SimBody> {
        self.bodies.iter().find(|body| body.object == *object)
    }

    pub fn body_position(&self, object: &AssetUuid) -> Option<Vec3> {
        self.body(object).map(|body| body.position)
    }

    pub fn velocity(&self, object: &AssetUuid) -> Option<Vec3> {
        self.body(object).map(|body| body.velocity)
    }

    /// Sets the velocity of a body (for a kinematic body this is how it moves). False if there is no such body.
    pub fn set_velocity(&mut self, object: &AssetUuid, velocity: Vec3) -> bool {
        match self.bodies.iter_mut().find(|body| body.object == *object) {
            Some(body) => {
                body.velocity = velocity;
                true
            }
            None => false,
        }
    }

    /// A sudden push: changes the velocity by `impulse / mass`. Does nothing to bodies that aren't dynamic.
    pub fn add_impulse(&mut self, object: &AssetUuid, impulse: Vec3) -> bool {
        match self.bodies.iter_mut().find(|body| body.object == *object) {
            Some(body) => {
                body.velocity += impulse * body.inverse_mass;
                true
            }
            None => false,
        }
    }

    /// Where every body that can move is now (static bodies never move, so they are not listed).
    pub fn positions(&self) -> Vec<(AssetUuid, Vec3)> {
        self.bodies.iter().filter(|body| body.kind != BodyKind::Static).map(|body| (body.object, body.position)).collect()
    }

    /// Every collider, where it is now.
    pub fn colliders(&self) -> Vec<WorldCollider> {
        let mut all = Vec::new();

        for body in &self.bodies {
            for collider in &body.colliders {
                all.push(WorldCollider { position: body.position + collider.offset, ..collider.shape.clone() });
            }
        }

        all
    }

    /// The first collider a ray hits right now.
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32, include_triggers: bool) -> Option<RayHit> {
        raycast(&self.colliders(), origin, direction, max_distance, include_triggers)
    }

    /// Moves time forward by `dt` seconds and returns what started or stopped touching.
    pub fn step(&mut self, dt: f32) -> Vec<PhysicsEvent> {
        if dt <= 0.0 {
            return Vec::new();
        }

        let substeps = self.settings.substeps.max(1);
        let small = dt / substeps as f32;
        let mut touching_now = HashSet::new();

        for _ in 0..substeps {
            self.integrate(small);
            self.solve(&mut touching_now);
        }

        let inside_now = self.trigger_overlaps();
        let mut events = self.changes(&self.touching, &touching_now, PhysicsEventKind::CollisionEnter, PhysicsEventKind::CollisionExit);

        events.extend(self.changes(&self.inside_triggers, &inside_now, PhysicsEventKind::TriggerEnter, PhysicsEventKind::TriggerExit));
        self.touching = touching_now;
        self.inside_triggers = inside_now;

        events
    }

    fn changes(&self, before: &HashSet<Pair>, now: &HashSet<Pair>, enter: PhysicsEventKind, exit: PhysicsEventKind) -> Vec<PhysicsEvent> {
        let event = |kind: PhysicsEventKind, pair: &Pair| PhysicsEvent { kind, a: self.bodies[pair.0].object, b: self.bodies[pair.1].object };
        let mut entered: Vec<&Pair> = now.difference(before).collect();
        let mut left: Vec<&Pair> = before.difference(now).collect();

        // A set has no order, so sort to give the same events in the same order every time.
        entered.sort();
        left.sort();

        entered.into_iter().map(|pair| event(enter, pair)).chain(left.into_iter().map(|pair| event(exit, pair))).collect()
    }

    /// Gravity, damping and moving.
    fn integrate(&mut self, small: f32) {
        let gravity = self.settings.gravity;

        for body in &mut self.bodies {
            match body.kind {
                BodyKind::Dynamic => {
                    body.velocity += gravity * body.gravity_scale * small;
                    body.velocity /= 1.0 + small * body.linear_damping;
                    body.position += body.velocity * small;
                }
                BodyKind::Kinematic => body.position += body.velocity * small,
                BodyKind::Static => {}
            }
        }
    }

    /// Pushes touching solid shapes apart and fixes their velocities.
    fn solve(&mut self, touching: &mut HashSet<Pair>) {
        let count = self.bodies.len();
        // What each contact pushed so far in this small step, so friction knows how hard things are pressed together.
        let mut pressed: HashMap<(usize, usize, usize, usize), f32> = HashMap::new();
        let mut slid: HashMap<(usize, usize, usize, usize), Vec3> = HashMap::new();

        for _ in 0..self.settings.iterations.max(1) {
            for i in 0..count {
                for j in (i + 1)..count {
                    let (inverse_a, inverse_b) = (self.bodies[i].inverse_mass, self.bodies[j].inverse_mass);
                    let total = inverse_a + inverse_b;

                    // Two things that can't be pushed (static or kinematic) never need fixing.
                    if total <= 0.0 {
                        continue;
                    }

                    for ci in 0..self.bodies[i].colliders.len() {
                        for cj in 0..self.bodies[j].colliders.len() {
                            let (a, b, friction, bounciness, solid) = {
                                let (first, second) = (&self.bodies[i].colliders[ci], &self.bodies[j].colliders[cj]);

                                (
                                    Convex::new(&first.shape, self.bodies[i].position + first.offset),
                                    Convex::new(&second.shape, self.bodies[j].position + second.offset),
                                    (first.shape.friction * second.shape.friction).max(0.0).sqrt(),
                                    first.shape.bounciness.max(second.shape.bounciness),
                                    !first.shape.trigger && !second.shape.trigger,
                                )
                            };

                            if !solid || (a.center() - b.center()).length() > a.bounding_radius() + b.bounding_radius() {
                                continue;
                            }

                            let Some(found) = penetration(&a, &b) else {
                                continue;
                            };
                            let normal = found.normal;
                            let key = (i, j, ci, cj);

                            touching.insert((i, j));

                            // Push apart, the lighter one more. A little overlap is left so contacts don't flicker.
                            let correction = (found.depth - 0.001).max(0.0);

                            self.bodies[i].position -= normal * correction * (inverse_a / total);
                            self.bodies[j].position += normal * correction * (inverse_b / total);

                            // Bounce: only when they come together (vn < 0), and only for a real hit, so resting things don't jitter.
                            let approach = (self.bodies[j].velocity - self.bodies[i].velocity).dot(normal);

                            if approach < 0.0 {
                                let bounce = if -approach > 1.0 { bounciness } else { 0.0 };
                                let impulse = -(1.0 + bounce) * approach / total;

                                self.bodies[i].velocity -= normal * impulse * inverse_a;
                                self.bodies[j].velocity += normal * impulse * inverse_b;
                                *pressed.entry(key).or_insert(0.0) += impulse;
                            }

                            // Friction: slow the sliding, but never more than friction * how hard they are pressed together.
                            let relative = self.bodies[j].velocity - self.bodies[i].velocity;
                            let sliding = relative - normal * relative.dot(normal);
                            let speed = sliding.length();

                            if speed > 1e-6 {
                                let direction = sliding / speed;
                                let limit = friction * pressed.get(&key).copied().unwrap_or(0.0);
                                let before = slid.get(&key).copied().unwrap_or(Vec3::ZERO);
                                let mut after = before - direction * (speed / total);

                                if after.length() > limit {
                                    after = after.normalize_or_zero() * limit;
                                }

                                let change = after - before;

                                self.bodies[i].velocity -= change * inverse_a;
                                self.bodies[j].velocity += change * inverse_b;
                                slid.insert(key, after);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Which bodies are inside which triggers right now (at least one of the two must be able to move,
    /// or be kinematic, like in most engines).
    fn trigger_overlaps(&self) -> HashSet<Pair> {
        let mut inside = HashSet::new();

        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                if self.bodies[i].kind == BodyKind::Static && self.bodies[j].kind == BodyKind::Static {
                    continue;
                }

                'pair: for first in &self.bodies[i].colliders {
                    for second in &self.bodies[j].colliders {
                        if !first.shape.trigger && !second.shape.trigger {
                            continue;
                        }

                        let a = Convex::new(&first.shape, self.bodies[i].position + first.offset);
                        let b = Convex::new(&second.shape, self.bodies[j].position + second.offset);

                        if (a.center() - b.center()).length() <= a.bounding_radius() + b.bounding_radius() && overlaps(&a, &b) {
                            inside.insert((i, j));
                            break 'pair;
                        }
                    }
                }
            }
        }

        inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ColliderShape;
    use crate::identity::{IdController, IdRegistryModel};
    use crate::scene::{ObjectPreset, SceneController};
    use glam::Quat;

    const FRAME: f32 = 1.0 / 60.0;

    fn id() -> AssetUuid {
        IdController.new_uuid(&mut IdRegistryModel::default())
    }

    fn collider(shape: ColliderShape, position: Vec3) -> WorldCollider {
        WorldCollider {
            object: id(),
            shape,
            position,
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

    fn add(world: &mut PhysicsWorld, params: BodyParams, position: Vec3, mut shape: WorldCollider) -> AssetUuid {
        let object = id();

        shape.object = object;
        shape.position = position;
        world.add_body(object, params, position, vec![shape]);
        object
    }

    fn dynamic() -> BodyParams {
        BodyParams::default()
    }

    fn immovable(kind: BodyKind) -> BodyParams {
        BodyParams { kind, ..Default::default() }
    }

    /// A wide static floor whose top is at y = 0.
    fn floor(world: &mut PhysicsWorld, friction: f32) -> AssetUuid {
        let shape = WorldCollider { size: Vec3::new(20.0, 1.0, 20.0), friction, ..collider(ColliderShape::Box, Vec3::ZERO) };

        add(world, immovable(BodyKind::Static), Vec3::new(0.0, -0.5, 0.0), shape)
    }

    fn run(world: &mut PhysicsWorld, seconds: f32) -> Vec<PhysicsEvent> {
        let mut events = Vec::new();

        for _ in 0..(seconds / FRAME).round() as usize {
            events.extend(world.step(FRAME));
        }

        events
    }

    fn near(a: f32, b: f32, tolerance: f32) -> bool {
        (a - b).abs() <= tolerance
    }

    #[test]
    fn a_body_in_the_air_falls_with_gravity() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 5.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        run(&mut world, 1.0);

        assert!(near(world.velocity(&ball).unwrap().y, -9.81, 0.02));
        assert!(near(world.body_position(&ball).unwrap().y, 5.0 - 4.905, 0.1));
    }

    #[test]
    fn gravity_scale_and_damping_change_the_fall() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let floating = add(&mut world, BodyParams { gravity_scale: 0.0, ..dynamic() }, Vec3::new(0.0, 5.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));
        let heavy = add(&mut world, BodyParams { gravity_scale: 2.0, ..dynamic() }, Vec3::new(4.0, 5.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));
        let slowed = add(&mut world, BodyParams { linear_damping: 5.0, ..dynamic() }, Vec3::new(8.0, 5.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        run(&mut world, 1.0);

        assert_eq!(world.velocity(&floating).unwrap(), Vec3::ZERO);
        assert!(near(world.velocity(&heavy).unwrap().y, -19.62, 0.05));
        assert!(world.velocity(&slowed).unwrap().y > -3.0);
    }

    #[test]
    fn a_ball_lands_on_the_floor_and_comes_to_rest_on_top_of_it() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let ground = floor(&mut world, 0.5);
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 3.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        run(&mut world, 3.0);

        let position = world.body_position(&ball).unwrap();

        assert!(near(position.y, 0.5, 0.02), "rests at {}", position.y);
        assert!(world.velocity(&ball).unwrap().length() < 0.1);
        assert_eq!(world.body_position(&ground).unwrap(), Vec3::new(0.0, -0.5, 0.0));
    }

    #[test]
    fn a_bouncy_ball_comes_back_up_with_most_of_its_speed() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());

        floor(&mut world, 0.5);

        let shape = WorldCollider { bounciness: 0.8, ..collider(ColliderShape::Sphere, Vec3::ZERO) };
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 3.0, 0.0), shape);
        let mut best_up: f32 = 0.0;

        for _ in 0..120 {
            world.step(FRAME);
            best_up = best_up.max(world.velocity(&ball).unwrap().y);
        }

        // It hits at sqrt(2 * 9.81 * 2.5) = 7.0, so it leaves at about 0.8 * 7.0 = 5.6.
        assert!(best_up > 4.5 && best_up < 6.5, "bounced up at {best_up}");
    }

    #[test]
    fn a_kinematic_body_ignores_gravity_and_carries_what_lands_on_it() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let platform_shape = WorldCollider { size: Vec3::new(4.0, 1.0, 4.0), ..collider(ColliderShape::Box, Vec3::ZERO) };
        let platform = add(&mut world, immovable(BodyKind::Kinematic), Vec3::ZERO, platform_shape);
        let crate_box = add(&mut world, dynamic(), Vec3::new(0.0, 2.0, 0.0), collider(ColliderShape::Box, Vec3::ZERO));

        run(&mut world, 2.0);

        assert_eq!(world.body_position(&platform).unwrap(), Vec3::ZERO);
        assert!(near(world.body_position(&crate_box).unwrap().y, 1.0, 0.03)); // 0.5 platform top + 0.5 half crate

        // Move the platform sideways: it goes where its velocity says, whatever is on it.
        world.set_velocity(&platform, Vec3::new(1.0, 0.0, 0.0));
        run(&mut world, 1.0);

        assert!(near(world.body_position(&platform).unwrap().x, 1.0, 0.05));
        assert!(world.body_position(&crate_box).unwrap().x > 0.5, "friction carries the crate along");
    }

    #[test]
    fn a_heavy_body_does_not_move_a_static_one() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let ground = floor(&mut world, 0.5);

        add(&mut world, BodyParams { mass: 1000.0, ..dynamic() }, Vec3::new(0.0, 4.0, 0.0), collider(ColliderShape::Box, Vec3::ZERO));
        run(&mut world, 2.0);

        assert_eq!(world.body_position(&ground).unwrap(), Vec3::new(0.0, -0.5, 0.0));
        assert!(world.positions().iter().all(|(object, _)| *object != ground), "static bodies are not listed");
    }

    #[test]
    fn two_equal_balls_hitting_head_on_swap_their_speeds() {
        let mut world = PhysicsWorld::new(PhysicsSettings { gravity: Vec3::ZERO, ..Default::default() });
        let shape = WorldCollider { bounciness: 1.0, ..collider(ColliderShape::Sphere, Vec3::ZERO) };
        let moving = add(&mut world, dynamic(), Vec3::new(-2.0, 0.0, 0.0), shape.clone());
        let resting = add(&mut world, dynamic(), Vec3::ZERO, shape);

        world.set_velocity(&moving, Vec3::new(3.0, 0.0, 0.0));
        run(&mut world, 1.0);

        assert!(world.velocity(&moving).unwrap().x.abs() < 0.2, "{:?}", world.velocity(&moving));
        assert!(near(world.velocity(&resting).unwrap().x, 3.0, 0.2), "{:?}", world.velocity(&resting));
    }

    #[test]
    fn a_light_body_is_pushed_more_than_a_heavy_one() {
        let mut world = PhysicsWorld::new(PhysicsSettings { gravity: Vec3::ZERO, ..Default::default() });
        let light = add(&mut world, BodyParams { mass: 1.0, ..dynamic() }, Vec3::new(-0.9, 0.0, 0.0), collider(ColliderShape::Box, Vec3::ZERO));
        let heavy = add(&mut world, BodyParams { mass: 9.0, ..dynamic() }, Vec3::new(0.0, 0.0, 0.0), collider(ColliderShape::Box, Vec3::ZERO));

        world.step(FRAME);

        let (light_moved, heavy_moved) = ((world.body_position(&light).unwrap().x + 0.9).abs(), world.body_position(&heavy).unwrap().x.abs());

        assert!(light_moved > heavy_moved * 5.0, "light {light_moved}, heavy {heavy_moved}");
    }

    #[test]
    fn friction_stops_a_sliding_box_and_no_friction_does_not() {
        for (friction, should_stop) in [(0.5, true), (0.0, false)] {
            let mut world = PhysicsWorld::new(PhysicsSettings::default());

            floor(&mut world, friction);

            let shape = WorldCollider { friction, ..collider(ColliderShape::Box, Vec3::ZERO) };
            let sliding = add(&mut world, dynamic(), Vec3::new(0.0, 0.5, 0.0), shape);

            world.set_velocity(&sliding, Vec3::new(3.0, 0.0, 0.0));
            run(&mut world, 2.0);

            let (speed, travelled) = (world.velocity(&sliding).unwrap().x, world.body_position(&sliding).unwrap().x);

            if should_stop {
                // It slows down by friction * g = 4.9 per second, so it stops after 3^2 / (2 * 4.9) = 0.92 m.
                assert!(speed.abs() < 0.05, "still moving at {speed}");
                assert!(near(travelled, 0.92, 0.25), "stopped after {travelled}");
            } else {
                assert!(near(speed, 3.0, 0.05), "slowed to {speed}");
            }
        }
    }

    #[test]
    fn two_boxes_stack_without_sinking_into_each_other() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());

        floor(&mut world, 0.5);

        let bottom = add(&mut world, dynamic(), Vec3::new(0.0, 0.5, 0.0), collider(ColliderShape::Box, Vec3::ZERO));
        let top = add(&mut world, dynamic(), Vec3::new(0.0, 1.6, 0.0), collider(ColliderShape::Box, Vec3::ZERO));

        run(&mut world, 3.0);

        assert!(near(world.body_position(&bottom).unwrap().y, 0.5, 0.03), "{:?}", world.body_position(&bottom));
        assert!(near(world.body_position(&top).unwrap().y, 1.5, 0.04), "{:?}", world.body_position(&top));
    }

    #[test]
    fn every_shape_lands_on_the_floor_and_stays_there() {
        for shape in [ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule, ColliderShape::Cylinder, ColliderShape::Prism] {
            let mut world = PhysicsWorld::new(PhysicsSettings::default());

            floor(&mut world, 0.5);

            let object = add(&mut world, dynamic(), Vec3::new(0.0, 4.0, 0.0), collider(shape, Vec3::ZERO));

            run(&mut world, 3.0);

            // Half the height of each shape standing on the floor.
            let rest = match shape {
                ColliderShape::Box | ColliderShape::Sphere => 0.5,
                _ => 1.0,
            };
            let height = world.body_position(&object).unwrap().y;

            assert!(near(height, rest, 0.03), "{shape:?} rests at {height}");
        }
    }

    #[test]
    fn a_trigger_is_not_solid_and_reports_when_something_goes_in_and_out() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let zone_shape = WorldCollider { size: Vec3::new(4.0, 2.0, 4.0), trigger: true, ..collider(ColliderShape::Box, Vec3::ZERO) };
        let zone = add(&mut world, immovable(BodyKind::Static), Vec3::new(0.0, 1.0, 0.0), zone_shape);
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 4.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        let events = run(&mut world, 1.5);
        let kinds: Vec<PhysicsEventKind> = events.iter().map(|event| event.kind).collect();

        // It fell straight through (nothing pushed it) and the trigger noticed both ways.
        assert_eq!(kinds, vec![PhysicsEventKind::TriggerEnter, PhysicsEventKind::TriggerExit]);
        assert!(events.iter().all(|event| [event.a, event.b].contains(&zone) && [event.a, event.b].contains(&ball)));
        assert!(world.body_position(&ball).unwrap().y < 0.0, "the ball went right through");
    }

    #[test]
    fn landing_on_the_floor_reports_one_collision_enter() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let ground = floor(&mut world, 0.5);
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 2.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));
        let events = run(&mut world, 3.0);
        let enters: Vec<&PhysicsEvent> = events.iter().filter(|event| event.kind == PhysicsEventKind::CollisionEnter).collect();

        assert!(!enters.is_empty());
        assert!(enters.iter().all(|event| [event.a, event.b].contains(&ground) && [event.a, event.b].contains(&ball)));
    }

    #[test]
    fn impulses_and_velocity_need_a_real_body() {
        let mut world = PhysicsWorld::new(PhysicsSettings { gravity: Vec3::ZERO, ..Default::default() });
        let object = add(&mut world, BodyParams { mass: 2.0, ..dynamic() }, Vec3::ZERO, collider(ColliderShape::Sphere, Vec3::ZERO));
        let fixed = add(&mut world, immovable(BodyKind::Static), Vec3::new(9.0, 0.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        assert!(world.add_impulse(&object, Vec3::new(4.0, 0.0, 0.0)));
        assert_eq!(world.velocity(&object).unwrap(), Vec3::new(2.0, 0.0, 0.0)); // impulse / mass

        world.add_impulse(&fixed, Vec3::new(4.0, 0.0, 0.0));
        assert_eq!(world.velocity(&fixed).unwrap(), Vec3::ZERO);

        assert!(!world.set_velocity(&id(), Vec3::X));
        assert!(!world.add_impulse(&id(), Vec3::X));
        assert!(world.velocity(&id()).is_none());
    }

    #[test]
    fn a_world_built_from_a_scene_drops_the_physics_cube_onto_the_floor() {
        let mut ids = IdRegistryModel::default();
        let mut scene = SceneController.create_scene(&mut ids, "Test");
        let ground = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::Floor, None).unwrap();
        let cube = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsCube, None).unwrap();
        let sphere = SceneController.create_from_preset(&mut scene, &mut ids, ObjectPreset::PhysicsSphere, None).unwrap();
        let mut world = PhysicsWorld::from_scene(crate::mivc::Model::data(&scene), PhysicsSettings::default());

        run(&mut world, 3.0);

        let moved = world.positions();

        assert_eq!(moved.len(), 2, "the floor is static");
        assert!(near(world.body_position(&cube).unwrap().y, 0.5, 0.03));
        assert!(near(world.body_position(&sphere).unwrap().y, 0.5, 0.03));
        assert_eq!(world.body_position(&ground).unwrap(), Vec3::ZERO);
    }

    #[test]
    fn a_ray_can_be_shot_at_the_world_as_it_is_now() {
        let mut world = PhysicsWorld::new(PhysicsSettings::default());
        let ball = add(&mut world, dynamic(), Vec3::new(0.0, 10.0, 0.0), collider(ColliderShape::Sphere, Vec3::ZERO));

        assert!(world.raycast(Vec3::new(0.0, 10.0, -5.0), Vec3::Z, 100.0, false).is_some());

        run(&mut world, 1.0);

        // The ball has fallen, so the same ray misses it now.
        assert!(world.raycast(Vec3::new(0.0, 10.0, -5.0), Vec3::Z, 100.0, false).is_none());
        assert_eq!(world.raycast(Vec3::new(0.0, world.body_position(&ball).unwrap().y, -5.0), Vec3::Z, 100.0, false).unwrap().object, ball);
    }
}
