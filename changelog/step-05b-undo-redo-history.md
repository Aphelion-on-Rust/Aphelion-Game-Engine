# Step 5b - Undo / redo history

Date: 2026-11-10 | Status: not compiled yet

## feat
- **editor:** `history` module: undo / redo of scene edits, kept as snapshots of the scene
  (simple and always correct).
- **editor:** one edit is one step. A whole number drag is one step, typing a name is one step,
  a command that changes nothing is no step, a new edit after an undo drops the redo steps,
  at most 100 steps are kept.
- **editor:** History panel: the list of states the scene went through, click one to undo or
  redo up to it. It is in the default layout next to Project and Console.
- **editor:** Edit menu Undo / Redo with the step name and shortcut, greyed out when there is
  nothing to do. Ctrl+Z / Ctrl+Y work (the keybinds already existed).
- **editor:** the unsaved-changes marker goes away again when undo returns to the saved scene.
- **editor:** `HierarchyController::prune_missing` drops selected objects that no longer exist.

## refactor
- **editor:** `apply_all` records the history around every command that edits the scene.
- **editor:** the default layout has seven panels (History added).

## test
- **editor:** undo / redo restore the scene, redo cleared by a new edit, merged steps, limit,
  groups (one step, unchanged = none, cancel), clear, the history rows, and the command flows
  (create, drag, cancel, rename, nothing changed, stale selection, jump several steps).

## docs
- `docs/EDITOR_UI.md`, `docs/ROADMAP.md`.
- Changelog files are named by step. There are no version numbers, the project stays at 1.0.0.

## chore
- **workspace:** project version set to 1.0.0 and kept there for now.

## Known issues
- The history is not saved, closing the editor forgets it.
- Selection and expanded rows are not part of undo, only scene data is.
- A scene with tens of thousands of objects would need a lighter history (changes instead of
  copies). Nothing outside `history` would have to change.
