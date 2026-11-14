# Step 2 - Scene core

Date: 2026-11-09 | Status: not compiled yet

## feat
- **engine:** `math`: `Vec2`, `Vec3`.
- **engine:** `scene`: GameObject, Scene, `Transform` and `RectTransform` components,
  `SceneController` (create, add / remove component, set parent, destroy, rename, active),
  read-only helpers in `query.rs`.
- **engine:** `prefab`: save a branch as a prefab, place copies with brand new uuids and
  component ids, the placed root remembers its prefab.
- **engine:** errors for missing objects / components and invalid operations.
- **engine:** rules enforced: one Transform or RectTransform per object, no parenting into own
  child, ids are never reused after destroying an object.

## test
- **engine:** hierarchy links, cycles, destroy, second transform refused, prefab ids,
  scene YAML round trip.

## docs
- Scenes and prefabs section in `docs/ARCHITECTURE.md`.
