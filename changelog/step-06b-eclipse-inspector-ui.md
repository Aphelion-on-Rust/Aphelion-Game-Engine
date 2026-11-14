# Step 6b - Eclipse Inspector UI

Date: 2026-11-11 | Status: not compiled yet

## feat
- **editor:** the Inspector now draws every component from the description of its fields instead of
  hard-coded rows. Transform, RectTransform and scripts all use the same drawing code.
- **editor:** a widget for every field kind: checkbox, slide-to-change number (or a slider with
  `[Range]`), text (several lines with `Multiline`), Vec2 / Vec3 number rows, color picker, enum
  drop-down, object picker, lists (add / remove elements) and structs (indented foldouts), nested to any depth.
- **editor:** attributes at work: headers, spaces, foldout groups, tooltips, labels, read-only (greyed
  out), hidden fields, `ShowIf` / `HideIf`, `Min` and `Range` limits in the number boxes.
- **editor:** right-click a field name for "Reset to default".
- **editor:** new commands `SetField`, `ResetField`, `ListPush`, `ListRemove`. Typing in a field is
  one undo step (steps for the same field merge), add / remove / reset are one step each.
- **engine:** `InspectController::get_field` and `inspect::query::read_field`.
- **editor:** `NumberFieldConfig::for_field`: the limits, whole-number counting and drag speed of a
  number box come from the field's kind and attributes (a range takes about 200 pixels to cross).

## fix
- **engine:** a struct that gains a field in its schema can now have that field set (it gets an entry
  when first edited). Before, editing it was refused.

## refactor
- **editor:** number boxes edit a `NumberTarget` made of object, component, field path and part
  (X / Y / Z / whole) instead of a hard-coded list of Transform fields. The old `ComponentField`
  list and `read_field` / `write_field` helpers are gone.
- **editor:** number drags go through the same rules as every other change (`set_field`).

## test
- **editor:** number target maths, number box settings from attributes, and the command flows on
  the demo script: range, wrong kind, typing merged into one step, lists, reset, a number drag
  inside a struct.
- **engine:** reading a field by path, a struct entry that did not exist yet.

## docs
- `docs/INSPECTOR.md` has an "In the editor" section, `docs/EDITOR_UI.md` is updated.

## Known issues
- The drawing code can only be checked by running the editor (the Inspector against the demo Player
  script shows every kind of field).
- Object pickers list all objects of the scene by name, there is no search yet.
- Several objects selected at once are not edited together.
- Foldout open / closed state is remembered by the window, not saved.
