# Step 3 - Game loop, time, input, GC

Date: 2026-11-09 | Status: not compiled yet

## feat
- **engine:** `time`: frame delta, elapsed time, fixed steps (1/60), time scale, lag-spike clamp.
- **engine:** `input`: key and mouse state (down / pressed / released), mouse delta, scroll;
  OS key repeat does not count as a new press.
- **engine:** `gc`: ref-counted resources with a grace period and safe handles (slot + generation).
  `collect` returns what to free. C# memory is left to the .NET GC.
- **engine:** `game_loop`: time -> gc -> fixed updates -> update -> render -> collect -> clear input,
  `tick_with_delta` for tests.

## refactor
- **engine:** `Vec2` derives `Default` in `math` instead of an impl in the input module.

## test
- **engine:** fixed steps, pause, clamp, key repeat, mouse delta, grace period, stale handles,
  loop order, input clearing.

## docs
- **docs:** plan changed: the real window moved out of this step (done in step 4b).
