# Editor UI (step 4b)

Run it with `cargo run -p flux-editor`.

## What you can do

- Drag a tab onto the edge of another group to split, onto the middle to make a tab.
  The highlight shows where it will land. Escape cancels.
- Drag the dividers between panels. Click the small x on a tab to close it.
- Window menu: open a closed panel, reset the layout. Theme menu: switch Dark / Light.
- Hierarchy: **+ Create** makes a cube, sphere, camera, light, sprite ... (see `docs/RENDERING.md`). Click, Ctrl+click, double-click or F2 to rename in place (Enter accepts, Escape cancels),
  arrows to open, right-click menu, drag a row onto another row
  to make it a child, drop in the empty area to move it to the top level.
- Inspector (the Eclipse Inspector): edit the name, the active switch, and the fields of every
  component. The widgets are drawn from the description of each field (sliders for [Range],
  foldout groups, lists, structs, enums, colors, object pickers), see `docs/INSPECTOR.md`.
  Add Component (bottom) adds a ViewCamera, LightSource, MeshDraw or SpriteDraw.
  Right-click a field name to reset it. Each component has Reset (Transform, RectTransform) or Remove (anything else).
  Hover a number: <-> cursor. Drag it: the pointer hides and the value slides.
  Shift = fine, Ctrl = coarse, Escape = cancel. At the window edge the hidden pointer wraps to
  the other side, so a long drag never runs out of room.
- Shortcuts come from the keybinds (Ctrl+S, Delete, Ctrl+Shift+N for now).
- Started with `--project <folder>` (the launcher does this) the editor opens that project's scene.
  Ctrl+S saves the scene and the id registry. The window title shows `*` for unsaved changes.
  Without `--project` you get a demo scene and nothing can be saved.
- The Project panel shows the project's name, engine version, pipeline and Aphelion's Control.
- Prefabs: right-click an object > Create Prefab / Apply Changes to Prefab. The Project panel lists
  the prefabs with a Place in scene button. Window > Aphelion's Control shows each prefab's
  revisions with a Revert button (only if the project was created with Aphelion's Control).
  See `docs/APHELION_CONTROL.md`.
- Undo / redo: Ctrl+Z and Ctrl+Y, or Edit > Undo / Redo. The History panel lists the steps,
  click one to go back or forward to it. A whole number drag and typing a name are one step each.
  The history is not saved when you close the editor.
- Layout, theme and keybinds are saved when you quit
  (Windows: %APPDATA%\FluxAphelion\editor, as .yaml files).

## Placeholders

Scene and Game (renderer, step 7), Animator / Timeline (step 11),
Terrain (step 14). They show a note for now.

## How it follows MIVC

```
input -> draw (views read &EditorModels, push UiCommand) -> apply_all (controllers) -> cursor -> save
```

- Views: `dock_view.rs`, `panels/*`, `menu_bar.rs`, `number_drag.rs`. They never get `&mut`.
- Controllers: `commands.rs` is the one place that turns a UiCommand into a controller call.
- Models: `models.rs` (EditorModels).
- egui is only the drawing layer. Nothing outside `editor/src/ui` knows about it.

## Known limits

- Closing the editor asks about unsaved changes. The undo history is not saved.
- Wrapping the pointer ignores the one-frame jump it causes. A very fast drag across the whole
  window in a single frame would be ignored too.
- Components that don't exist yet (physics, rendering, audio ...) are not in the Inspector.
  When they arrive they only need a description of their fields.
