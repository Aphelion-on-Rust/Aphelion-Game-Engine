# Aphelion's Control

A version history for **prefab saves**, chosen when a project is created (it can't be switched
on or off later). It only watches prefabs, not scenes or anything else.

## What you do in the editor

1. Right-click an object in the Hierarchy and choose **Create Prefab**. The object is saved to
   `Assets/Prefabs/<name>.prefab.yaml` and becomes an instance of it. That is **revision 1**.
2. Change the instance (rename things, add children, edit components) and choose
   **Apply Changes to Prefab**. Each apply that really changes something is a new revision.
3. Open **Window > Aphelion's Control**. Pick a prefab to see its revisions, newest first, with the
   date, the message and a short summary like `+1 object, 2 edits`.
4. **Revert** on a revision undoes **only what that revision changed**.
5. The Project panel lists the prefabs. **Place in scene** puts a copy in the scene.

Without Aphelion's Control, prefabs are still saved and placed, they just have no history.

## What "revert" does

Say revision 2 renamed a child and revision 3 moved the root. Reverting revision 2 gives the child
its old name back and leaves the move alone.

If a later revision touched the same thing, that part is **kept** and reported. For example if
revision 2 renamed a child to "A" and revision 3 renamed it to "B", reverting revision 2 changes
nothing and says why. Some other cases:

- an object added by the revision is removed, unless it now has newer children
- an object removed by the revision comes back, unless its parent is gone
- a component edit is undone only if the component still has the value the revision gave it
- the first revision (it creates the prefab) can't be reverted

The revert is saved as a **new revision** ("Reverted revision 2"). History is never rewritten or
deleted. Scene instances are not changed by a revert, they follow when you apply or place again.

## Files

```
.aphelion/prefabs/<prefab uuid>.yaml     one file per prefab
```

```yaml
format: aphelion_prefab_history
version: 1
data:
  prefab: 6f1c...            # the prefab's uuid
  name: Tree
  file: Assets/Prefabs/Tree.prefab.yaml
  revisions:
    - number: 2
      saved_on: 2026-11-10
      message: Applied changes from a scene object
      reverts: null
      changes:
        - change: ObjectRenamed
          object: 8d20...
          from: Leaves
          to: Branches
```

Every change keeps its old and new value, so a revision can be undone without reading any other
one. The kinds are: ObjectAdded, ObjectRemoved (the whole object is kept), ObjectRenamed,
ObjectActiveChanged, ObjectMoved, ComponentAdded, ComponentRemoved, ComponentEdited.
Every id that appears in a history (even of removed objects) is registered when the project opens,
so an id is never used twice.

## How "Apply" lines things up

The instance and the prefab are matched branch by branch (first child with first child, and so
on) and components by kind. Everything that matches keeps the prefab's own uuids and component ids.
New parts get new ids. If you reorder children in the instance, they are matched by position.
