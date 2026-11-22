# Physics

Step 8a is the base: **colliders**, **bodies** (data only for now) and **raycasts**. Moving things (gravity,
collisions, joints, 2D physics, the character controller) come in the next steps.

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
`lock_rotation`. In 8a it only stores these values, the movement comes in 8b.

The Create menu has three ready-made objects: **Physics Cube**, **Physics Sphere** and **Floor**.

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
- `grp/src/gizmos.rs` the outline lines
