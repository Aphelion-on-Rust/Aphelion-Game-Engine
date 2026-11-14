# Step 3b - Script lifecycle events

Date: 2026-11-09 | Status: not compiled yet

## feat
- **engine:** `lifecycle`: 37 events (Awake, OnEnable, Start, FixedUpdate, Update, OnIK, LateUpdate,
  OnPreRender, OnPostRender, OnMouse*, OnCollision*, OnTrigger*, OnBecame*, OnApplication*, ...).
- **engine:** `LifecycleController` decides who gets what: Awake / OnEnable / Start in order and
  once, OnDisable / OnDestroy when scripts or objects go away, nothing for switched-off scripts.
- **engine:** `MouseEventController`: hover + mouse input become Enter / Exit / Over / Down /
  Up / Click / Drag events.
- **engine:** `ScriptRunner` plugs into the game loop and sends every `Dispatch` to a `ScriptHost`.
- **engine:** `Script` component (script uuid + enabled) and `is_active_in_hierarchy`.

## test
- **engine:** event order, enable/disable cycles, destroy, queued events, shutdown, full click,
  runner frames.

## docs
- `docs/LIFECYCLE.md`: every event, when it fires, and which system fires it.

## Known issues
- Detecting collisions, visibility, picking and animation events comes with their own steps.
  Their events can already be sent with `queue_event` / `broadcast`.
- The real `ScriptHost` (C#) comes with the scripting step.
