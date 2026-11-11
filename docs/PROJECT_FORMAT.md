# Project format

A project is a folder. The launcher creates it, the editor opens it
(`flux-editor --project <folder>`). All files are YAML.

```
MyGame/
  project.yaml            name, engine version, graphics pipeline, Aphelion's Control
  .gitignore              ignores Library/
  Assets/
    Scenes/               <name>.scene.yaml
    Prefabs/              (prefabs arrive with the asset steps)
    Scripts/              (C# scripts arrive with the scripting step)
  Settings/
  Library/                id_registry.yaml (and later caches). Safe to ignore in git.
  .aphelion/              only if Aphelion's Control was chosen at creation
```

## project.yaml

```yaml
format: project
version: 1
data:
  name: Space Game
  engine_version: 1.0.0
  pipeline: hgp            # sgp = Simple Graphics Pipeline, hgp = Heavy Graphics Pipeline
  aphelion_control: true
  created_on: 2026-11-10
```

- `pipeline` and `aphelion_control` are chosen when the project is created and **never change**.
  There is no controller function that edits them.
- `engine_version` is the version the project opens with. Changing it in the launcher also
  rewrites this file.
- The name is also a folder name, so it can't contain `\ / : * ? " < > |`, and is at most 64 characters.

## Ids

- Every object has a uuid and every component an integer id (from 10000). They are written
  in the scene file and never change.
- `Library/id_registry.yaml` remembers every id ever handed out, including ids of objects that
  were deleted, so an id is never reused.
- When a scene is opened its ids are added to the registry. A scene file that contains the
  same uuid or component id twice is refused, with a message saying which.
- If the registry file is lost, it is rebuilt from the scenes (only ids of already deleted
  objects are forgotten).

## Saving

Files are written to a temporary file first and then renamed over the real one, so stopping
the program in the middle of a save can't leave a half-written scene.
The editor shows `*` in the window title and "unsaved changes" in the status bar until you
press Ctrl+S (File > Save Scene).
