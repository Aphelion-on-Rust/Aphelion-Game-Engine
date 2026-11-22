# Flux Aphelion

A 2D/3D game engine written in Rust, with a Unity/Cocos-style editor and C# scripting.

Author: atropinex

## Layout

| Folder        | What lives there                                              |
| ------------- | ------------------------------------------------------------- |
| `/core`       | Shared base: MIVC building blocks, errors, YAML storage       |
| `/engine`     | Engine: ids, scenes, components, loop, input, GC, lifecycle   |
| `/editor`     | The editor (panels, inspector, themes, keybinds)              |
| `/launcher`   | Separate app: downloads engine versions from GitHub, installs, starts the editor |
| `/scripting`  | C# scripting host                                             |
| `/grp`        | Graphics Render Pipeline (SGP and HGP)                        |
| `/extensions` | Extension API (editor customization, build templates)         |
| `/docs`       | Architecture, roadmap, guides                                 |
| `/changelog`  | What changed in every version (Conventional Commit style)     |

Read `docs/ARCHITECTURE.md` first, then `changelog/CHANGELOG.md`.

## Build

```
cargo check
cargo test
cargo run -p flux-editor
cargo run -p flux-launcher
```
