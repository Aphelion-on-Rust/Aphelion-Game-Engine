# Play mode

Press **Play** (toolbar under the menu, or **Ctrl+P**) and the scene runs with physics: things with a `Body3D` fall,
land, bounce and slide, and you watch it in the Scene and Game views and in the Inspector (the Transform numbers change
live). Press **Stop** and the scene is put back exactly as it was.

## How it works

- **Play** saves a copy of the scene and builds the physics world from it (`PhysicsWorld::from_scene`, see
  `docs/PHYSICS.md`).
- Every screen frame the game is moved forward by the time the frame took, in fixed steps of 1/60 s, so the physics
  behaves the same on a fast and a slow screen. If a frame takes too long (more than 5 steps), the game slows down
  instead of freezing.
- After each frame the new positions of the bodies are written into the Transforms. A Transform holds a position inside
  its parent, so the world position is converted back (an object that moves under a parent works).
- **Pause** freezes the game. **Resume** continues.
- **Stop** brings the saved copy back. Anything that happened while playing is gone.

## What is turned off while playing

While playing, the scene is a temporary copy, so everything that would change it for good is refused with a message:
creating, deleting, renaming and moving objects, changing any value (including dragging numbers), adding and removing
components, prefab actions, **saving**, and **undo / redo**. Looking around still works: selecting objects, the Scene view
camera (orbit, pan, zoom, focus, click to select), panels and themes.

The window title shows `[playing]`, and the "unsaved changes" mark compares the scene Stop will bring back, so playing
alone never makes the scene look changed.

## Toolbar

`Play` / `Stop` and `Pause` / `Resume`, the time played, and the last thing that happened between two objects
(`Hit: Ball / Floor`, `Trigger enter: Ball / Zone`). Trigger and collision events are not sent to scripts yet, that comes
with the scripting host.

## Try it

1. Open a project (or the demo), then **+ Create > Floor**, **Physics Cube** and **Physics Sphere**.
2. Press Play. The cube and the sphere fall onto the floor.
3. Press Stop. They are back up in the air where you put them.

## Files

- `editor/src/play/` the state (model), the questions the views ask (query), and start / pause / stop / advance (controller)
- `editor/src/ui/toolbar.rs` the buttons
- `editor/src/ui/app.rs` holds the physics world while playing and moves the game forward every frame
