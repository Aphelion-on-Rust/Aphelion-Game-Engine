# Physics

Step 8a is the base: **colliders**, **bodies** and **raycasts**. Step 8b makes bodies **move and collide**.
Still to come: pressing Play in the editor to see it (8c), 2D physics, joints, the character controller.

## Components

**Collider3D** gives an object a solid shape. Pick the shape, the Inspector only shows the fields that shape uses.

| Shape      | Fields it uses                                  |
|------------|--------------------------------------------------|
| `Box`      | `size` (the sides)                               |
| `Sphere`   | `radius`                                         |
| `Capsule`  | `radius`, `height` (the round ends count in the height) |
| `Cylinder` | `radius`, `height`                               |
| `Prism`    | `sides` (3 to 32), `radius` of the base, `height` |

Every shape also has `center` (where it is, in the object's own space), `trigger` (not solid, things pass through and
it only notices), `friction` (0 slippery, 1 grippy) and `bounciness` (0 none, 1 full).
Shapes stand along the Y axis.

**Body3D** makes an object a physical thing: `kind` (`Dynamic` is moved by forces, `Kinematic` only by code but
pushes others, `Static` never moves), `mass`, `gravity_scale`, `linear_damping`, `angular_damping`,
`lock_rotation`. The physics world (8b) uses `kind`, `mass`, `gravity_scale` and `linear_damping`.

The Create menu has three ready-made objects: **Physics Cube**, **Physics Sphere** and **Floor**.

## Simulation (`flux_engine::physics::PhysicsWorld`)

`PhysicsWorld::from_scene(scene, settings)` builds a world from the active objects that have a Body3D or a collider.
An object with a collider but no Body3D is **static**. `world.step(dt)` moves time forward and returns the events.

- **Dynamic** bodies fall with gravity (`settings.gravity`, default 0, -9.81, 0), slow down by their damping, and are pushed
  out of whatever they hit. A lighter body is pushed more than a heavier one.
- **Kinematic** bodies move only with the velocity you give them (`set_velocity`). Dynamic bodies can't push through
  them and are carried by friction. **Static** bodies never move.
- **Bounce**: the bigger of the two colliders' `bounciness`. Slow touches (under 1 m/s) don't bounce, so things come to rest.
- **Friction**: the square root of the two colliders' `friction`, never stronger than how hard the two are pressed together.
- **Triggers** are not solid. When something goes into a trigger or leaves it, you get `TriggerEnter` / `TriggerExit`.
  Solid bodies that start or stop touching give `CollisionEnter` / `CollisionExit`. At least one of the two must be able to
  move (or be kinematic) for a trigger to notice it.
- Time is cut into small steps (`substeps`, default 4) and contacts are fixed several times per step (`iterations`,
  default 4) so stacks of boxes settle.
- `positions()` lists where every body that can move is, `raycast()` shoots a ray at the world as it is now,
  `add_impulse()` gives a sudden push.

How two shapes are tested: each shape can say which of its points is farthest in any direction (`convex.rs`), and
GJK and EPA (`gjk.rs`) use that to find out if two shapes overlap and the shortest way to push them apart.
It works the same for all five shapes and every pair of them.

**Limits of 8b:** bodies move but do not turn (they keep their rotation, so `lock_rotation` is always on for now),
there is no sleeping, no protection against very fast bodies passing through thin things, and every pair of bodies is
tested (fine for hundreds of bodies, a faster broad phase comes later). The world is not connected to the editor yet,
that is step 8c.

## Scale

A collider follows the scale of its object: a box is scaled per axis, a sphere by the biggest scale, and a capsule,
cylinder or prism uses the bigger of X and Z for its radius and Y for its height. An object scaled to nothing, or an
inactive object, has no collider.

## Raycast

`flux_engine::physics::raycast(colliders, origin, direction, max_distance, include_triggers)` returns the first
collider hit as a `RayHit` (object, distance, point, normal). `extract_colliders(scene)` makes the list of colliders
with their world position, rotation and scaled size.

- Only a ray that comes from **outside** a shape hits it. A ray that starts inside does not hit it.
- Triggers are skipped unless `include_triggers` is true.
- The direction does not need to be length 1.

## In the editor

Select an object with a collider and its outline is drawn in the Scene view: **green** for a solid collider,
**cyan** for a trigger. (Outlines are lines drawn on top of everything, see `docs/RENDERING.md`.)

## Files

- `engine/src/components/collider.rs`, `body.rs` the data
- `engine/src/physics/world.rs` colliders in the world
- `engine/src/physics/raycast.rs` rays against every shape
- `engine/src/physics/convex.rs`, `gjk.rs` the collision test for every pair of shapes
- `engine/src/physics/sim.rs` the physics world
- `grp/src/gizmos.rs` the outline lines
