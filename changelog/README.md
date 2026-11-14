# Changelog

One file per roadmap step (`docs/ROADMAP.md`). Entries use
[Conventional Commits](https://www.conventionalcommits.org) types, grouped under headings.

**There are no version numbers.** The project version stays at 1.0.0 (root `Cargo.toml`)
until you decide to publish a release.

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

Name the file `step-<number>-<short-name>.md` and add a row to the table below.

## Steps

| Step | Date | Summary |
| ---- | ---- | ------- |
| [6b](step-06b-eclipse-inspector-ui.md) | 2026-11-11 | Eclipse Inspector UI |
| [6a](step-06a-inspector-fields.md) | 2026-11-11 | Field descriptions, attributes and script fields |
| [4d](step-04d-editor-polish.md) | 2026-11-11 | Editor polish |
| [5c](step-05c-aphelion-control.md) | 2026-11-10 | Aphelion's Control and prefab saving |
| [5b](step-05b-undo-redo-history.md) | 2026-11-10 | Undo / redo history |
| [5a](step-05a-projects.md) | 2026-11-10 | Projects |
| [4c](step-04c-launcher-and-core.md) | 2026-11-10 | Launcher and shared core |
| [4b](step-04b-editor-window.md) | 2026-11-09 | Editor window and drawing |
| [4a](step-04a-editor-logic.md) | 2026-11-09 | Editor logic |
| [3b](step-03b-lifecycle-events.md) | 2026-11-09 | Script lifecycle events |
| [3](step-03-loop-time-input-gc.md) | 2026-11-09 | Game loop, time, input, GC |
| [2](step-02-scene-core.md) | 2026-11-09 | Scene core |
| [1](step-01-foundation.md) | 2026-11-09 | Foundation |

Nothing has been compiled yet (the build environment had no Rust).
