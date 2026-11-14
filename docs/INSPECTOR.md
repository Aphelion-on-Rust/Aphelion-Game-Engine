# The Eclipse Inspector: fields, attributes and schemas

The Inspector does not know what a Transform or a script looks like. Every component describes
its fields and the Inspector draws widgets from that description. A new component only needs a
description, no new Inspector code. The data and rules are in `engine/src/inspect` (step 6a), the drawing is in
`editor/src/ui/field_widgets.rs` (step 6b).

## Field kinds

| Kind      | Value                         | Widget                        |
| --------- | ----------------------------- | ----------------------------- |
| `Bool`    | true / false                  | checkbox                      |
| `Int`     | whole number                  | slide-to-change number, or a slider with `Range` |
| `Float`   | number                        | slide-to-change number, or a slider with `Range` |
| `Text`    | text                          | text box (`Multiline`: several lines) |
| `Vec2` `Vec3` | 2 or 3 numbers            | number boxes in a row         |
| `Color`   | red, green, blue, alpha       | color picker                  |
| `Enum`    | one of a list of names        | drop-down                     |
| `ObjectRef` | another object in the scene | object picker                 |
| `List`    | elements of one kind          | list with add / remove        |
| `Struct`  | named fields (`[Serializable]`) | indented group of fields    |

A `List` of `Struct`s and a `Struct` that holds lists both work, to any depth.

## In the editor

- Each field is one line: the name on the left, the widget on the right. **Hover** the name for
  its tooltip, **right-click** it for "Reset to default".
- A `Vec2` / `Vec3` is a row of slide-to-change boxes. With `Range` or `Min` the box stops at the limits.
- Lists show their elements, an `x` to remove one and `+ Add` at the end. Structs are indented foldouts.
  Both can hold each other to any depth. Lists of structs work.
- `Group` fields share a foldout, `Header` is a bold title, `Space` is room, `ReadOnly` fields are greyed out,
  `ShowIf` / `HideIf` fields appear and disappear as the other field changes.
- Typing in one field, or dragging its slider, is one undo step. A whole number drag is one step too.
- The pointer wraps around the window edges during a number drag (see `docs/EDITOR_UI.md`).
- A script without a schema shows "No field information yet" (the scripting host writes the schema).

## Attributes

They match what a C# script can put on a field.

| Attribute            | C# equivalent        | Effect                                            |
| -------------------- | -------------------- | ------------------------------------------------- |
| `Range {min, max}`   | `[Range(0, 10)]`     | slider; the value always stays between the two    |
| `Min {value}`        | `[Min(0)]`           | the value never goes below it                     |
| `Label {text}`       | label text           | shown instead of the field name                   |
| `Tooltip {text}`     | `[Tooltip("...")]`   | text on hover                                     |
| `Header {text}`      | `[Header("...")]`    | a title above the field                           |
| `Space {height}`     | `[Space]`            | empty room above the field                        |
| `ReadOnly`           | read-only            | shown, but can't be changed (or reset)            |
| `Hide`               | `[HideInInspector]`  | not shown                                         |
| `Multiline {lines}`  | `[TextArea]`         | text field with several lines                     |
| `Group {name}`       | foldout group        | fields with the same name share one foldout       |
| `ShowIf {field, equals}` | show-if          | only shown while another field has that value     |
| `HideIf {field, equals}` | hide-if          | hidden while another field has that value         |

Without a `Label`, the name is made readable: `jumpHeight` and `jump_height` both become "Jump Height".
For a `List`, the attributes of the list field apply to its elements (a `Range` on a list of
floats keeps every element in range).

## The rules (what happens when a value is set)

- The kind must fit. A bool field refuses a number. An int is accepted for a float field.
- `Range` and `Min` are applied: the value is **kept inside**, not refused. An int is rounded.
- An enum only takes one of its options.
- Everything inside a list or a struct is checked the same way, down to the last value.
- A `ReadOnly` field, or anything inside one, can't be changed.
- A refused change leaves the scene exactly as it was.
- Script fields are filled with defaults when a script is added or its schema gains a field.
  Values of fields that are no longer in the schema are kept, nothing is thrown away.

## Script schemas

A script's fields are described in `Library/script_schemas/<script uuid>.yaml`. The scripting host
(step 9) will write these by reading the C# class (`[Serializable]`, `[SerializeField]`, `[Range]`
and so on). Until then you can write them by hand. They are rebuilt, so they are not meant for version control.

```yaml
format: script_schema
version: 1
data:
  script: 4b1e0c52-0a7a-4a33-9d2f-7a8f0e3d9c11     # uuid of the script asset
  class_name: PlayerController
  fields:
    - name: speed
      kind:
        kind: Float
      default:
        type: Float
        value: 5.0
      attributes:
        - attribute: Header
          text: Movement
        - attribute: Range
          min: 0.0
          max: 10.0
    - name: can_fly
      kind:
        kind: Bool
    - name: fly_speed
      kind:
        kind: Float
      attributes:
        - attribute: ShowIf
          field: can_fly
          equals:
            type: Bool
            value: true
```

The values of a script live in the scene, on the Script component:

```yaml
type: Script
id: 10003
script: 4b1e0c52-0a7a-4a33-9d2f-7a8f0e3d9c11
enabled: true
fields:
  - name: speed
    value:
      type: Float
      value: 3.5
```

(Values use `type` + `value` on purpose: YAML's `!Tag` form can't be read back inside the component data.)

## Built-in components

Transform, RectTransform, ViewCamera, LightSource, MeshDraw and SpriteDraw are described in
`inspect/builtin.rs`. Each field is written once (name, kind, attributes, how to get and set it) and the
description, the reader and the writer are all made from that list. A new built-in component adds one list
and one line in each of `builtin_schema`, `builtin_get` and `builtin_set`.
(A derive macro that writes the list from the struct could come later.)

## Layout

`query::layout` turns a list of fields into the order to draw: headers and spaces before their
field, hidden and `ShowIf`-failing fields left out, and all fields of one `Group` collected into
one foldout that sits where the group's first field is.
