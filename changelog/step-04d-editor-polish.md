# Step 4d - Editor polish

Date: 2026-11-11 | Status: not compiled yet

## feat
- **editor:** the hidden pointer wraps around the window edges while a number is dragged, and the
  jump it causes is ignored, so a long drag never runs out of room.
- **editor:** rename in the Hierarchy: double-click, F2 or right-click > Rename. Enter or clicking
  away accepts, Escape cancels, an empty name changes nothing. One undo step, typing merges.
- **editor:** the Inspector can Reset a Transform / RectTransform to its defaults and Remove a
  component that is not a Transform. Both are undoable.
- **editor:** closing the editor with unsaved changes asks: Save and close / Close without saving /
  Cancel. If saving fails the editor stays open and the status bar says why.
- **editor:** the Inspector says when an object is a prefab instance.

## fix
- **engine:** `Vec3` converts into `[f32; 3]`. The Inspector had relied on that conversion since
  step 4b without it existing, which would not have compiled.

## test
- **editor:** pointer wrap and jump rules, rename flow (accept, cancel, empty, undo, F2),
  rename ends when the object disappears, reset / remove component flows.
- **engine:** vector conversion and axis setting.

## docs
- `docs/EDITOR_UI.md`: rename, reset / remove, pointer wrap, unsaved-changes question.

## Known issues
- Components that don't exist yet have no Inspector fields, they arrive with their steps.
- The window-closing question and the pointer wrap can only be checked by running the editor.
