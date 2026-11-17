# Rendering

The renderer is built in steps. Step 7a is everything that happens **before** a pixel is drawn, step 7b draws it on the CPU.

| Step | What                                                                              |
| ---- | --------------------------------------------------------------------------------- |
| 7a   | Rendering components, ready-made objects, the maths that turns a scene into a draw list (done) |
| 7b   | A software renderer (runs on the CPU, no GPU needed) and the real Scene / Game views (done) |
| 7c   | The editor camera (orbit, pan, zoom), picking and selection, 2D sprites, materials |
| 7d   | A GPU backend that draws the same draw list                                       |

The software renderer comes first on purpose: it works on every computer, its output can be checked
exactly in tests, and the GPU backend can later be compared against it.

## Components

They have their own names (not Unity's). All are added from **Inspector > Add Component** or made
with **Hierarchy > + Create**. Their fields show in the Inspector like any other (see `docs/INSPECTOR.md`).

| Component    | Does                                   | Fields                                                              |
| ------------ | -------------------------------------- | ------------------------------------------------------------------- |
| `ViewCamera` | what the Game view shows               | projection (Perspective / Orthographic), field of view, size, near, far, background, priority |
| `LightSource`| a light                                | kind (Directional / Point / Spot), color, intensity, range, spot angle, shadows |
| `MeshDraw`   | draws a mesh (mesh filter + renderer)  | mesh (Cube, Sphere, Plane, Cylinder, Capsule), color, unlit, cast / receive shadows, visible |
| `SpriteDraw` | draws a flat rectangle or picture      | color, flip X / Y, order, size, visible                             |

`Transform` and `RectTransform` are the placement components and keep their names for now.
Mesh files, sprite images and materials come with the asset step (10) and the shader steps (12).

Fields show or hide as others change: the camera shows *Field of View* only for Perspective and *Size* only
for Orthographic, a Directional light has no *Range*, only a Spot light has a *Spot Angle*.

## Ready-made objects (Create menu)

Empty, Cube, Sphere, Plane, Cylinder, Capsule, Camera, Directional Light, Point Light, Spot Light, Sprite.
The camera starts pulled back at (0, 1, -10), the directional light tilted (50, -30, 0), the point and
spot lights above the middle.

## The software renderer (`grp/src/software`)

`render(frame, width, height)` draws a `RenderFrame` into an `Image` (red, green, blue, alpha bytes).
It implements the `RenderBackend` trait, which the GPU renderer (7d) will implement too.

- **Meshes:** transformed by the camera, cut at the near plane, faces pointing away from the camera are skipped,
  the nearest surface wins (depth buffer), colors and normals are interpolated with perspective correction.
- **Lighting:** a lit mesh is its color times (ambient light + every light). Directional lights shine everywhere,
  point lights fade out smoothly to their range, spot lights add a cone. There are no shadows yet (the shadow
  switches are stored for later). *Unlit* meshes show their color as it is. No gamma correction.
- **Sprites:** drawn after the meshes, in order (a higher order on top), flat colored, hidden by meshes in front.
- **Shapes** (`mesh.rs`): Cube 1 x 1 x 1, Sphere diameter 1, Plane 10 x 10 facing up, Cylinder radius 0.5 and height 2,
  Capsule radius 0.5 and height 2. All centered on their object.

### In the editor

The **Game** view draws the scene from its camera (the ViewCamera with the highest priority, and a note if
there is none). The **Scene** view uses a built-in camera for now, orbit / pan / zoom come in 7c.
The picture is drawn again only when the scene or the panel size changes, and never wider than 640 pixels
(it is stretched). Drawing on the CPU is slow in a debug build: use `cargo run --release -p flux-editor`.
The demo scene has a ground, a crate and a ball so there is something to see.

## From scene to draw list (`grp/src/frame`)

`collect(scene, camera)` gives a `RenderFrame`: the camera matrices, then meshes, sprites and lights, each with
its world matrix.

- **Coordinates:** left-handed, Y up, Z forward. Rotation is Euler angles in degrees, applied Z, then X, then Y.
- **Camera:** the active ViewCamera with the highest priority (the first one wins a tie), or the built-in
  fallback camera. The editor's Scene view will pass its own camera.
- **Left out:** inactive objects, objects inside inactive ones, and meshes / sprites with *visible* off.
- **Sprites** are sorted by order, so later ones draw on top. A sprite's matrix includes its size.
- **Lights** shine along the +Z direction of their object. Their position and direction are in world space.
