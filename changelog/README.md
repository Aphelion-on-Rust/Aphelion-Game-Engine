# Changelog

One file per version. Entries use [Conventional Commits](https://www.conventionalcommits.org)
types, grouped under headings. The version is the project milestone (the number in the root
`Cargo.toml`), and the "Step" says which roadmap step it belongs to (`docs/ROADMAP.md`).

## Types

| Type       | Use it for                                                          |
| ---------- | ------------------------------------------------------------------- |
| `feat`     | something new                                                       |
| `fix`      | a bug fixed                                                         |
| `refactor` | code reorganised, behaviour the same                                |
| `perf`     | faster or lighter                                                   |
| `test`     | tests added or changed                                              |
| `docs`     | documentation                                                       |
| `build`    | dependencies, Cargo, CI                                             |
| `chore`    | housekeeping (folders, tooling)                                     |

Also: **BREAKING CHANGES** (anything that makes old code stop compiling) and **Known issues**
(what is not done or not checked yet).

## Scopes

`core`, `engine`, `editor`, `launcher`, `scripting`, `grp`, `extensions`, `workspace`, `docs`

## Writing an entry

```
- **scope:** what changed, in plain words
```

Add the new file here and a row to the table below.

## Versions

| Version | Date       | Step | Summary                                                     |
| ------- | ---------- | ---- | ----------------------------------------------------------- |
| [0.8.0](0.8.0.md) | 2026-11-10 | 5a | Projects: files, creating them in the launcher, opening and saving in the editor |
| [0.7.0](0.7.0.md) | 2026-11-10 | 4c | Launcher (GitHub download + install), shared `flux-core`   |
| [0.6.0](0.6.0.md) | 2026-11-09 | 4b | Editor window and drawing (egui)                           |
| [0.5.0](0.5.0.md) | 2026-11-09 | 4a | Editor logic: layout, themes, keybinds, number fields, hierarchy |
| [0.4.0](0.4.0.md) | 2026-11-09 | 3b | Script lifecycle events                                    |
| [0.3.0](0.3.0.md) | 2026-11-09 | 3  | Game loop, time, input state, resource GC                  |
| [0.2.0](0.2.0.md) | 2026-11-09 | 2  | Scene core: objects, components, prefabs                   |
| [0.1.0](0.1.0.md) | 2026-11-09 | 1  | Foundation: workspace, MIVC, ids, YAML                     |

Nothing from 0.1.0 to 0.7.0 has been compiled yet (the build environment had no Rust).
