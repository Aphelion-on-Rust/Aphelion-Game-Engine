# Roadmap

Heavy steps are done one at a time. Easy steps are grouped. After each step you get a new zip
(named `aphelion-engine-v0.<minor>.<patch>-alpha`, patch 0 to 12, then the minor goes up) and a new section in
`changelog/CHANGELOG.md`.

## Done

- [x] 1  Foundation: workspace, MIVC base, ids, YAML storage
- [x] 2  Scene core: objects, components, Transform / RectTransform, prefabs
- [x] 3  Game loop, time, input state, resource GC
- [x] 3b Script lifecycle events
- [x] 4a Editor logic: layout, themes, keybinds, number fields, hierarchy
- [x] 4b Editor window + drawing (egui)
- [x] 4c Shared core crate + Launcher (GitHub download, install, version select)
- [x] 5a Projects: files, creating them in the launcher (SGP/HGP, Aphelion's Control),
         opening and saving in the editor
- [x] 5b Undo / redo history (History panel, Edit menu, Ctrl+Z / Ctrl+Y)
- [x] 5c Aphelion's Control: prefab saves, revision history, revert only one revision
- [x] 4d Editor polish: pointer wrap, hierarchy rename, reset / remove components, unsaved-changes question
- [x] 6a Eclipse Inspector part 1: field kinds, attributes ([Range] ...), script field values and rules
- [x] 6b Eclipse Inspector UI: every component drawn from its field descriptions
- [x] 7a Render components (ViewCamera, LightSource, MeshDraw, SpriteDraw), Create menu, scene to draw list
- [x] 7b Software renderer (meshes, lights, sprites) and the real Scene / Game views
- [x] 7c Scene view camera (orbit, pan, zoom, focus), picking and selection highlight
- [x] 7d Textures and sprite images (asset files with fixed uuids). Materials come with the shader steps (12).
- [x] 8a Physics base: Collider3D shapes, Body3D data, raycasts, collider outlines in the Scene view
- [x] 8b Physics world: gravity, collisions, friction, bounce, triggers and events
- [x] 8c Play mode in the editor: Play, Pause, Stop on a copy of the scene

About 20 steps and 405 files so far (delivery v0.1.5-alpha).

## To do (estimates)

"One delivery" = one zip, which is one reply of the size you have been getting.
File counts include tests and docs. Treat every number as +/- 40%.

| Step | What                                                                 | Deliveries | New files |
| ---- | -------------------------------------------------------------------- | ---------- | --------- |
| 7e   | GPU backend that draws the same draw list | 1 | 15 |
| 8    | Physics rest: 2D physics, joints, character controller, turning bodies, wheels, mesh and terrain colliders | 2 | 40 |
| 9    | C# scripting host: load assemblies, bind the engine API, serialized fields | 3     | 45        |
| 10   | Asset pipeline: FBX / humanoid, atlas (convert, trim), audio, importers | 3       | 50        |
| 11   | Animation: timeline, visual-graph animator, bone rigs, IK            | 4          | 60        |
| 12   | HGP: shaders, shader graph, LoD, ray tracing (C++), baked light, probes | 5       | 90        |
| 13   | Graphics quality UI (levels, ray tracing, DLSS, frame boost)         | 1          | 12        |
| 14   | Terrain, UI components, Rich Text Label, audio mixer                 | 4          | 60        |
| 15   | Camera system (Cinemachine-like), mesh editing tool (ProBuilder-like) | 2         | 35        |
| 16   | Input system UI, extensions, Windows build template                  | 3          | 45        |
| 17   | Documentation pass                                                   | 2          | 25        |
|      | **Total**                                                            | **about 30** | **about 405** |

So roughly 30 more steps and 385 to 445 more files, about 790 files in the end.

## Where it can grow

- **7, 8, 9, 12** are the risky ones: they depend on outside libraries (a GPU library for
  rendering, a physics library or our own, .NET hosting for C#) and on hardware features
  (ray tracing, DLSS). Each can grow by one or two deliveries.
- Everything is written without being compiled here. Every few deliveries you should run
  `cargo check` and `cargo test` and send the errors. Fixing them early costs one small delivery,
  fixing them late costs several.
- This gives you a readable, working foundation of each system in the list. Matching everything
  Unity has, at Unity's quality, is far beyond what any plan like this can cover.
