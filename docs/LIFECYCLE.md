# Script lifecycle

The C# method name is the event name. Scripts only need to write the ones they use.

## Order inside a frame

```
sync        Awake, OnEnable (new / re-enabled scripts), then Start
fixed step  FixedUpdate           (0, 1 or more times per frame)
update      OnMouse... events, Update, OnIK, LateUpdate
render      OnPreRender
```

## Setup and teardown

| Event       | When                                                             |
| ----------- | ---------------------------------------------------------------- |
| Reset       | Editor only. Script added or "Reset" pressed                     |
| OnValidate  | Editor only. A value changed in the Eclipse Inspector            |
| Awake       | Once, the first time the object is active (even if script is off)|
| OnEnable    | Every time the script becomes enabled and the object is active   |
| Start       | Once, after the script is first enabled, before its first update |
| OnDisable   | Script turned off, object turned off, or before OnDestroy        |
| OnDestroy   | Object destroyed, script removed, or the game closes             |

An object inside a switched-off parent counts as switched off.

## Every frame

| Event         | Notes                                                    |
| ------------- | -------------------------------------------------------- |
| FixedUpdate   | Steady rate (default 60/s). Physics code goes here       |
| Update        | Once per frame                                           |
| OnIK(layer)   | After Update. Animator calls it once per layer (step 11) |
| LateUpdate    | After everything above                                   |
| OnPreRender   | Just before drawing                                      |
| OnPostRender  | Just after drawing (fired by the renderer, step 7)       |
| OnDrawGizmos  | Editor only (fired by the editor, step 4)                |

## Mouse (object under the cursor)

OnMouseEnter, OnMouseExit, OnMouseOver (every frame while over),
OnMouseDown(button), OnMouseUp(button), OnMouseClick(button) (down and up on the same object),
OnMouseDrag(button) (every frame while held, starting the frame after the press).

Finding the object under the cursor needs colliders, so it arrives with physics (step 8).
Until then `ScriptRunner::set_hovered` is how the object is passed in.

## Other events (fired by the system named)

| Event                                                         | Fired by                  |
| ------------------------------------------------------------- | ------------------------- |
| OnCollisionEnter/Stay/Exit(other), OnTriggerEnter/Stay/Exit(other) | Physics, 2D and 3D (step 8) |
| OnParticleCollision(other), OnJointBreak(force)               | Physics / particles       |
| OnBecameVisible, OnBecameInvisible                            | Renderer (step 7)         |
| OnTransformParentChanged, OnTransformChildrenChanged          | Scene (later)             |
| OnAnimationEvent(name)                                        | Animation (step 11)       |
| OnApplicationFocus, OnApplicationPause, OnApplicationQuit     | Window / game loop        |

Each of these already works through `queue_event` or `broadcast`. Only the part
that detects them is still to come.

## How it is built (MIVC)

- `LifecycleController` only decides, and returns a list of `Dispatch` (object, script, event).
- `ScriptHost::invoke` carries a Dispatch out. The C# host (step 9) will implement it.
- `ScriptRunner` plugs the controller into the game loop (it is a `LoopCallbacks`).
