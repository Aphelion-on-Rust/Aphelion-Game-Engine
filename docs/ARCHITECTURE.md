# Architecture: MIVC

| Part       | Job                                   | May change data? |
| ---------- | ------------------------------------- | ---------------- |
| Interface  | Describes the data shape (YAML)       | no               |
| Model      | Stores the data                       | only via token   |
| View       | Reads and shows data                  | never            |
| Controller | All the logic                         | yes              |

## How the rules are enforced (not just a convention)

- A `View` only receives read-only references (`&Model`, or a tuple of them like
  `(&SceneModel, &HierarchyUiModel)`). The `ReadOnly` marker rejects `&mut`, so a view can't change anything.
- `Model::data_mut` needs a `WriteToken`. The token can only be created inside
  `core/src/mivc/controller.rs`, and the only public way to get one is
  `Controller::write(model)`. So only controllers can write.

## Folder pattern for every feature

```
feature/
  interface.rs   data shape (derive Serialize/Deserialize, impl Interface)
  model.rs       usually just `type XModel = Store<XData>`
  controller.rs  logic, `impl Controller for XController {}`
  view.rs        impl View for XView
```

## Files

- Everything is YAML with a header: `format`, `version`, `data`.
- AssetUuid never changes and never repeats. ComponentId starts at 10000.
- The IdController is the only thing that creates ids.

## Scenes and prefabs

- A scene is a flat list of GameObjects. `parent` and `children` hold the hierarchy.
- A component is `ComponentData` (what it is) plus a `ComponentId` (fixed, from 10000).
- Prefabs are saved copies. Placing one gives every object and component new ids.
- `scene/query.rs` has read-only lookups that views are allowed to call.

## Script lifecycle

- Controllers decide and return `Dispatch` lists. A `ScriptHost` carries them out.
- This keeps all lifecycle rules testable in Rust, without C# running.
- Events by name and timing: `docs/LIFECYCLE.md`.

## Crates

```
core      MIVC base, errors, YAML storage, settings folder     (no engine, no UI)
engine    scenes, ids, loop, input, GC, script lifecycle        (depends on core)
editor    the editor window                                    (depends on engine, egui)
launcher  downloads and installs engine versions               (depends on core only, egui)
```

`engine` re-exports `core`'s `mivc`, `error` and `storage`, so `flux_engine::mivc::...` works.

## Side effects stay out of MIVC

In the launcher the MIVC folders never touch the network or the disk. Anything that does
(download, unzip, delete, start a program) lives in `launcher/src/services`. Controllers
answer a command with an `Effect` and the app carries it out. That keeps the rules testable.
