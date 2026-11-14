# Rendering

The renderer is built in steps. Step 7a (done) is everything that happens **before** a pixel is drawn.

| Step | What                                                                              |
| ---- | --------------------------------------------------------------------------------- |
| 7a   | Rendering components, ready-made objects, the maths that turns a scene into a draw list |
| 7b   | A software renderer (runs on the CPU, no GPU needed) and the real Scene / Game views |
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

## From scene to draw list (`grp/src/frame`)

`collect(scene, camera)` gives a `RenderFrame`: the camera matrices, then meshes, sprites and lights, each with
its world matrix.

- **Coordinates:** left-handed, Y up, Z forward. Rotation is Euler angles in degrees, applied Z, then X, then Y.
- **Camera:** the active ViewCamera with the highest priority (the first one wins a tie), or the built-in
  fallback camera. The editor's Scene view will pass its own camera.
- **Left out:** inactive objects, objects inside inactive ones, and meshes / sprites with *visible* off.
- **Sprites** are sorted by order, so later ones draw on top. A sprite's matrix includes its size.
- **Lights** shine along the +Z direction of their object. Their position and direction are in world space.
