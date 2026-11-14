# Step 5c - Aphelion's Control and prefab saving

Date: 2026-11-10 | Status: not compiled yet

## feat
- **engine:** `control` module (Aphelion's Control): every prefab save is stored as a revision of
  changes (objects added / removed / renamed / switched / moved, components added / removed /
  edited), each with its old and new value.
- **engine:** reverting a revision undoes only that revision's changes. A change that later edits
  touched again is kept and reported. The first revision can't be reverted. A revert is saved as
  a new revision, history is never rewritten.
- **engine:** `PrefabController::update_from_instance` ("apply"): writes a scene instance back into
  its prefab, keeping the prefab's uuids and component ids for everything that lines up.
- **engine:** `SceneController::set_prefab_source`.
- **editor:** Create Prefab, Apply Changes to Prefab (Hierarchy right-click), Place in scene
  (Project panel), prefab files in `Assets/Prefabs`, a list of the project's prefabs.
- **editor:** Aphelion's Control panel: prefabs, revisions newest first, Revert button.
- **editor:** history files are written to `.aphelion/prefabs/<prefab uuid>.yaml` and read when
  the project opens. Ids of prefabs and of everything in the histories are registered on open.
- **editor:** a prefab file that can't be read is a warning in the status bar, it does not block
  the project.
- **core:** layout paths for prefab files and history files, `.aphelion/prefabs` is created with
  the project.

## refactor
- **core:** the date helpers moved from the launcher to `flux-core` (`flux_core::clock`).
- **editor:** `project_io` reads scenes and prefabs with one shared folder-listing function.
- **engine:** `PrefabData` can be compared (`PartialEq`).

## test
- **engine:** revert only what a revision changed, conflicts kept and reported, removed objects
  come back in place, added objects removed unless they have newer children, first revision,
  numbering, YAML round trip of a history, the revision rows, prefab update keeping ids.
- **editor:** prefab library, opening with prefabs / histories / broken files, ids registered
  without saving the registry, and the whole flow (create, apply, revert, place, reopen,
  no Aphelion's Control).

## docs
- `docs/APHELION_CONTROL.md`, `docs/PROJECT_FORMAT.md`, `docs/EDITOR_UI.md`.

## Known issues
- Instances in scenes are not updated when a prefab changes (no prefab propagation yet).
- Prefabs are edited by applying an instance. There is no "open prefab" editing mode yet.
- "Apply" lines children up by position, so reordering children in an instance matches by position.
- Prefabs are one file per name, names are unique per project.
- A history file is rewritten whole on every save (fine for now).
