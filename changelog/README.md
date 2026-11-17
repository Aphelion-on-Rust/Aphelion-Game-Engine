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