# Step 4b - Editor window and drawing

Date: 2026-11-09 | Status: not compiled yet

## feat
- **editor:** egui window (`cargo run -p flux-editor`) with a menu bar and a status bar.
- **editor:** dock view: tab groups, draggable dividers, close buttons, drag a tab to another
  group (edge = split, middle = tab) with a highlighted drop zone, Escape to cancel.
- **editor:** Hierarchy panel: select, Ctrl+select, expand, create, delete, right-click menu,
  drag a row onto another to reparent, drop in the empty area for the top level.
- **editor:** Inspector panel: name, active switch, Transform / RectTransform / Script components.
- **editor:** slide-to-change numbers in the Inspector: <-> cursor on hover, pointer hidden while
  dragging, Shift fine, Ctrl coarse, Escape cancels.
- **editor:** themes applied to every widget, shortcuts from the keybinds, layout / themes /
  keybinds saved as YAML when changed and on exit.
- **editor:** views only read models and push `UiCommand`s, `commands.rs` applies them
  through the controllers.
- **engine:** `Vec2` / `Vec3` `get` / `set` by axis.
- **engine:** `SceneController::update_component` (a component can be edited but not change kind).

## build
- **workspace:** `eframe 0.29` added, pinned because the API changes between versions.

## test
- **editor:** command layer (number drag, cancel, create, refused reparent, theme, shortcuts),
  field read/write, settings store.

## docs
- `docs/EDITOR_UI.md`.

## Known issues
- Scene, Game, Project, Animator, Timeline, Terrain, Profiler are placeholders.
- The hidden pointer is not locked or wrapped while dragging a number.
- No rename inside the hierarchy yet (rename in the Inspector). No undo/redo yet.
- The drawing code can only be checked by running it.
