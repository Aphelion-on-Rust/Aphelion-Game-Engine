# Step 7a - Render components and the draw list

Date: 2026-11-12 | Status: not compiled yet

## feat
- **engine:** four components that make things appear, with their own names: `ViewCamera`,
  `LightSource`, `MeshDraw` (built-in Cube, Sphere, Plane, Cylinder, Capsule) and `SpriteDraw`.
- **engine:** `ComponentData` gained the four kinds, plus `type_name`, `reset` and `new_of_kind`.
- **engine:** ready-made objects (`ObjectPreset`: Empty, Cube, Sphere, Plane, Cylinder, Capsule, Camera,
  Directional / Point / Spot Light, Sprite) and `SceneController::create_from_preset`.
- **grp:** `frame` module: world matrices from the parent chain, the game camera (highest priority,
  with a fallback camera), perspective and orthographic projection, and `collect`, which turns a scene
  into a `RenderFrame` of meshes, sprites (sorted by order) and lights. Left-handed, Y up, Z forward,
  Euler rotation in the order Z, X, Y. No GPU.
- **editor:** Hierarchy `+ Create` is a menu of the ready-made objects, the Inspector has
  `Add Component`, and every new component is edited like any other (the camera shows Field of View only
  for Perspective, a Directional light has no Range, only a Spot light has a Spot Angle).

## refactor
- **engine:** the descriptions of the built-in components are now one table per component. Every field is
  written once (name, kind, attributes, get, set) and the schema, reader and writer are made from it.
- **editor:** resetting a component calls `ComponentData::reset` instead of listing the kinds.
- **editor:** the Inspector takes the component name from `type_name`.

## build
- **grp:** depends on `glam` for the matrix maths.

## test
- **grp:** nested parent matrices, the rotation order, skipped inactive / invisible things, camera
  priority and ties, perspective and orthographic projection, light position and direction, sprite
  sorting and size, the fallback camera.
- **engine:** presets, addable kinds, reset, every described field readable and writable, enum
  options, show / hide rules of the camera.
- **editor:** create a preset, set a camera field (kept in range), reset, add a component, refuse
  an unknown one.

## docs
- `docs/RENDERING.md` (the plan for step 7 and the components), updates to `docs/INSPECTOR.md` and
  `docs/EDITOR_UI.md`.

## Known issues
- Nothing is drawn yet. The Scene and Game panels stay placeholders until 7b.
- `Transform` and `RectTransform` keep their names. Say so if you want them renamed too.
- Meshes are the built-in shapes only, sprites are plain rectangles, there are no materials or textures yet.
- A RectTransform object is only moved by its position, UI drawing comes later.
