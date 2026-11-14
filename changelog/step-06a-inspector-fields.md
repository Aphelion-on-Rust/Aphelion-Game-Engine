# Step 6a - Field descriptions, attributes and script fields (Eclipse Inspector, part 1)

Date: 2026-11-11 | Status: not compiled yet

## feat
- **engine:** `inspect` module: a component describes its fields (kind, default, attributes) and the
  Inspector will draw from that. Kinds: Bool, Int, Float, Text, Vec2, Vec3, Color, Enum, ObjectRef,
  List and Struct (the `[Serializable]` class), nested to any depth.
- **engine:** attributes: Range, Min, Label, Tooltip, Header, Space, ReadOnly, Hide, Multiline, Group,
  ShowIf, HideIf.
- **engine:** the rules: the kind must fit, Range and Min keep values inside (ints are rounded),
  enum options are checked, read-only is respected everywhere inside, and a refused change leaves
  the scene unchanged.
- **engine:** `InspectController`: set a field, reset it to its default, add / remove list elements,
  and fill a script's missing fields with defaults. Works through paths into structs and lists.
- **engine:** script field values are stored on the Script component (`ScriptData::fields`).
- **engine:** Transform and RectTransform described with the same system, so anchors and pivot are
  kept between 0 and 1.
- **engine:** `layout` (headers, spaces, foldout groups, ShowIf / HideIf) and readable labels
  (`jumpHeight` -> "Jump Height") for the Inspector to use.
- **engine:** `Rgba` color.
- **editor:** script schemas are read from `Library/script_schemas` when a project opens, and scripts
  get default values for their fields. A schema that can't be read is a warning.
- **editor:** the demo scene's Player script has a demo schema with every kind of field.
- **core:** layout paths for the script schema files.

## refactor
- **engine:** `ScriptData::new(script, enabled)` instead of writing the struct out.

## test
- **engine:** rules (clamp, kinds, enums, nested lists and structs, paths and read-only inheritance),
  defaults and syncing, reading / writing by path, the controller (range, refusals, nested values,
  lists, reset, syncing, built-in components, RectTransform anchors, missing schema),
  layout and visibility, built-in fields, and YAML round trips of a scene with script values and
  of a schema using every kind and attribute.
- **editor:** schemas loaded on open, defaults filled in, a broken schema file is a warning.

## docs
- `docs/INSPECTOR.md`: kinds, attributes, rules, the schema and value file formats.

## Known issues
- The Inspector still draws Transform, RectTransform and Script the old way. Step 6b draws every
  component from the descriptions.
- The scripting host (step 9) will write the schema files. Until then they are written by hand.
- Fields added to a schema are filled in, but a struct that gains a field inside an existing
  value is not (it gets the new field when a new value is created).
- `ObjectRef` stores an object uuid. Picking an object comes with the Inspector UI.
