# Roadmap

Heavy steps are done one at a time. Easy steps are grouped. After each step you get a new zip
and a changelog entry (`changelog/`).

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

About 9 deliveries and 200 files so far.

## To do (estimates)

"One delivery" = one zip, which is one reply of the size you have been getting.
File counts include tests and docs. Treat every number as +/- 40%.

| Step | What                                                                 | Deliveries | New files |
| ---- | -------------------------------------------------------------------- | ---------- | --------- |
| 5c   | Aphelion's Control: prefab save history, rollback of what was edited | 1          | 15        |
| 4d   | Editor polish: pointer wrap, hierarchy rename, more inspector fields | 1          | 10        |
| 6    | Eclipse Inspector, Serializable, [Range]                             | 2          | 35        |
| 7    | SGP renderer: window integration, 2D sprites, 3D meshes, materials, lights | 4    | 70        |
| 8    | Physics 2D + 3D: rigidbody, all colliders, joints, character controller, raycasts | 4 | 70   |
| 9    | C# scripting host: load assemblies, bind the engine API, serialized fields | 3     | 45        |
| 10   | Asset pipeline: FBX / humanoid, atlas (convert, trim), audio, importers | 3       | 50        |
| 11   | Animation: timeline, visual-graph animator, bone rigs, IK            | 4          | 60        |
| 12   | HGP: shaders, shader graph, LoD, ray tracing (C++), baked light, probes | 5       | 90        |
| 13   | Graphics quality UI (levels, ray tracing, DLSS, frame boost)         | 1          | 12        |
| 14   | Terrain, UI components, Rich Text Label, audio mixer                 | 4          | 60        |
| 15   | Camera system (Cinemachine-like), mesh editing tool (ProBuilder-like) | 2         | 35        |
| 16   | Input system UI, extensions, Windows build template                  | 3          | 45        |
| 17   | Documentation pass                                                   | 2          | 25        |
|      | **Total**                                                            | **about 39** | **about 560** |

So roughly 39 more deliveries and 530 to 590 more files, about 760 files in the end.

## Where it can grow

- **7, 8, 9, 12** are the risky ones: they depend on outside libraries (a GPU library for
  rendering, a physics library or our own, .NET hosting for C#) and on hardware features
  (ray tracing, DLSS). Each can grow by one or two deliveries.
- Everything is written without being compiled here. Every few deliveries you should run
  `cargo check` and `cargo test` and send the errors. Fixing them early costs one small delivery,
  fixing them late costs several.
- This gives you a readable, working foundation of each system in the list. Matching everything
  Unity has, at Unity's quality, is far beyond what any plan like this can cover.
